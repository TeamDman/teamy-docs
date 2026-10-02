# Own the terminal and keep logs readable

Give one active interface control of terminal modes, input and screen drawing. Keep operational events flowing to independent destinations while the interface owns the screen. A lock around `draw()` alone does not coordinate raw mode, input readers, nested pickers or writes from tracing subscribers.

Our implementation reference is Cloud-Terrastodon at public revision [`eadf23c`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/eadf23c349c50a8a745df3e206536f4a53a419bb), inspected on 2 October 2026. Its [terminal coordinator](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/terminal_coordinator.rs) knows nothing about Ratatui or Crossterm. An owner implements backend suspension and resumption, then acknowledges the completed transition.

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
