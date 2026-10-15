# Logging with tracing

Use the logging setup provided by [teamy-rust-cli](rust-cli-template.md). Emit structured events and spans through `tracing`; let `tracing-subscriber` filter them and send them to the terminal, a file or a profiler. Keep command results on stdout and operational communication on stderr, even when the result format is JSON.

[Our Rust code policy](code-standards.md) rejects plain print macros in adopted packages. Diagnostics belong to tracing; results use a supplied `Write` sink. Cargo instructions and a logging-bootstrap failure have narrowly scoped exceptions. Terminal backends remain valid writers under their terminal owner.

For startup delays, CPU hotspots and completed GPU-work timing, read [performance analysis](performance-analysis.md). It connects these spans to the capture harness and explains what its CPU summaries can establish.

[TeamDman's earlier Windows terminal-colour answer](https://stackoverflow.com/a/78741674) enables virtual terminal processing on `STD_OUTPUT_HANDLE`. It is historical prior art, not proof that stderr has the same mode. Check terminal detection and ANSI support separately for each writer; redirected logs should follow their declared output format. The [authored-answer index](stackoverflow-prior-art.md) records the answer's inspection status.

The template baseline below is public revision [`7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs), checked on 1 October 2026. Later local changes are identified explicitly. The reference projects demonstrate capabilities to reuse when needed; copying the template alone does not add their buffering or cross-process behavior.

## Local independent-sink rollout

The 9 October local template implements independently filtered/formatted stderr and optional files. This behavior is validated locally and is not contained in the historical public revision cited below. Email, Discord, embedding, LLM and the docs CLI are adopting the same contract; installed binaries require separate activation. Source evidence is the template's `src/logging_init.rs`, `src/cli/global_args.rs`, `src/lib.rs`, `docs/logging-progress.md` and `tests/log_sinks.rs`. These working-tree changes have no new public revision permalink yet.

| Setting | Behavior |
| --- | --- |
| stderr filter | Explicit --log-filter/--log-level, then --debug, then RUST_LOG, then INFO. |
| file filter | --log-file-filter, otherwise DEBUG; independent of stderr and RUST_LOG. |
| formats | --log-format defaults to text; --log-file-format defaults to ndjson. |
| destination | --log-file is an explicit file path; parent directories are created and existing files are appended. |

Each formatter owns its filter; no global INFO filter discards DEBUG before the file sink sees it. Full EnvFilter directives are accepted with regex matching disabled. The file mutex covers each complete formatted record. Invalid filters are rejected before creating a log file; subscriber installation failures propagate. There is no rotation, background queue or durability sync. Binary failures use safe generic messages, including early NDJSON startup errors; producer fields are not automatically redacted. Successful help/version requests keep parser presentation and do not initialize a log file.

Emit direct tracing events at meaningful operation call sites. Use DEBUG for routine dispatch and completion, INFO for useful updates, and WARN/ERROR for failures that need attention. Choose safe fields that describe real work, such as items_processed, bytes_written, elapsed_ms or phase. There is no mandatory progress schema or universal run/task/state envelope. Counting a command from 0 to 1 adds no information about its work and is no longer part of the template.

Command runtime expectations are separate from progress and estimates. Actual counts can support measured rates or estimates; a command duration threshold cannot supply an ETA. Read [runtime warnings](performance-analysis.md#make-slow-commands-warn-during-normal-use) for that separate contract.

Automatic span-close receipts belong in the file sink, including when stderr uses NDJSON. Ordinary terminal events retain their span context. The local LLM follows the same policy for direct commands, daemon replay and interactive workers: DEBUG dispatch/completion events replace its former INFO command counter. Worker timing records have an explicit private transport marker, stripped before replay; the parent retains their kind as `worker_span_close`. A separate source-target header preserves `worker_target` across long records without mistaking formatted span context for a target. Daemon receipts retain `daemon_kind=SpanClose`. Terminal sinks exclude those receipts while independently filtered file sinks preserve them for profiling. Useful INFO updates, warnings and diagnostic-loss summaries still reach the terminal and its TUI buffer.

LLM source evidence is `crates/teamy_llm_cli/src/{lib,logging}.rs`, `logging/source_sink_filter.rs`, `logging/source_aware.rs`, `interactive/engine.rs` and `tests/log_sinks.rs`. Remote source capture still applies the request filter and daemon capture ceiling before transmission. Local sinks cannot recover events already dropped at their source.

The following baseline sections describe the earlier public implementation; the independent-sink behavior above supersedes their shared filters, debug conflict, directory naming and truncating writer for adopted local checkouts.

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
| `--log-file` | A separate NDJSON stream, with one JSON event per line and source file/line fields. The local timing update also records span-close durations. Omitted means no file layer. |
| Tracy | An optional profiling layer with its own fixed filter: trace generally, warnings for selected compiler/runtime targets. It is enabled by a Cargo feature. |

Terminal and file layers use the same selected filter. A quiet terminal filter does not reduce Tracy's independent collection. The locally inspected template also excludes `tracy.frame_mark` metadata from terminal and file layers; that exclusion is a local change absent from the baseline snapshot. Grounding: `tracy_log_filter_directives()` and `init_logging()` in [logging initialization](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs).

For slow commands, start with the [performance methodology](performance-analysis.md). It connects tracing, subscriber layers, NDJSON, Tracy's capture/viewer and teamy-profiler, and shows how to [warn on runtime expectations](performance-analysis.md#make-slow-commands-warn-during-normal-use). The 4 October local template update enables `FmtSpan::CLOSE` on the NDJSON formatter only; stderr retains its existing event formatting. Ordinary span timing requires a filter that enables the span's level. The matching real-CLI test is `tests/log_span_timings.rs`; older template snapshots need that adoption explicitly.

`--log-file` treats an existing directory as a directory and generates `log_<local timestamp>.ndjson` within it. Every other argument is treated as a file path; parent directories are created. `File::create` truncates an existing file. Timestamp names have seconds precision, so two runs can select the same name. The file writer clones a file handle for each writer request; its mutex covers handle cloning, not subsequent writes. There is no application-level buffered writer, worker queue, rotation or explicit durability sync in this setup.

File-creation errors propagate through `eyre`. If installing the global subscriber fails, the template instead prints a diagnostic and returns success from initialization; the requested layers may not be active. The log file has already been opened by that point. A CLI promising reliable log delivery must account for this behavior.

## Keep runtime diagnostics in the logging pipeline

The local `teamy-llm` CUDA runtime routes upload, graph and launch diagnostics through the targets `teamy_llm_makepad_gguf::cuda::{upload,graph,launch}`. Routine profiling is debug-level; fallback and cleanup failures use warning or error levels. The CLI's `teamy_llm` family filter includes these targets. For direct `prompt` or `serve`, `--log-level warn` suppresses successful profiling while retaining warnings and errors; `--debug --log-file ./startup.ndjson` captures the structured CUDA fields alongside service spans.

Rust runtime events use `tracing` directly, while a native host callback bridges launch diagnostics into the same subscriber. Direct stderr prints bypass filters, the NDJSON layer and terminal ownership. Routing these diagnostics through the pipeline lets the application control their delivery. This bridge is specific to the vendored runtime; it does not intercept arbitrary native output.

Interactive inference relays formatted child stderr through a `worker_log` field. In the newer local relay, a recognized RFC3339 timestamp followed by a tracing level preserves that level. Those records use the reserved `teamy_llm::worker::filtered` target and pass the parent's output layers because the child's tracing filter already admitted them. Long-line chunks retain their first chunk's route and level. Unrecognized native or bootstrap stderr uses the ordinary `teamy_llm::worker` target at info level and remains subject to the parent's filter.

[Terminal ownership](terminal-ownership.md) buffers human delivery while NDJSON records the wrapper. Local worker CUDA fields remain formatted text under `worker_log`, rather than separate parent JSON fields. A target-specific source filter does not require adding a second worker-target directive to retain recognized tracing records. Daemon events follow the newer typed relay described below and retain their metadata through the parent.

Grounding in the matching service checkout: `vendor/makepad-ai-llm/src/cuda_exec/initial_upload.rs`, `real.rs`, `vendor/makepad-ai-cuda/src/launch_diagnostics.rs`, `crates/teamy_llm_cli/src/logging.rs` and `interactive/engine.rs`. Keep [performance measurements](performance-analysis.md) bound to the executable that produced them; changing diagnostic delivery does not requalify an earlier timing result.

## Forward one request's diagnostics

The local `teamy-llm --daemon` implementation uses MFT's typed-event and client-replay pattern with a private queue for each inference request. [Resident inference connections](resident-inference.md) shows the command surface. It covers prompt generation, finite-choice decisions and interactive clients.

`RequestTraceLayer` inherits request identity through tracing span extensions. It captures only descendants of that request root. Blocking inference jobs carry their request span and subscriber dispatch; the logging layer remains installed once for the whole server. There is no per-request subscriber replacement and no global log history to send to unrelated clients.

Each request uses its invoking client's effective source filter. Server console and file filters apply only to those output layers. Direct client replay uses the original event level and preserves original metadata in `daemon_*` fields. Its reserved replay target is allowed by the client output layers because source filtering already ran on the server. A target-specific directive such as `teamy_llm_service=debug` therefore survives replay through `teamy_llm::daemon_remote`.

An interactive relay child forwards each owned source event as a bounded `WorkerEvent::DaemonLog` reply on its private stdout protocol. The outer reply ID identifies the active worker action; the inner event retains the daemon's request ID. They belong to different processes and must not be equated. The parent validates the active action ID, consumes diagnostic frames before passing result events to the TUI or script, and replays the original record through `remote_logging::replay`. Its NDJSON therefore retains `daemon_*` fields, rather than embedding the daemon envelope in `worker_log`.

The child does not also replay these events into stderr. Its own local tracing, native and bootstrap stderr still use the compact worker-text path described above. Only the reserved daemon-replay and filtered-worker targets receive the source-filter exemption; arbitrary native stderr does not.

The owned version 1 event contains timestamp, level, target, source location, request identity, message, fields and span stack. Span-close records add busy and idle nanoseconds. The wire event bounds its aggregate text to 16 KiB, fields to 16 and spans to 8. Its 128-event queue uses nonblocking sends; dropped and truncated diagnostics produce a count summary. Capture accepts a filter of at most 4 KiB and disables regular-expression matching for field-value directives.

Diagnostic transport has separate credit from model result events. Cancellation does not need a log receiver to make progress. A bounded forwarder may abandon undelivered logs rather than delaying inference indefinitely. Keep this separate from a durable audit requirement.

Grounding in the matching local service checkout: `crates/teamy_llm_core/src/remote_log.rs`, `crates/teamy_llm_service/src/{request_logs,vox_api}.rs`, `crates/teamy_llm_cli/src/{logging,remote_logging}.rs` and `crates/teamy_llm_cli/src/cli/{daemon_client,daemon_drain}.rs`. The daemon client's callback validates each source event before delivery. The interactive relay lives in `crates/teamy_llm_cli/src/interactive/{worker,protocol,engine}.rs`. Reply and diagnostic closure are drained together, including cancellation; completing a numerical reply does not silently discard later queued logs. [MFT's daemon implementation](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/machine/daemon_log.rs) is the prior art. MFT retains recent global history; this implementation deliberately has only live request queues.

## Present source records at the client

Forwarded diagnostics should explain the operation that produced them. A receiver-side timestamp and `teamy_llm::daemon_remote` target describe transport, while the source timestamp and target describe inference. The newer local `teamy-llm` formatter uses the source record for human presentation and retains the transport fields for structured capture.

The stderr layer uses a custom `tracing_subscriber::fmt::FormatEvent` implementation. Client events carry a `[client]` label, and the server's own events carry `[daemon]`. Typed remote events carry `[daemon]`, their source UTC timestamp, original target, message and event fields. Request identity stays visible. Source file and line appear on a separate line, followed by the remote span stack. Span-close busy and idle nanoseconds become readable durations. The human output does not print nested `daemon_fields` or `daemon_spans` JSON.

This changes formatting at the subscriber layer, rather than replacing tracing events with direct stderr writes. Replay still uses the reserved target and original level. Source filtering remains on the server; the client admits the previously filtered replay record. The independent NDJSON layer retains `daemon_*` metadata, including exact timing values and encoded field and span arrays. Use those original nanoseconds for calculations instead of parsing terminal text.

The formatter writes through `HumanWriter`, so [terminal ownership](terminal-ownership.md) can buffer its output while Ratatui owns the screen. It does not bypass that owner or add an unbounded event history. It sanitises remote control characters and escapes embedded newlines, keeping source strings from creating false log headers. Source UTC time describes when the daemon captured a record; receiver time describes delivery. Clocks on separate hosts need synchronization before comparing them.

Interactive daemon events reach the parent's same source-aware formatter as typed records. Human delivery uses its terminal buffer while its NDJSON preserves the original `daemon_*` fields. The internal child keeps compact timestamp-and-level formatting only for its own stderr. Its parent recognises that prefix to preserve local worker levels and filtering, then records those lines under `worker_log`.

On a terminal, source-aware records match `tracing-subscriber`'s pretty palette: error is red, warning yellow, information green, debug blue and trace magenta. Messages and event values use the severity colour; targets and event field names add bold. Timestamps are dim, source-context labels are dim and italic, and span names are bold. Client and daemon labels use green and blue respectively. Sanitization happens before the formatter adds these styles.

The newer local log pane converts the formatter's small SGR vocabulary into Ratatui foreground, bold, dim and italic styles. It follows [Cloud-Terrastodon's structured log presentation](terminal-ownership.md#buffer-human-logs-while-the-screen-is-active) without treating log text as a terminal emulator. Only reset, those modifiers and the five severity colours become styles; other controls remain escaped data. Model text still uses the plain sanitizer. Real interactive screens enable formatter styling; scripts, redirected stderr, NDJSON and the internal worker stay ANSI-free.

Grounding in the matching local service checkout: `crates/teamy_llm_cli/src/logging/source_aware.rs`, `logging.rs`, `remote_logging.rs`, `interactive/terminal_output.rs`, `interactive/log_style.rs` and `interactive/ui.rs`. [MFT's logging initialization](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/logging_init.rs) provides prior art for a source-aware formatter. The local source additions are separate from the historical executable identities and timings in [resident inference](resident-inference.md#qualified-final-local-build).

## Choose a representation before styling a TUI log

Cloud-Terrastodon and `teamy-llm` protect their screens by deferring human delivery. Capture, filtering and the separate file layer continue while the terminal is owned. Ratatui does not provide that coordination itself.

| Implementation | What the screen retains | How it applies styles |
| --- | --- | --- |
| Cloud-Terrastodon terminal layer | Records with level, message, source and span fields. Its field values are strings. | The picker maps record levels directly to Ratatui `Style` for toasts. No ANSI parser is required for that path. |
| Local `teamy-llm` human writer | Bounded formatted text while a terminal lease is active. | `styled_log_lines` converts the formatter's approved SGR codes into Ratatui `Line` and `Span` styles. |
| Local `teamy-llm` daemon transport | Owned Facet records with source metadata, span context and timings. Field values are still strings. | The client replays the source record through its formatter before the human writer and TUI conversion. |

The current Cloud-Terrastodon reference is [`223e7889`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/223e7889fa98fef9838e1b5da6d57ca10eb6cf11), inspected on 5 October 2026. Its [terminal layer](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/tracing/src/structured_log.rs), [picker log adapter](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/app/src/terminal.rs) and [toast renderer](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/user_input/src/picker/picker_tui.rs) ground this distinction. Its inactive human sink formats the record separately; retaining a record does not require retaining its terminal escape sequences.

Use this sequence when reproducing the local LLM conversion:

1. Acquire terminal ownership before changing modes. Route the human formatter through the guarded writer; keep the file layer independent.
2. Sanitize source-controlled text before adding our styles. Enable ANSI for the real interactive screen; keep script, redirected and file output plain.
3. Parse only the formatter's vocabulary: reset `0`, bold `1`, dim `2`, italic `3` and foreground colours `31` through `35`. Combined codes update the current style; reset clears it. Bound parameter scanning to 64 bytes.
4. Produce styled Ratatui spans and preserve their styles when wrapping by terminal display width. Escape unknown, incomplete and non-SGR controls as visible data. Model output uses the plain sanitizer.
5. Restore terminal modes before releasing the lease. Qualify success, cancellation, panic and partial setup failures, then verify that later human output works.

The LLM lease release permits future stderr delivery; it does not automatically replay buffered history. Cloud-Terrastodon's replay is a separate operation too. Follow [terminal ownership](terminal-ownership.md) for nesting, retention and restoration differences. Converting ANSI solves presentation; it does not recover typed fields from formatted text.

## Attach tracing spans to eyre errors

`eyre` propagates contextual error reports. `color-eyre` installs their human rendering hook. [`tracing-error::ErrorLayer`](https://docs.rs/tracing-error/0.2.1/tracing_error/struct.ErrorLayer.html) supplies the span context that [`SpanTrace`](https://docs.rs/tracing-error/0.2.1/tracing_error/struct.SpanTrace.html) captures. [color-eyre](https://docs.rs/color-eyre/0.6.5/color_eyre/) enables tracing integration and spantrace capture by default. Feature selection, hook configuration and `RUST_SPANTRACE` can change that behavior.

| Diagnostic | What it explains |
| --- | --- |
| Error chain and `wrap_err` context | Which operations failed and which underlying errors they retained. |
| Stack backtrace | Machine stack frames at capture, rather than the whole async workflow. |
| Span trace | The active logical tracing spans at capture, with their recorded fields. |
| Transported source-span snapshot | Context from another process, explicitly copied into a wire record. |

Install the error hook during startup and `ErrorLayer` on the subscriber that runs the work. Install that layer independently of human/file destinations; a quiet console or missing `--log-file` must not disable error context. A report created before the subscriber is active cannot capture spans that did not exist yet. Capture spans through instrumented futures; do not hold an entered-span guard across unrelated async work.

Cloud-Terrastodon's [subscriber construction](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/tracing/src/lib.rs) now installs `ErrorLayer` and contains a capture regression for terminal and ordinary modes. Its [application hook setup](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/app/src/lib.rs) builds the color-eyre hooks once, retaining the panic hook for later installation. These tests were inspected, not rerun for this documentation change. The local LLM also installs `ErrorLayer` in `crates/teamy_llm_cli/src/logging.rs` and color-eyre in `crates/teamy_llm_cli/src/lib.rs`.

If capture reports `Unsupported`, check that the active dispatch has the error layer and that resolved versions are compatible. `Empty` means capture support exists but no spans were active. Changing report colours does not fix missing capture. Preserve source errors with `wrap_err`; rebuilding an error from its display string loses its underlying chain.

A span trace identifies local span ancestry, not a distributed trace. Its field formatting is lazy. `SpanTrace::with_spans` exposes span metadata and formatted field text when visited; create an owned snapshot at the boundary when capture-time values matter. Sending it cannot restore the original field types or another process's live span IDs. Cross-process errors need their own owned contract; see the next section and [typed results and errors](output-shape.md).

## Preserve field types before crossing IPC

Tracing supports primitive fields and visitor callbacks. Our current collectors make a lossy choice: Cloud-Terrastodon's `CapturedFields` and the LLM's `RemoteLogField` turn values into strings. Facet makes the surrounding record serializable, but it cannot distinguish the integer `12` from a string containing `12` afterward. Fields emitted with `?value` or `%value` also erase arbitrary structures into Debug or Display presentation.

Override the primitive callbacks in [`tracing::field::Visit`](https://docs.rs/tracing/latest/tracing/field/trait.Visit.html) to preserve their types. Its arbitrary-value integration uses the separate `valuable` trait and currently requires both the `valuable` feature and `tracing_unstable`; deriving Facet alone does not enable it.

Existing Facet prior art is [tracing-wide 0.2.0](https://docs.rs/tracing-wide/0.2.0/tracing_wide/), listed in [Facet's observability ecosystem](https://facet.rs/ecosystem/#observability). It offers typed events and an opt-in `facet::Peek` hook before forwarding to tracing. This is a community-maintained adapter candidate, not an adopted dependency or a replacement dispatcher. Its Facet version must be checked against our pinned fork before reuse.

The first useful ownership boundary is a shared telemetry contract and its renderers, while retaining tracing instrumentation from our code and dependencies. The following is a design direction, not an implemented library:

| Boundary to own | Contract to establish |
| --- | --- |
| Event and span capture | Preserve booleans, signed/unsigned numbers, strings and explicitly formatted fallback values before rendering. Opt into schema-bearing Facet payloads at the emitter. |
| IPC envelope | Version the record. Retain source process/session, request correlation, sequence, source timestamp, span identity/parent context, original metadata, timing units and drop/truncation evidence. |
| Error transport | Own a stable error category, contextual/source-chain text and captured source spans. Mark formatted backtraces as presentation data; do not attempt to serialize an arbitrary `eyre::Report`. |
| Presentation and storage | Render the same record directly to Ratatui styles, human terminal text and Facet-encoded files. Apply bounded retention and overflow policy independently of display. |

Process-local span numbers require a source/session identity. Preserve the caller's correlation context when a server creates its own spans; receiving an event does not make the receiver its source. Carry the active tracing span and dispatch into spawned or blocking work deliberately. Source and receiver timestamps answer different questions, especially across machines.

[`tracing-serde`](https://docs.rs/tracing-serde/latest/tracing_serde/) supplies serialization adapters. The subscriber's [JSON layer](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html) uses Serde; core instrumentation does not require choosing that formatter. Replacing our file layer with a Facet writer can remove that chosen Serde path, but it does not prove the whole dependency tree is Serde-free. Inspect Cargo features and reverse dependencies before making that claim. Serialization alone does not recover erased values.

Qualify a new adapter with typed-field round trips, source attribution, ordered span updates, async context propagation, per-request isolation, bounded overflow and cancellation. Keep terminal safety and [timing methodology](performance-analysis.md) intact. Replace the tracing dispatcher or eyre report mechanism only if a measured requirement remains unmet after the boundary is owned; a full replacement would also need compatibility with third-party instrumentation.

Human and JSON span-field formatters need distinct caches. Cloud-Terrastodon's current setup uses `.pretty()` with `PrettyFields` and a separate `.json()` layer. Merely changing the JSON event formatter can leave human ANSI field text in a cache later parsed as JSON. [Its subscriber source](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/223e7889fa98fef9838e1b5da6d57ca10eb6cf11/crates/tracing/src/lib.rs) records that failure mode. Owned rendering should keep presentation-specific strings separate from captured values.

## Buffering means several different things

Choose the mechanism for the behavior required. Retaining logs for a screen, batching disk writes and delivering logs from another process solve different problems.

For an active Ratatui screen, read [terminal ownership and guarded logging](terminal-ownership.md#buffer-human-logs-while-the-screen-is-active). Cloud-Terrastodon routes human tracing events into structured memory records while a terminal owner is active, then presents or replays them; its NDJSON layer stays independent. Stdout and stderr can reach the same PTY, so putting logs on stderr alone does not protect a stdout TUI. This coordinator and buffer are reference capabilities, not additions already supplied by the CLI template.

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
