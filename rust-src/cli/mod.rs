pub mod book_root;
pub mod global_args;
pub mod log_format;
pub mod output;
pub mod sections;

use self::book_root::BookRootArgs;
use self::output::CliOutput;
use self::sections::SectionsArgs;
use facet::Facet;
use figue::{self as args, FigueBuiltins};
use teamy_cancellation::CancellationToken;

/// Discover public mdBook sections without executing examples.
#[derive(Facet, Debug)]
pub struct Cli {
    #[facet(flatten, default)]
    pub global_args: GlobalArgs,
    #[facet(flatten)]
    pub builtins: FigueBuiltins,
    #[facet(args::subcommand)]
    pub command: Command,
}
pub use global_args::GlobalArgs;
#[derive(Facet, Debug)]
#[repr(u8)]
pub enum Command {
    /// Discover mdBook sections.
    #[facet(args::alias = "section")]
    Sections(SectionsArgs),
    /// Configure the default local book.
    BookRoot(BookRootArgs),
}
impl Cli {
    /// # Errors
    /// Returns runtime or discovery errors.
    pub fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;
        runtime.block_on(self.command.invoke(cancellation))
    }
}
impl Command {
    /// # Errors
    /// Returns discovery or cancellation errors.
    pub async fn invoke(self, cancellation: CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        match self {
            Self::Sections(args) => args.invoke(cancellation).await,
            Self::BookRoot(args) => args.invoke(cancellation).await,
        }
    }
}
