use crate::book_root::save_book_root;
use crate::cli::output::CliOutput;
use facet::Facet;
use figue as args;
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;

/// Save a local book root without changing the book.
#[derive(Facet, Debug)]
pub struct BookRootSetArgs {
    /// Existing directory containing book.toml; relative paths resolve from this CWD.
    #[facet(args::named)]
    pub path: PathBuf,
}
impl BookRootSetArgs {
    /// # Errors
    /// Returns book validation, cancellation, or config-write errors.
    #[expect(
        clippy::unused_async,
        reason = "template commands share an async invoke contract"
    )]
    pub async fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(
            save_book_root(&self.path, &cancellation)?,
            Some(std::time::Duration::from_secs(1)),
        ))
    }
}
