use eyre::Context;
use facet::Facet;
use facet_pretty::{ColorMode, PrettyPrinter};
use std::io::Write;
#[derive(Facet, Clone, Copy, Debug, PartialEq, Eq)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum OutputFormat {
    Text,
    Json,
}
pub struct CliOutput {
    value: Box<dyn OutputValue>,
    /// Completion budget including output; None explicitly exempts interactive or unbounded work.
    pub elapsed_duration_warn_threshold: Option<std::time::Duration>,
}
impl std::fmt::Debug for CliOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliOutput")
            .field(
                "elapsed_duration_warn_threshold",
                &self.elapsed_duration_warn_threshold,
            )
            .finish_non_exhaustive()
    }
}
trait OutputValue {
    fn render(&self, format: OutputFormat, terminal: bool) -> eyre::Result<String>;
}
struct FacetOutput<T>(T);
impl CliOutput {
    #[must_use]
    pub fn facet<T: Facet<'static> + 'static>(
        value: T,
        elapsed_duration_warn_threshold: Option<std::time::Duration>,
    ) -> Self {
        Self {
            value: Box::new(FacetOutput(value)),
            elapsed_duration_warn_threshold,
        }
    }
    /// # Errors
    /// Returns serialization, write, or flush errors.
    pub fn emit_to<W: Write + ?Sized>(
        self,
        writer: &mut W,
        requested: Option<OutputFormat>,
        terminal: bool,
    ) -> eyre::Result<()> {
        let format = requested.unwrap_or(if terminal {
            OutputFormat::Text
        } else {
            OutputFormat::Json
        });
        let rendered = self.value.render(format, terminal)?;
        writer
            .write_all(rendered.as_bytes())
            .wrap_err("failed to write command output")?;
        if !rendered.ends_with('\n') {
            writer
                .write_all(b"\n")
                .wrap_err("failed to terminate command output")?;
        }
        writer.flush().wrap_err("failed to flush command output")?;
        Ok(())
    }
}
impl<T: Facet<'static> + 'static> OutputValue for FacetOutput<T> {
    fn render(&self, format: OutputFormat, terminal: bool) -> eyre::Result<String> {
        match format {
            OutputFormat::Json => facet_json::to_string_pretty(&self.0)
                .wrap_err("failed to serialize command output as JSON"),
            OutputFormat::Text => Ok(PrettyPrinter::new()
                .with_colors(if terminal {
                    ColorMode::Always
                } else {
                    ColorMode::Never
                })
                .format(&self.0)),
        }
    }
}
