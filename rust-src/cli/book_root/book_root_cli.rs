use super::set::BookRootSetArgs;
use crate::cli::output::CliOutput;
use facet::Facet;
use figue as args;
use teamy_cancellation::CancellationToken;

/// Configure the default local documentation book.
#[derive(Facet, Debug)]
pub struct BookRootArgs {
    #[facet(args::subcommand)]
    pub command: BookRootCommand,
}
#[derive(Facet, Debug)]
#[repr(u8)]
pub enum BookRootCommand {
    /// Save an existing book root for commands run from any directory.
    Set(BookRootSetArgs),
}
impl BookRootArgs {
    /// # Errors
    /// Returns book validation, cancellation, or config-write errors.
    pub async fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        match self.command {
            BookRootCommand::Set(args) => args.invoke(cancellation).await,
        }
    }
}
