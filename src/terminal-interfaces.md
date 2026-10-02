# Build an interactive terminal interface

Start from [teamy-rust-cli](rust-cli-template.md), keep its typed command hierarchy, and add Ratatui when the task benefits from a persistent screen. Reuse the service behind the CLI; entering a prompt should not spawn a new executable and reload the model for every turn.

Our prior art covers three different interfaces. Choose the one that solves the problem before choosing widgets.

| Interaction | Existing implementation | What to reuse |
| --- | --- | --- |
| Pick values, then return them to a pipeline | Cloud-Terrastodon `pick` | Searchable choices, async candidate loading, stderr rendering and selected values on stdout. |
| Explore objects while commands run | Cloud-Terrastodon `ratatui` | A persistent view, async engine, nested terminal ownership and structured log presentation. |
| Enter several requests against one loaded model | teamy-tts `interactive` | Load once, accept successive lines, check cancellation while waiting for input, and retain explicit output-file policy. |

These references were inspected on 2 October 2026. The source links below identify public snapshots; source inspection is not a claim that every interface was replayed in this documentation update. Interactive terminal behavior also depends on the terminal host and redirection.

## Use Ratatui with the existing CLI

Ratatui provides widgets, layout and a terminal backend abstraction. The application owns its state, input handling and work. The [backend documentation](https://ratatui.rs/concepts/backends/) describes raw mode, alternate screens and the cross-platform Crossterm backend.

Cloud-Terrastodon's [workspace manifest](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/Cargo.toml) requests Ratatui `0.30.1`; its [lockfile](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/Cargo.lock) resolves `0.30.2` with Crossterm `0.29.0`. Ratatui's [0.30.2 release notes](https://ratatui.rs/highlights/v0302/) describe a fix for widgets losing `Send` and `Sync` in 0.30.1. That makes the resolved version relevant to async and threaded designs.

Keep the dependency graph on one compatible Crossterm version. Different versions have separate event and raw-mode state. Inspect the consuming project's lockfile; a local reference clone's version does not determine the published crate being compiled. Pulling a source clone is a separate operation from selecting a reviewed Cargo dependency. See the [version compatibility guidance](https://ratatui.rs/concepts/backends/#crossterm-version-compatibility).

The UI command still belongs in the typed CLI schema. Cloud-Terrastodon dispatches its [Ratatui command](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/entrypoint/src/cli/command/ratatui/mod.rs) into a separate UI module. Follow our [argument parsing](cli-parsing.md) and subcommand module conventions instead of constructing a second parser inside the screen.

## Decide which stream owns the screen

Stdout and stderr are separate streams, but they usually render in the same terminal. Moving logs to stderr preserves machine stdout; it does not stop those logs from overwriting a stdout TUI.

| Command contract | Screen destination | Result destination |
| --- | --- | --- |
| Pipeline picker | Stderr, if it is an interactive terminal | Stdout after the picker restores the terminal. |
| Full-screen interactive application | Stdout is appropriate when the command owns it | The screen itself; define any export separately. |
| Batch JSON command | No TUI escape sequences in stdout | One documented JSON or record stream on stdout. |

Cloud-Terrastodon's picker uses `Terminal<CrosstermBackend<BufWriter<Stderr>>>`. Its [`pick stdin` command](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/entrypoint/src/cli/command/pick/pick_stdin_command_cli.rs) reads candidates first, runs the picker, then writes selected lines or JSON to stdout. The full [object browser](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/ui_ratatui/src/object_browser/terminal.rs) uses Ratatui's default stdout terminal. Both need [terminal ownership and buffered diagnostics](terminal-ownership.md).

For example, the published picker accepts line candidates:

```powershell
"native", "onnx" | cloud_terrastodon pick stdin --input-format lines --single
```

After accepting `onnx`, stdout contains that selected line. The picker controls the screen through stderr while it is open. This is a human selection interface; it is not Julia model inference. A piped stdin and an interactive keyboard are different input requirements: qualify the intended host and platform before promising that combination universally.

## Keep the loaded runtime between turns

teamy-tts [loads its runtime before its input loop](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/src/cli/interactive.rs). A dedicated thread reads stdin into a Tokio channel. Terminal prompts go to stderr; synthesis and playback are sequential, and `--output-dir` opts into persistent WAV files. Cancellation can end the async wait for input. Its stdin channel is unbounded, so this reference needs an explicit capacity policy before reuse with an unattended or long-running producer.

Ollama offers a different reference. Its [interactive reader](https://github.com/ollama/ollama/blob/dd1d4e99e7e8475d1669f566bb5c0ae30db419f1/cmd/interactive.go) handles history, multiline input, bracketed paste and slash commands. The CLI sends requests to a service whose [scheduler](https://github.com/ollama/ollama/blob/dd1d4e99e7e8475d1669f566bb5c0ae30db419f1/server/sched.go) can reuse a loaded runner and expire it later. The useful lesson is runtime lifetime, not just a `>>>` prompt.

Keeping a service alive can avoid repeated tokenizer loading, weight identity checks and GPU session preparation. It does not automatically reuse conversation state or KV cache. Define those lifetimes separately. [Local text generation](local-text-generation.md) records the measured cold and resident costs in our own backend; [async terminal work](async-terminal-ui.md) explains how to keep that backend from blocking input and redraws.

## Learn navigation from K9s

[K9s](https://k9scli.io/) is an existing Kubernetes terminal application to study for resource navigation, filtering, live status and contextual commands. It is a curated interaction reference already valued in our workflow; this chapter does not claim a Rust dependency or a verified cluster session.

Its [command guide](https://k9scli.io/topics/commands/) documents `?` for help, `:` for resource navigation, `/` for filtering and `k9s --readonly` for disabling modification commands. Those affordances help explain how a dense terminal application can remain discoverable. Read-only UI mode is an application policy; Kubernetes permissions remain a separate access boundary.

For embedding an actual child terminal rather than drawing a TUI, use [terminal integration and PTYs](terminal-integration.md).
