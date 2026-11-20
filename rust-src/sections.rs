use eyre::{Context, ensure};
use facet::Facet;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use teamy_cancellation::CancellationToken;

/// Finite source report. Paths are relative to the configured book source.
#[derive(Facet, Debug, PartialEq, Eq)]
pub struct SectionsReport {
    pub schema_version: u32,
    /// No mdBook includes or arbitrary preprocessors are executed.
    pub content_basis: String,
    pub pages: Vec<Page>,
    pub sections: Vec<Section>,
}
#[derive(Facet, Debug, PartialEq, Eq)]
pub struct Page {
    pub path: String,
    pub title: String,
    pub parent_path: Option<String>,
    pub hierarchy: Vec<String>,
}
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub id: String,
    pub title: String,
    pub level: u8,
}
#[derive(Facet, Debug, PartialEq, Eq)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum SectionKind {
    Preamble,
    Heading,
}
#[derive(Facet, Debug, PartialEq, Eq)]
pub struct Section {
    /// Path plus anchor; heading renaming or earlier duplicate insertion can change it.
    pub id: String,
    pub page_path: String,
    pub kind: SectionKind,
    pub title: String,
    pub level: u8,
    pub anchor: String,
    pub parent_id: Option<String>,
    /// Ancestors only, in root-to-parent order.
    pub heading_hierarchy: Vec<Heading>,
    /// Direct body, excluding heading and descendants.
    pub text: String,
    pub markdown: String,
    /// Zero-based source byte offsets; end is exclusive.
    pub source_start: usize,
    pub source_end: usize,
    /// One-based line containing this record's start.
    pub source_line: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct DiscoveryLimits {
    pub max_page_bytes: u64,
    pub max_total_bytes: u64,
    pub max_pages: usize,
    pub max_sections: usize,
}
impl Default for DiscoveryLimits {
    fn default() -> Self {
        Self {
            max_page_bytes: 2_097_152,
            max_total_bytes: 33_554_432,
            max_pages: 1_024,
            max_sections: 10_000,
        }
    }
}
#[derive(Facet, Debug)]
struct BookConfig {
    book: BookSettings,
}
#[derive(Facet, Debug)]
struct BookSettings {
    #[facet(default = String::from("src"))]
    src: String,
}

/// Discover SUMMARY-linked pages with bounded, canonical-contained reads.
///
/// # Errors
/// Returns config, path, file, UTF-8, limit, anchor-collision, or cancellation errors.
pub fn list_sections(
    book_root: &Path,
    limits: DiscoveryLimits,
    cancellation: &CancellationToken,
) -> eyre::Result<SectionsReport> {
    ensure!(
        limits.max_page_bytes > 0
            && limits.max_total_bytes > 0
            && limits.max_pages > 0
            && limits.max_sections > 0,
        "discovery limits must be positive"
    );
    let root = book_root
        .canonicalize()
        .wrap_err("cannot resolve book root")?;
    let mut total = 0;
    let config_path = contained(&root, Path::new("book.toml"))?;
    let config = read_bounded(&config_path, limits, &mut total, cancellation)?;
    let config: BookConfig = facet_toml::from_str(&config).wrap_err("cannot parse book.toml")?;
    let source = contained(&root, &clean_relative(&config.book.src)?)?;
    ensure!(source.is_dir(), "book source is not a directory");
    let summary_path = contained(&source, Path::new("SUMMARY.md"))?;
    let summary = read_bounded(&summary_path, limits, &mut total, cancellation)?;
    let pages = summary_pages(&summary, limits.max_pages)?;
    let mut sections = Vec::new();
    for page in &pages {
        cancellation.bail_if_cancelled()?;
        let path = contained(&source, Path::new(&page.path))?;
        let markdown = read_bounded(&path, limits, &mut total, cancellation)?;
        let remaining = limits.max_sections.saturating_sub(sections.len());
        sections.extend(page_sections(page, &markdown, remaining, cancellation)?);
    }
    Ok(SectionsReport {
        schema_version: 1,
        content_basis: "markdown-source".to_owned(),
        pages,
        sections,
    })
}
fn read_bounded(
    path: &Path,
    limits: DiscoveryLimits,
    total: &mut u64,
    cancellation: &CancellationToken,
) -> eyre::Result<String> {
    cancellation.bail_if_cancelled()?;
    let bound = limits
        .max_total_bytes
        .saturating_sub(*total)
        .min(limits.max_page_bytes);
    let file = File::open(path).wrap_err_with(|| format!("cannot open {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(bound.saturating_add(1)).read_to_end(&mut bytes)?;
    cancellation.bail_if_cancelled()?;
    ensure!(
        bytes.len() as u64 <= bound,
        "input byte limit exceeded for {}",
        path.display()
    );
    *total += bytes.len() as u64;
    String::from_utf8(bytes).wrap_err_with(|| format!("{} is not UTF-8", path.display()))
}
fn clean_relative(value: &str) -> eyre::Result<PathBuf> {
    ensure!(
        !value.contains(['%', '?', '\\']),
        "unsupported encoded, query, or backslash book path: {value}"
    );
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && path
                .components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)),
        "book path must be relative and cannot traverse parents: {value}"
    );
    Ok(path
        .components()
        .filter_map(|c| match c {
            Component::Normal(v) => Some(v),
            _ => None,
        })
        .collect())
}
fn contained(root: &Path, relative: &Path) -> eyre::Result<PathBuf> {
    let target = root
        .join(relative)
        .canonicalize()
        .wrap_err_with(|| format!("cannot resolve {}", relative.display()))?;
    ensure!(
        target.starts_with(root),
        "book path resolves outside its source root: {}",
        relative.display()
    );
    Ok(target)
}
fn summary_pages(markdown: &str, max_pages: usize) -> eyre::Result<Vec<Page>> {
    let mut depth = 0usize;
    let mut link: Option<(String, String)> = None;
    let mut pages = Vec::new();
    let mut ancestors = Vec::<(usize, String)>::new();
    let mut seen = HashSet::new();
    for event in Parser::new_ext(markdown, options()) {
        match event {
            Event::Start(Tag::List(_)) => depth += 1,
            Event::End(Tag::List(_)) => depth = depth.saturating_sub(1),
            Event::Start(Tag::Link(_, href, _)) => link = Some((href.to_string(), String::new())),
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, title)) = &mut link {
                    title.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, title)) = &mut link {
                    title.push(' ');
                }
            }
            Event::End(Tag::Link(..)) => {
                let Some((href, title)) = link.take() else {
                    continue;
                };
                let level = depth.saturating_sub(1);
                while ancestors.last().is_some_and(|(d, _)| *d >= level) {
                    ancestors.pop();
                }
                if href.is_empty()
                    || href.starts_with('#')
                    || href.contains(':')
                    || href.starts_with("//")
                {
                    ancestors.push((level, String::new()));
                    continue;
                }
                ensure!(
                    ancestors.iter().all(|(_, p)| !p.is_empty()),
                    "local page has an unsupported external or draft SUMMARY parent"
                );
                let raw_path = href.split('#').next().unwrap_or_default();
                let relative = clean_relative(raw_path)?;
                ensure!(
                    relative.extension().is_some_and(|v| v == "md"),
                    "SUMMARY target must be Markdown: {href}"
                );
                let path = relative.to_string_lossy().replace('\\', "/");
                let hierarchy = ancestors.iter().map(|(_, p)| p.clone()).collect::<Vec<_>>();
                let parent_path = hierarchy.last().cloned();
                ancestors.push((level, path.clone()));
                if seen.insert(path.clone()) {
                    ensure!(
                        pages.len() < max_pages,
                        "linked page count exceeds {max_pages}"
                    );
                    pages.push(Page {
                        path,
                        title,
                        parent_path,
                        hierarchy,
                    });
                }
            }
            _ => {}
        }
    }
    Ok(pages)
}
#[derive(Debug)]
struct ParsedHeading {
    title: String,
    level: u8,
    explicit_id: Option<String>,
    range: Range<usize>,
}
fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
}

/// Split direct section bodies without file I/O or process initialization.
///
/// # Errors
/// Returns limit, empty/duplicate anchor, or cancellation errors.
pub fn page_sections(
    page: &Page,
    markdown: &str,
    max_sections: usize,
    cancellation: &CancellationToken,
) -> eyre::Result<Vec<Section>> {
    let headings = parse_headings(markdown, max_sections, cancellation)?;
    ensure!(max_sections > 0, "section count limit exceeded");
    let first = headings.first().map_or(markdown.len(), |h| h.range.start);
    let mut result = vec![make_section(
        page,
        markdown,
        SectionKind::Preamble,
        page.title.clone(),
        0,
        String::new(),
        None,
        Vec::new(),
        0..first,
        0,
    )];
    let mut hierarchy = Vec::<Heading>::new();
    let mut used = HashSet::new();
    let mut counts = HashMap::<String, usize>::new();
    for (index, h) in headings.iter().enumerate() {
        cancellation.bail_if_cancelled()?;
        while hierarchy.last().is_some_and(|a| a.level >= h.level) {
            hierarchy.pop();
        }
        let anchor = if let Some(explicit) = &h.explicit_id {
            ensure!(
                !explicit.is_empty() && used.insert(explicit.clone()),
                "empty or duplicate explicit heading anchor"
            );
            explicit.clone()
        } else {
            let base = slug(&h.title);
            ensure!(
                !base.is_empty(),
                "heading has no usable generated anchor; supply an explicit id"
            );
            let count = counts.entry(base.clone()).or_default();
            let mut candidate = if *count == 0 {
                base.clone()
            } else {
                format!("{base}-{count}")
            };
            while !used.insert(candidate.clone()) {
                *count += 1;
                candidate = format!("{base}-{count}");
            }
            *count += 1;
            candidate
        };
        let id = format!("{}#{anchor}", page.path);
        let end = headings
            .get(index + 1)
            .map_or(markdown.len(), |next| next.range.start);
        result.push(make_section(
            page,
            markdown,
            SectionKind::Heading,
            h.title.clone(),
            h.level,
            anchor,
            hierarchy.last().map(|a| a.id.clone()),
            hierarchy.clone(),
            h.range.end..end,
            h.range.start,
        ));
        hierarchy.push(Heading {
            id,
            title: h.title.clone(),
            level: h.level,
        });
    }
    Ok(result)
}
#[expect(
    clippy::too_many_arguments,
    reason = "internal constructor makes source metadata explicit"
)]
fn make_section(
    page: &Page,
    source: &str,
    kind: SectionKind,
    title: String,
    level: u8,
    anchor: String,
    parent_id: Option<String>,
    heading_hierarchy: Vec<Heading>,
    body: Range<usize>,
    source_start: usize,
) -> Section {
    let markdown = source[body.clone()].trim().to_owned();
    let text = plain_text(&markdown);
    let id = if level == 0 {
        page.path.clone()
    } else {
        format!("{}#{anchor}", page.path)
    };
    let source_line = source[..source_start]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1;
    Section {
        id,
        page_path: page.path.clone(),
        kind,
        title,
        level,
        anchor,
        parent_id,
        heading_hierarchy,
        text,
        markdown,
        source_start,
        source_end: body.end,
        source_line,
    }
}
fn plain_text(markdown: &str) -> String {
    let mut text = String::new();
    for event in Parser::new_ext(markdown, options()) {
        match event {
            Event::Text(value) | Event::Code(value) => text.push_str(&value),
            Event::SoftBreak
            | Event::HardBreak
            | Event::Rule
            | Event::End(Tag::Paragraph | Tag::CodeBlock(_) | Tag::Item | Tag::TableRow) => {
                text.push('\n');
            }
            Event::End(Tag::TableCell) => text.push('\t'),
            Event::TaskListMarker(checked) => text.push_str(if checked { "[x] " } else { "[ ] " }),
            _ => {}
        }
    }
    text.trim().to_owned()
}
fn slug(title: &str) -> String {
    let mut result = String::new();
    for ch in title.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            result.extend(ch.to_lowercase());
        } else if ch.is_whitespace() {
            result.push('-');
        }
    }
    result
}

fn parse_headings(
    markdown: &str,
    max_sections: usize,
    cancellation: &CancellationToken,
) -> eyre::Result<Vec<ParsedHeading>> {
    let mut headings = Vec::<ParsedHeading>::new();
    let mut current: Option<ParsedHeading> = None;
    for (event, range) in Parser::new_ext(markdown, options()).into_offset_iter() {
        cancellation.bail_if_cancelled()?;
        match event {
            Event::Start(Tag::Heading(level, id, _)) => {
                ensure!(
                    headings.len().saturating_add(1) < max_sections,
                    "section count limit exceeded"
                );
                current = Some(ParsedHeading {
                    title: String::new(),
                    level: level as u8,
                    explicit_id: id.map(str::to_owned),
                    range,
                });
            }
            Event::Text(value) | Event::Code(value) => {
                if let Some(h) = &mut current {
                    h.title.push_str(&value);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(h) = &mut current {
                    h.title.push(' ');
                }
            }
            Event::End(Tag::Heading(..)) => {
                if let Some(mut h) = current.take() {
                    h.range.end = range.end;
                    headings.push(h);
                }
            }
            _ => {}
        }
    }
    Ok(headings)
}
