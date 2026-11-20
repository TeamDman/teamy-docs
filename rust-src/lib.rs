mod entrypoint;
mod failure_record;
pub use entrypoint::{default_entrypoint, entrypoint_with_failure_message};
// Reusable, bounded mdBook section discovery and a composable typed CLI.
pub mod book_root;
pub mod cli;
pub mod elapsed_duration;
pub mod logging_init;
pub mod sections;

use std::io::{self, IsTerminal};
use teamy_cancellation::CtrlCHandler;

/// Standalone startup owns cancellation, logging, runtime, and stdout.
///
/// # Errors
/// Returns startup, parsing, discovery, cancellation, or output errors.
///
/// # Panics
/// Panics if the statically defined CLI schema is invalid.
pub fn main() -> eyre::Result<()> {
    // Binary entrypoint owns safe diagnostic hooks; library callers own their hooks.
    let cancellation_token = CtrlCHandler::default().install()?;
    let cli: figue::DriverOutcome<cli::Cli> = figue::Driver::new(
        figue::builder::<cli::Cli>()
            .expect("valid CLI schema")
            .cli(|cli| cli.args_os(std::env::args_os().skip(1)).strict())
            .help(|help| help.version(env!("CARGO_PKG_VERSION")))
            .build(),
    )
    .run();
    let cli = entrypoint::finish_parse(cli)?;
    let _stop_after = cli
        .global_args
        .stop_after
        .start_stop_after_duration_thread(cancellation_token.clone())?;
    logging_init::init_logging(&cli.global_args, cancellation_token.clone())?;
    let command_name = match &cli.command {
        cli::Command::Sections(_) => "sections",
        cli::Command::BookRoot(_) => "book-root",
    };
    tracing::debug!(command = command_name, "Dispatching command");
    let result = (|| -> eyre::Result<()> {
        let _command_span = tracing::info_span!("command", command = command_name).entered();
        let requested_format = cli.global_args.output_format;
        let started = std::time::Instant::now();
        let output = cli.invoke(cancellation_token.clone())?;
        let threshold = output.elapsed_duration_warn_threshold;
        cancellation_token.bail_if_cancelled()?;
        let stdout = io::stdout();
        let is_terminal = stdout.is_terminal();
        let emitted = output.emit_to(&mut stdout.lock(), requested_format, is_terminal);
        elapsed_duration::warn_if_exceeded(command_name, started.elapsed(), threshold);
        emitted?;
        cancellation_token.bail_if_cancelled()?;
        Ok(())
    })();
    tracing::debug!(
        command = command_name,
        success = result.is_ok(),
        "Command finished"
    );
    result
}

/// Binary startup with safe, actionable book-selection categories.
#[must_use]
pub fn entrypoint() -> std::process::ExitCode {
    entrypoint_with_failure_message(book_failure_message)
}

fn book_failure_message(error: &eyre::Report) -> &'static str {
    let text = error.to_string();
    for message in [
        "invalid --book-root",
        "invalid TEAMY_DOCS_BOOK_ROOT",
        "no default book is configured; use --book-root or book-root set",
        "saved book root is invalid",
        "saved book root must be an absolute path",
        "saved book root is not UTF-8",
        "saved book root exceeds 64 KiB",
        "TEAMY_DOCS_CONFIG_DIR must be an absolute, nonempty path",
        "book root must contain book.toml",
    ] {
        let category = message.split(';').next().expect("fixed category");
        if text.contains(category) {
            return message;
        }
    }
    entrypoint::default_failure_message(error)
}
