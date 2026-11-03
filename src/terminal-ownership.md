# Own the terminal and keep logs readable

Give one active interface control of terminal modes, input and screen drawing. Keep operational events flowing to independent destinations while the interface owns the screen. A lock around `draw()` alone does not coordinate raw mode, input readers, nested pickers or writes from tracing subscribers.

Our implementation reference is Cloud-Terrastodon at public revision [`eadf23c`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/eadf23c349c50a8a745df3e206536f4a53a419bb), inspected on 2 October 2026. Its [terminal coordinator](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/terminal_coordinator.rs) knows nothing about Ratatui or Crossterm. An owner implements backend suspension and resumption, then acknowledges the completed transition.

## Put logs above an inline view

Cloud-Terrastodon's existing checkout was fast-forward checked on 2 November 2026 and remained at [`fd989bd`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/fd989bd357cbfc5d60ad06aff2a7677a3a46491f). Its [inline backend](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/user_input/src/inline_terminal.rs) owns raw mode and a Ratatui `Viewport::Inline` on stderr without entering the alternate screen. With terminal ownership acquired, it clears the viewport, prints already-formatted log lines with CRLF, then reanchors the viewport at the actual cursor. Native terminal wrapping remains intact. The [log buffer](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/tracing/src/terminal_log_buffer.rs) advances and acknowledges a cursor only after successful output; undelivered records can be replayed at exit. Its optional debug capture is bounded independently, while the normal replay buffer shown there is not bounded by default.

Ratatui's [inline example](https://github.com/ratatui/ratatui/blob/0a2a7c0363a4806b0cf05c1915bf7cdd438f756c/examples/apps/inline/src/main.rs) instead uses `Terminal::insert_before` to render rows into scrollback. Choose deliberately: that method renders Ratatui cells, whereas Cloud-Terrastodon's formatter path preserves native wrapping and existing styled lines. Both require a single terminal owner. Bound queues, expose dropped records, sanitize untrusted controls before styling, serialize redraws with log writes, and keep file logging independent. Test output failures, resize, cancellation, initialization failure and terminal restoration; a `TestBackend` does not prove real terminal behavior.

The local [Teamy CLI template](rust-cli-template.md) has optional, caller-owned inline terminal, bounded log buffer and screen-stack building blocks. The buffer exposes loss counts and acknowledges replay only after successful writes and flushes. The screen stack changes application navigation, not terminal ownership. Codex copies the inline guard and escaped scrollback from template revision `9cbff90`, retaining its telemetry-specific capture and navigation. Synthetic PTY tests exercise normal and error cleanup, reentrant rejection and restoration failure; a test backend covers resize, while actual PTY resizing and more hosts remain unqualified. The guard is not a nested terminal coordinator. The optional Windows adapter now has a native tray and explicitly allocated-console backend, compiled and tested through injected logic but not activated or visually qualified. A process inside a shared terminal must not hide its parent window: on Windows, distinguish an application-owned console from an inherited console and ConPTY/Windows Terminal before attempting visibility changes. See [tray prior art](system-tray.md); the exact behavior of a given host still needs qualification.

Track the shared work and its implementation status in the [terminal and Windows foundation roadmap](terminal-foundation-roadmap.md).

Closing a console can terminate processes attached to it. The implemented, synthetic-tested
[separate console UI process](#keep-the-tray-owner-alive-when-the-console-closes) keeps the tray owner and
worker outside that console so closing the UI can leave the application alive;
the actual console Close button and desktop hosts are not yet live-qualified.

## Transfer ownership after restoration

The coordinator uses a stack of owners. A nested acquisition asks the current owner to suspend. The child receives control only after the parent has acknowledged suspension. Releasing the child waits for its parent to resume. The handoff is cooperative; it cannot stop arbitrary code that ignores the coordinator.

```text
Parent owns terminal
    -> child asks to acquire
    -> parent restores terminal modes and suspends its input/drawing
    -> parent acknowledges suspension
    -> child owns terminal
    -> child restores its terminal and releases ownership
    -> parent resumes and acknowledges
Parent owns terminal again
```

Cloud-Terrastodon's `TerminalBackend` exposes `is_active`, `suspend` and `resume`. `apply_terminal_control` performs the backend transition before acknowledging it. Failed setup, restoration or acknowledgement poisons the coordinator, preventing a new owner from assuming the terminal is safe. `TerminalGuard::release().await` is the normal release path; dropping a guard cannot perform that complete async handshake.

The source includes regressions for nested last-in-first-out ownership, cancelled acquisition and failure before suspension acknowledgement. Those tests were inspected for this chapter, not rerun. Reuse their scenarios when qualifying a new terminal integration.

The [picker](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/picker/picker_tui.rs) restores raw mode and leaves the alternate screen on normal completion. Its panic hook attempts restoration before delegating to the original hook. The [object browser](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/ui_ratatui/src/object_browser/run.rs) also reports restoration and release failures. Partial initialization needs cleanup too: raw mode may have succeeded before entering the alternate screen failed.

Restoration is a best effort during a panic or forced process termination. Make controlled completion, errors and [cooperative cancellation](cancellation.md#restore-an-interactive-terminal) run the explicit cleanup path before printing a final error or result.

## Buffer human logs while the screen is active

Cloud-Terrastodon's [application terminal session](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/app/src/terminal.rs) connects the same `TerminalActivity` probe to ownership and tracing. It constructs the log buffer before the runtime, then provides the coordinator and log view to the invocation.

Its [structured terminal layer](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/tracing/src/structured_log.rs) chooses the human destination per event:

| Terminal state | Human event destination |
| --- | --- |
| No active owner | Write the formatted record to stderr. |
| An owner is active | Queue a typed record for screen presentation or later replay. |

Records retain level, message, target, timestamp, fields and span context. The picker reads new records with a cursor and can show informative messages as toasts. The [tracing setup](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/tracing/src/lib.rs) installs the NDJSON layer separately. Buffering terminal logs therefore does not silence the log file or change a command's structured stdout contract.

This buffer is an unbounded Tokio channel plus an unbounded vector. Replay writes the retained records and does not clear them; `replay_to_stderr` ignores replay errors. These are observed limits, not defaults to copy into a long-running inference service. Choose a bounded record or byte budget, show how many records were dropped, and keep file durability separate from screen history. See [logging buffering mechanisms](logging.md#buffering-means-several-different-things).

Task-local contexts need deliberate propagation. A raw `tokio::spawn` does not inherit these terminal and log scopes. Cloud-Terrastodon [attaches them to invocation futures](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/ui_ratatui/src/object_browser/run.rs). For a smaller application, passing an explicit shared handle can make this dependency easier to inspect.

## Keep every writer in the contract

Exclusive ownership applies to terminal output from both stdout and stderr, because both may share one PTY. A full-screen view on stdout still needs guarded tracing stderr. A picker on stderr must restore the screen before returning selected values on stdout. A JSON command must never receive alternate-screen escapes in its stdout. See [terminal stream choices](terminal-interfaces.md#decide-which-stream-owns-the-screen).

Third-party code can bypass `tracing` and write directly to stderr. Subscriber buffering cannot intercept those bytes. Route that library's diagnostics through a callback when available, select its quiet logging configuration, or capture an isolated child's output. Do not claim a screen is protected solely because our own subscriber is guarded.

When another process needs interactive access, first restore and suspend the current interface, run the child with the intended terminal connection, then reinitialize the view. A child that needs an independent terminal requires a [PTY](terminal-integration.md), not just another terminal guard.

## Keep the tray owner alive when the console closes

The shared template has a synthetic-tested process foundation at local revision
`1d6de93`; Codex has a code-only adoption at `3017e63`. Defibrillator has
adopted the earlier foundation and needs its application-specific integration of
the newer messages and disconnect handling. None establishes actual Windows
Close-button behavior until the opt-in test below.
Windows sends `CTRL_CLOSE_EVENT` to processes attached to the console; its
documented handler contract describes cleanup before termination, including
when a handler returns true. See [HandlerRoutine](https://learn.microsoft.com/en-us/windows/console/handlerroutine).

[Piing's tray implementation](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/tray/window_proc.rs)
distinguishes inherited, detached and owned consoles and replays buffered logs
when it creates one. [tb's tray implementation](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs)
allocates and detaches its console and routes console control events toward
tray shutdown. They are useful prior art for ownership and replay, not evidence
that closing a same-process console preserves a background worker.

### Process boundary

Use an unelevated, Windows-subsystem tray **owner** that is not attached to the
UI console. It owns the application runtime, tray, input or telemetry sources,
bounded history, optional file sinks and shutdown. A separately spawned
console-subsystem **UI child** owns one dedicated console, its Ratatui view,
raw mode and input. Closing that console may end the child; the owner observes
the child exit, keeps its own work running, and allows Show to create a new UI.
Do not rely on a console-close handler to keep the owner alive.

The ordinary attached-terminal commands remain ordinary foreground commands.
In tray mode, `q` closes the UI while the tray owner remains active; the tray's
explicit Exit action stops the application. Hide requests an orderly UI close;
Show reopens it. These tray-mode semantics differ from the ordinary foreground
Codex command, where `q` exits telemetry. Never hide a
shared parent terminal. The background owner must not remain attached to the
console whose closure it is meant to survive. Process jobs, terminal-host
lifetime, Windows sign-out and system shutdown can impose additional limits;
qualify the actual launch environment rather than claiming survival in all
circumstances.

Use one child at a time, tracked by the actual child handle and an owner-assigned
generation. Model `Closed`, `Starting`, `Open`, `Closing` and `Stopped`.
Repeated Show during `Starting` or `Open` should coalesce; wait for the previous
child to exit and be reaped before spawning a replacement. Show during
`Closing` records one pending reopen, which runs only after that exit. Codex's
current code can miss a Show that races with a window Close before it detects
the disconnect; retrying Show after closure works. Ignore messages
from an old generation. A failed spawn, broken channel or unresponsive close
is reported and must not silently create a duplicate child. Do not kill an
unrelated process or infer ownership from a title or PID alone. On owner exit,
request child shutdown and wait for a bounded interval; the child also exits
on control-channel EOF. Failure to complete cleanup is reported rather than
presented as success.

### IPC and bounded replay

Use two anonymous pipes dedicated to the spawned child, not a public network
listener. The Windows implementation creates non-inheritable handles and starts
the child without handle inheritance. The child verifies its actual parent and
its creation time, then duplicates only the two designated endpoints. This avoids
leaking temporarily inheritable handles to a concurrent process spawn. The owner
launches a known bundled UI executable
without a shell or caller-provided executable path. It accepts only explicit
application-defined intents, never command strings to execute. An unauthenticated
named pipe is not a substitute for these private channels. Use a versioned,
length-delimited typed protocol with strict frame and queue limits.

The exchange includes `Hello`/`Ready` with protocol version and generation,
`Record` with sequence numbers, `Gap` loss indicators, `Ack`, and bounded
application-intent and transient-event envelopes. Consumers must define and
validate their own closed set of typed application messages. The owner receives
clean pipe EOF independently of process exit, cancels any UI-bound capture and
uses a bounded deadline to stop an unresponsive owned child. Keep control
messages separate from terminal text.
The owner is the source of truth: it continues receiving events while no UI is
open, keeps a bounded ring with explicit eviction and truncation counters, and
never blocks the receiver or recovery loop on a slow UI. Reopening requests a
snapshot of retained records, followed by live records. A per-child bounded
delivery queue reports gaps; it must not convert a slow child into unbounded
memory growth. An acknowledgement means the child actually presented a record;
it does not erase the owner's retained history. Reopening can display retained
history again, and records already evicted are explicitly unavailable. File
logging and sensitive event retention remain separate opt-in policies.
The template's existing `TerminalLogBuffer::acknowledge` removes delivered
entries, so it cannot by itself supply this retained reopen history; add a
separately bounded retained store or change the contract deliberately.

On Windows Codex's UI child opens `CONIN$` and `CONOUT$` explicitly for terminal
input and display, while the two private pipes carry protocol frames. The
template's non-Windows fallback uses standard streams for protocol transport.
Do not mix protocol frames with logs. Windows documents that redirected
standard handles may be pipes and that the console devices can be opened
separately: [console handles](https://learn.microsoft.com/en-us/windows/console/console-handles).
The current Crossterm 0.29 Windows raw-mode and event paths use the current
console input handle, but existing app code also checks standard streams for
terminal status and writes to stderr; those call sites need an explicit UI
transport rather than an assumption that piped standard streams are a TTY.
The Windows UI child uses `CREATE_NEW_CONSOLE`; verify the actual console and
host behavior in an isolated opt-in test.
See [process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
and [pipe handle inheritance](https://learn.microsoft.com/en-us/windows/win32/ipc/pipe-handle-inheritance).

### Ownership and staged qualification

The optional Teamy Rust CLI template should own the reusable child lifecycle,
bounded transport and test harness; template consumers copy reviewed modules
with their source revision. Codex owns telemetry-specific records, receiver,
retention and presentation. Defibrillator owns its input capture, binding and
recovery policy: loss of UI focus or child disconnect must cancel/disarm any
in-progress capture and preserve the previous binding. Its background runtime
and authorized input matching do not depend on the UI child. No privileged
service or DWM action is implied by this design.

Qualify and extend in stages as consumers adopt the foundation:

1. Pure state-machine and synthetic pipe tests cover repeated Show, concurrent
   close/reopen, spawn and handshake failure, stale generations, EOF, child
   crash, owner shutdown, malformed or oversized frames and no duplicate UI.
2. Exercise bounded queues, replay after reconnect, partial writes and flush
   failures, loss counters, sanitization and file-sink independence. A stalled
   UI must not stall the owner. Verify no ingested OTLP record is exported back
   to its sender and no unrelated raw global keystrokes enter logs or IPC.
   Only an explicitly initiated binding capture may send a bounded preview of
   its candidate binding to the UI; do not persist that preview as a key log.
3. A Windows process fixture, using invented data and no tray activation,
   checks that unrelated handles are not inherited, child identity, EOF and
   cleanup using hidden synthetic processes. Distinct real console ownership,
   `CONIN$`/`CONOUT$`, job and terminal-host behavior remain to be qualified.
4. After specific opt-in and a suitable graphics state, launch the tray owner,
   show its child console, close the child with the window Close button, confirm
   the owner and its source continue, reopen once, inspect bounded replay, and
   choose tray Exit. Repeat for both applications with synthetic data before
   any real telemetry capture or privileged recovery testing.
