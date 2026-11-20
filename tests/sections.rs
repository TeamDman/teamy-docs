use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use teamy_cancellation::CancellationToken;
use teamy_docs::cli::output::{CliOutput, OutputFormat};
use teamy_docs::sections::{
    DiscoveryLimits, Page, SectionKind, SectionsReport, list_sections, page_sections,
};

fn fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/book")
}
fn report() -> SectionsReport {
    list_sections(
        &fixture(),
        DiscoveryLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap()
}
fn page() -> Page {
    Page {
        path: "nested/test.md".into(),
        title: "Synthetic".into(),
        parent_path: None,
        hierarchy: Vec::new(),
    }
}
#[test]
fn book_order_page_and_heading_hierarchies() {
    let r = report();
    assert_eq!(
        r.pages.iter().map(|p| p.path.as_str()).collect::<Vec<_>>(),
        [
            "index.md",
            "parent.md",
            "nested/child.md",
            "empty.md",
            "plain.md"
        ]
    );
    assert_eq!(r.pages[2].parent_path.as_deref(), Some("parent.md"));
    assert_eq!(r.pages[2].hierarchy, ["parent.md"]);
    let deep = r
        .sections
        .iter()
        .find(|s| s.id == "parent.md#deep")
        .unwrap();
    assert_eq!(deep.parent_id.as_deref(), Some("parent.md#parent"));
    assert_eq!(deep.heading_hierarchy[0].title, "Parent");
    let sibling = r
        .sections
        .iter()
        .find(|s| s.id == "parent.md#sibling")
        .unwrap();
    assert_eq!(sibling.parent_id, deep.parent_id);
}
#[test]
fn direct_bodies_fences_setext_empty_sections_and_preambles() {
    let r = report();
    let intro = r.sections.iter().find(|s| s.id == "index.md").unwrap();
    assert_eq!(intro.kind, SectionKind::Preamble);
    assert_eq!(intro.text, "Intro before a heading.");
    let root = r
        .sections
        .iter()
        .find(|s| s.id == "index.md#what---help-means")
        .unwrap();
    assert_eq!(root.text, "Read help and syntax.");
    assert!(root.markdown.contains("**help**"));
    assert!(!root.text.contains("Another direct"));
    let empty = r
        .sections
        .iter()
        .find(|s| s.id == "index.md#empty")
        .unwrap();
    assert!(empty.text.is_empty());
    let nested = r
        .sections
        .iter()
        .find(|s| s.id == "index.md#skipped-child")
        .unwrap();
    assert!(nested.text.contains("# not a heading"));
    assert!(!r.sections.iter().any(|s| s.title.contains("not a heading")));
    assert!(r.sections.iter().any(|s| s.title == "Setext title"));
    assert!(
        r.sections
            .iter()
            .find(|s| s.id == "empty.md")
            .unwrap()
            .text
            .is_empty()
    );
    assert_eq!(
        r.sections.iter().find(|s| s.id == "plain.md").unwrap().text,
        "Only text, no headings."
    );
}
#[test]
fn ids_anchors_explicit_attributes_and_determinism() {
    let a = report();
    let b = report();
    assert_eq!(a, b);
    let child = a
        .sections
        .iter()
        .filter(|s| s.page_path == "nested/child.md")
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        child,
        [
            "nested/child.md",
            "nested/child.md#foo",
            "nested/child.md#foo-1",
            "nested/child.md#foo-1-1",
            "nested/child.md#chosen"
        ]
    );
    assert!(a.sections.iter().all(|s| !s.page_path.contains('\\')));
    assert_eq!(a.content_basis, "markdown-source");
}
#[test]
fn unicode_crlf_byte_ranges_and_no_final_newline() {
    let input = "Preamble\r\n\r\n# Café\r\nBody é\r\n\r\n## Last\r\nDone";
    let sections = page_sections(&page(), input, 10, &CancellationToken::new()).unwrap();
    assert_eq!(sections[1].anchor, "café");
    assert_eq!(sections[1].source_line, 3);
    assert_eq!(
        &input[sections[1].source_start..sections[1].source_end],
        "# Café\r\nBody é\r\n\r\n"
    );
    assert_eq!(sections[2].text, "Done");
    assert_eq!(sections[2].source_end, input.len());
}
#[test]
fn limits_and_cancellation_reject_without_partial_report() {
    let limits = DiscoveryLimits {
        max_pages: 1,
        ..DiscoveryLimits::default()
    };
    assert!(
        list_sections(&fixture(), limits, &CancellationToken::new())
            .unwrap_err()
            .to_string()
            .contains("page count")
    );
    let limits = DiscoveryLimits {
        max_total_bytes: 3,
        ..DiscoveryLimits::default()
    };
    assert!(
        list_sections(&fixture(), limits, &CancellationToken::new())
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );
    let limits = DiscoveryLimits {
        max_page_bytes: 3,
        ..DiscoveryLimits::default()
    };
    assert!(list_sections(&fixture(), limits, &CancellationToken::new()).is_err());
    let limits = DiscoveryLimits {
        max_sections: 1,
        ..DiscoveryLimits::default()
    };
    assert!(
        list_sections(&fixture(), limits, &CancellationToken::new())
            .unwrap_err()
            .to_string()
            .contains("section count")
    );
    let cancelled = CancellationToken::new();
    cancelled.request_cancel("synthetic cancellation");
    assert!(
        list_sections(&fixture(), DiscoveryLimits::default(), &cancelled)
            .unwrap_err()
            .to_string()
            .contains("synthetic cancellation")
    );
}
#[test]
fn duplicate_explicit_or_empty_generated_ids_fail_explicitly() {
    for input in ["# First {#same}\n# Second {#same}\n", "# !!!\n"] {
        assert!(page_sections(&page(), input, 10, &CancellationToken::new()).is_err());
    }
}
#[test]
fn typed_json_pipe_output_round_trip_and_explicit_text() {
    for (format, terminal) in [(None, false), (Some(OutputFormat::Json), true)] {
        let mut out = Vec::new();
        CliOutput::facet(report(), Some(std::time::Duration::from_secs(1)))
            .emit_to(&mut out, format, terminal)
            .unwrap();
        assert_eq!(out.last(), Some(&b'\n'));
        assert!(!out.contains(&0x1b));
        let decoded: SectionsReport = facet_json::from_slice(&out).unwrap();
        assert_eq!(decoded, report());
    }
    let mut out = Vec::new();
    CliOutput::facet(report(), Some(std::time::Duration::from_secs(1)))
        .emit_to(&mut out, Some(OutputFormat::Text), false)
        .unwrap();
    assert!(!out.contains(&0x1b));
}
#[derive(Debug)]
struct BrokenWriter {
    flush_error: bool,
}
impl Write for BrokenWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.flush_error {
            Ok(buf.len())
        } else {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("synthetic flush failure"))
    }
}
#[test]
fn output_failures_are_observable() {
    for flush_error in [false, true] {
        let err = CliOutput::facet(report(), Some(std::time::Duration::from_secs(1)))
            .emit_to(
                &mut BrokenWriter { flush_error },
                Some(OutputFormat::Json),
                false,
            )
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(if flush_error { "flush" } else { "write" })
        );
    }
}
#[test]
fn actual_cli_schema_help_and_default_pipe_json() {
    let exe = env!("CARGO_BIN_EXE_teamy-docs");
    let help = Command::new(exe)
        .args(["sections", "list", "--help"])
        .output()
        .unwrap();
    assert!(
        help.status.success(),
        "{}",
        String::from_utf8_lossy(&help.stderr)
    );
    assert!(String::from_utf8_lossy(&help.stdout).contains("--book-root"));
    let output = Command::new(exe)
        .args(["sections", "list", "--book-root"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let decoded: SectionsReport = facet_json::from_slice(&output.stdout).unwrap();
    assert_eq!(decoded, report());
}

#[test]
fn rejects_parent_traversal_and_ambiguous_external_parent() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let error = list_sections(
        &fixtures.join("external-parent"),
        DiscoveryLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("external or draft SUMMARY parent")
    );
    let error = list_sections(
        &fixtures.join("traversal"),
        DiscoveryLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("cannot traverse parents"));
}

#[test]
fn singular_alias_preserves_plural_help_and_json() {
    let exe = env!("CARGO_BIN_EXE_teamy-docs");
    let mut outputs = Vec::new();
    for name in ["section", "sections"] {
        let help = Command::new(exe)
            .args([name, "list", "--help"])
            .output()
            .unwrap();
        assert!(
            help.status.success(),
            "{}",
            String::from_utf8_lossy(&help.stderr)
        );
        assert!(String::from_utf8_lossy(&help.stdout).contains("--book-root"));
        let output = Command::new(exe)
            .args([name, "list", "--book-root"])
            .arg(fixture())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let decoded: SectionsReport = facet_json::from_slice(&output.stdout).unwrap();
        assert_eq!(decoded, report());
        outputs.push(output.stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    let help = Command::new(exe).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("aliases: section"));
}

#[test]
fn budget_metadata_never_changes_serialized_result() {
    for format in [OutputFormat::Text, OutputFormat::Json] {
        let mut reference = Vec::new();
        CliOutput::facet(report(), Some(std::time::Duration::from_secs(1)))
            .emit_to(&mut reference, Some(format), false)
            .unwrap();
        for threshold in [None, Some(std::time::Duration::from_secs(30))] {
            let mut bytes = Vec::new();
            CliOutput::facet(report(), threshold)
                .emit_to(&mut bytes, Some(format), false)
                .unwrap();
            assert_eq!(bytes, reference);
        }
    }
}
