use super::list::SectionsListArgs;
use crate::cli::output::CliOutput;
use facet::Facet;
use figue as args;
use teamy_cancellation::CancellationToken;
#[derive(Facet, Debug)]
pub struct SectionsArgs {
    #[facet(args::subcommand)]
    pub command: SectionsCommand,
}
#[derive(Facet, Debug)]
#[repr(u8)]
pub enum SectionsCommand {
    /// List SUMMARY-linked Markdown sections deterministically.
    List(SectionsListArgs),
}
impl SectionsArgs {
    /// # Errors
    /// Returns discovery or cancellation errors.
    pub async fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        match self.command {
            SectionsCommand::List(args) => args.invoke(cancellation).await,
        }
    }
}
