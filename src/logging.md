# Logging with tracing

Use the logging setup provided by [teamy-rust-cli](rust-cli-template.md). Emit structured events and spans through `tracing`; let `tracing-subscriber` filter them and send them to the terminal, a file or a profiler. Keep command results on stdout and operational communication on stderr, even when the result format is JSON.

The template baseline below is public revision [`7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs), checked on 1 October 2026. Later local changes are identified explicitly. The reference projects demonstrate capabilities to reuse when needed; copying the template alone does not add their buffering or cross-process behavior.

## Events, spans and destinations

An [event](glossary.md#event) records something that happened. A [span](glossary.md#span) gives an operation a name, duration and context. A [subscriber](glossary.md#subscriber) receives that instrumentation; layers give different destinations their own formatting and filtering.

Use fields that explain work without requiring someone to parse a sentence:

```rust,ignore
tracing::info!(completed, total, "Repository review progressed");
tracing::warn!(action_id, "Destination is occupied");
```

These are instrumentation examples; the variables belong to the command using them. Put a completed finding in the [typed command result](output-shape.md) as well when callers need to act on it. Logs are useful operational evidence, not the result schema.

`#[tracing::instrument]` can create a function span, but it records arguments by default. Choose `skip(...)`, `skip_all` and explicit fields deliberately. The template has no automatic redaction: logged arguments and source metadata can disclose paths or other data. See the [instrumentation API](https://docs.rs/tracing/latest/tracing/attr.instrument.html).

## Filters and the log-level alias

The template's shared field is `GlobalArgs.log_filter`. `default_log_filter()` chooses these directives:

| Invocation | Terminal and NDJSON filter |
| --- | --- |
| No explicit filter | `warn,teamy_rust_cli=info` |
| `--debug` | `warn,teamy_rust_cli=debug` |
| `--log-filter "error"` | `error` |
| `--log-filter "warn,teamy_rust_cli=trace"` | The supplied full directive string. |

An explicit filter conflicts with `--debug`. It replaces the default directives. When creating a new project, update the default target to that crate's Rust identifier; package-name hyphens normally become underscores in tracing targets. Filters can select levels, targets, spans and fields. See [EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html).

On 1 October 2026, the local template gained `--log-level` as an alias for `--log-filter`, backed by the same field:

```rust,ignore
#[facet(args::named, args::alias = "log-level")]
pub log_filter: Option<String>,
```

It accepts a plain level or the same full directive string. It is an alias, not a second setting with a separate precedence. Four parser regressions verify both spellings, nested placement, help and the existing debug conflict. This addition is local and absent from the baseline permalink above; an installed binary or older template snapshot may not have it. Root help lists the alias; the pinned local parser omits global flags from leaf-command help.

From a reviewed template checkout, these commands exercise existing read-only command behavior; the third also creates or truncates its explicitly named log file:

```powershell
cargo run -- --log-filter "warn,teamy_rust_cli=debug" home show
# Requires the local alias addition:
cargo run -- --log-level info home show
cargo run -- --log-filter debug --log-file ./review.ndjson `
  --output-format json home show
```

`RUST_LOG` is currently ignored by this template despite comments claiming otherwise. Its parser configures CLI input only, and logging calls `EnvFilter::builder().parse(...)` on an explicit or default string. Reading an environment variable requires a separate environment-reading path. Also, `--debug` does not set `RUST_BACKTRACE`. Grounding: [argument fields](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/global_args.rs), [entry point](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/lib.rs), and [filter construction](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.Builder.html).

## Terminal logs, NDJSON and Tracy

| Destination | Current template behavior |
| --- | --- |
| Stderr | Pretty text with target names and no timestamps. Source file and line are included in debug builds. |
| `--log-file` | A separate NDJSON stream, with one JSON event per line and source file/line fields. Omitted means no file layer. |
| Tracy | An optional profiling layer with its own fixed filter: trace generally, warnings for selected compiler/runtime targets. It is enabled by a Cargo feature. |

Terminal and file layers use the same selected filter. A quiet terminal filter does not reduce Tracy's independent collection. The locally inspected template also excludes `tracy.frame_mark` metadata from terminal and file layers; that exclusion is a local change absent from the baseline snapshot. Grounding: `tracy_log_filter_directives()` and `init_logging()` in [logging initialization](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs).

`--log-file` treats an existing directory as a directory and generates `log_<local timestamp>.ndjson` within it. Every other argument is treated as a file path; parent directories are created. `File::create` truncates an existing file. Timestamp names have seconds precision, so two runs can select the same name. The file writer clones a file handle for each writer request; its mutex covers handle cloning, not subsequent writes. There is no application-level buffered writer, worker queue, rotation or explicit durability sync in this setup.

File-creation errors propagate through `eyre`. If installing the global subscriber fails, the template instead prints a diagnostic and returns success from initialization; the requested layers may not be active. The log file has already been opened by that point. A CLI promising reliable log delivery must account for this behavior.

## Buffering means several different things

Choose the mechanism for the behavior required. Retaining logs for a screen, batching disk writes and delivering logs from another process solve different problems.

| Mechanism | What it provides | Reference or status |
| --- | --- | --- |
| Memory replay | Keep prior messages so an interactive view can show them later. | Piing and MFT examples below. |
| Buffered file writes | Batch bytes before sending them to a file; require a flush policy. | Not present in the template's file writer. |
| Background writer queue | Move output writes to a worker, with a defined capacity and overflow policy. | `tracing-appender` is an available option, not a template dependency. |
| Bounded event history and live broadcast | Give new subscribers recent context and deliver ongoing events. | MFT's daemon log hub. |

For a background writer, [`tracing-appender::NonBlockingBuilder`](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.NonBlockingBuilder.html) exposes queue capacity and a loss policy. Its default is lossy: full queues drop logs. Non-lossy mode blocks producers for capacity. Retain a named [`WorkerGuard`](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html) until work completes so normal shutdown can drain queued logs. Forced exit can bypass this cleanup; coordinate the policy with [cancellation](cancellation.md#choose-the-interruption-policy).

The following reference implementations show how our projects already use replay and transport.

## Learn from Piing and MFT

Piing writes terminal logs both to stderr and to `teamy_windows::log::LOG_BUFFER`. Its interactive `ShowLogs` action replays buffered text. This preserves access to messages while the interface is showing another view. Piing also has a separate NDJSON file layer. Grounding: [Piing's logger at `d2c5065`](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/logging.rs#L71-L127) and [tray replay](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/tray/window_proc.rs#L172-L175).

The published [`teamy-windows` 0.7.0 buffer](https://docs.rs/crate/teamy-windows/0.7.0/source/src/log/buffer_sink.rs) is an unbounded `Arc<Mutex<Vec<u8>>>`. It appends bytes, holds its lock during replay and implements `flush()` as a no-op. Piing calls stdout flush after replay but ignores that error. Reuse the replay pattern with an explicit retention policy for a long-running application.

MFT implements a separate in-memory byte buffer as `Arc<Mutex<Vec<u8>>>`. Its writer appends bytes and its `flush` is a no-op. That buffer is unbounded; it is not a ring buffer or an async disk queue. Grounding: [MFT buffer sink](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/windows_utils/log/buffer_sink.rs#L24-L66).

MFT also supports two distinct ways to keep process logs visible in the caller's terminal. Elevation reconnects the new process to the original console. Daemon logging forwards structured events over RPC. The next section explains their different roles.

## Elevation and daemon log forwarding

For an elevated relaunch, MFT's original client calls `ShellExecuteExW` with the `runas` verb and includes a hidden `--console-pid` argument containing the original client's process ID. The elevated child calls `AttachConsole`, rebinds stdout and stderr to `CONOUT$`, and attempts to rebind stdin to `CONIN$`. Successful attachment keeps its later logs visible through the original console and terminal host. The original client waits for the child and propagates its exit status.

```text
Original terminal / PTY
  |
  +-- MFT client -- runas, --console-pid=<client PID> --> elevated MFT child
  |       |                                                  |
  |       +-- waits for child                                 +-- AttachConsole
  |                                                          +-- rebind streams
  +<---------------------------- child's stderr logs --------+
```

This is console attachment, not a parent reading a captured stderr pipe. The implementation points are [invocation arguments](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/windows_utils/invocation/same_invocation_same_console.rs#L10-L30), [UAC relaunch](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/windows_utils/elevation/run_as_admin.rs#L23-L40), [wait and exit propagation](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/windows_utils/elevation/ensure_elevated.rs#L8-L19) and [console attachment](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/windows_utils/console/attach_to_existing.rs#L14-L25). Logging initializes before reattachment, so early messages precede the handoff. Rebinding streams does not promise preservation of shell redirects, a machine-readable stdout pipe or every PTY configuration.

For the daemon, `DaemonLogHub` keeps the last 2,048 typed events in a deque and uses a bounded Tokio broadcast channel for live subscribers. Correlation IDs select a request's events. The Vox RPC connection runs over Windows named pipes. The client reconstructs received events and emits them through its own tracing subscriber, labelled `[daemon]`, so its existing writers deliver them to the terminal. Grounding: [daemon log hub](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/machine/daemon_log.rs#L24-L93), [forwarder](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/machine/daemon_log.rs#L411-L511), and [client emission and drain](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/machine/daemon_log.rs#L536-L750).

```text
Daemon events -> recent history + live broadcast -> typed Vox stream
                                                        |
                                                     MFT client
                                                        |
                                              tracing subscriber
                                                        |
                                               stderr -> terminal
```

The pinned stream transport uses Tokio `BufWriter` and bounded queues of 128 frames. It batches queued frames before flushing; stream-level sends await queue capacity and a local flush acknowledgement. These bounds count messages, not payload bytes, and a local flush does not prove the client received or rendered the log. See [the stream worker](https://github.com/TeamDman/facet/blob/5fd9cfaa46b4babc1f79d10d714600e710c28c2f/vox/rust/vox-stream/src/lib.rs#L237-L312) and [send acknowledgement](https://github.com/TeamDman/facet/blob/5fd9cfaa46b4babc1f79d10d714600e710c28c2f/vox/rust/vox-stream/src/lib.rs#L343-L376).

Broadcast publishing does not wait for subscribers. A slow forwarder can lag, lose events and emit a warning. Forwarder shutdown waits up to two seconds without guaranteeing complete delivery. Recent history is finite replay, not a durable audit log.

Keep stdout's result contract intact when adopting either pattern. Reuse these implementations when their process and terminal assumptions fit the new CLI; document capacity, loss and shutdown behavior alongside the flags that expose them.
