use crate::book_root::resolve_book_root;
use crate::cli::output::CliOutput;
use crate::sections::{DiscoveryLimits, list_sections};
use facet::Facet;
use figue as args;
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;
/// List page preambles and Markdown headings in book order.
#[derive(Facet, Debug)]
#[facet(rename_all = "kebab-case")]
pub struct SectionsListArgs {
    /// Override the default book with this directory containing book.toml.
    #[facet(args::named)]
    pub book_root: Option<PathBuf>,
    /// Maximum UTF-8 bytes in a page, config, or SUMMARY.
    #[facet(args::named, default = 2_097_152)]
    pub max_page_bytes: u64,
    /// Maximum total input bytes.
    #[facet(args::named, default = 33_554_432)]
    pub max_total_bytes: u64,
    /// Maximum distinct linked pages.
    #[facet(args::named, default = 1_024)]
    pub max_pages: usize,
    /// Maximum records, including preambles.
    #[facet(args::named, default = 10_000)]
    pub max_sections: usize,
}
impl SectionsListArgs {
    fn completion_budget(&self) -> std::time::Duration {
        let pages = u64::try_from(self.max_pages).unwrap_or(u64::MAX);
        let sections = u64::try_from(self.max_sections).unwrap_or(u64::MAX);
        let bytes = self.max_total_bytes.div_ceil(1_048_576);
        std::time::Duration::from_millis(
            1000_u64
                .saturating_add(bytes.saturating_mul(100))
                .saturating_add(pages.saturating_mul(5))
                .saturating_add(sections),
        )
    }
    /// # Errors
    /// Returns discovery or cancellation errors.
    #[expect(
        clippy::unused_async,
        reason = "template commands share an async invoke contract"
    )]
    pub async fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        // Budget the declared bounded discovery workload, including output rendering.
        let threshold = Some(self.completion_budget());
        let book_root = resolve_book_root(self.book_root.as_deref())?;
        let report = list_sections(
            &book_root,
            DiscoveryLimits {
                max_page_bytes: self.max_page_bytes,
                max_total_bytes: self.max_total_bytes,
                max_pages: self.max_pages,
                max_sections: self.max_sections,
            },
            &cancellation,
        )?;
        tracing::debug!(
            pages_discovered = report.pages.len(),
            sections_discovered = report.sections.len(),
            "Section discovery finished"
        );
        Ok(CliOutput::facet(report, threshold))
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn discovery_budget_scales_with_limits_and_saturates() {
        let mut args = SectionsListArgs {
            book_root: None,
            max_page_bytes: 2_097_152,
            max_total_bytes: 33_554_432,
            max_pages: 1024,
            max_sections: 10_000,
        };
        let baseline = args.completion_budget();
        assert_eq!(baseline, std::time::Duration::from_millis(19_320));
        args.max_total_bytes *= 2;
        assert!(args.completion_budget() > baseline);
        args.max_pages = usize::MAX;
        args.max_sections = usize::MAX;
        assert!(args.completion_budget() >= baseline);
    }
}
