# Integrate a terminal

Choose the terminal task first. Hosting an interactive program, interpreting its output and sharing an existing terminal require different machinery. Our prior art is Teamy-Studio for a PTY and terminal engine, and Cloud-Terrastodon for cooperative TUI ownership.

| Need | Start with |
| --- | --- |
| Host an interactive shell in a native window | Teamy-Studio's PTY session, terminal core and engine. |
| Share an existing terminal with nested prompts or TUIs | Cloud-Terrastodon's terminal coordinator. |
| Keep elevated or daemon diagnostics visible | [Console attachment and RPC logging](logging.md#elevation-and-daemon-log-forwarding). |

The CLI template supplies parsing, logging, typed output and cancellation. It does not supply a PTY host, terminal emulator, GUI renderer or nested TUI coordinator.

## Separate process transport from interpretation

Teamy-Studio's [`TerminalCore` at public revision `e338ba2`](https://github.com/TeamDman/Teamy-Studio/blob/e338ba27d778dc1b99b034fc4857c1e0a0773626/crates/teamy_studio_terminal_core/src/windows_terminal_impl.rs#L1720-L1815) opens a PTY through `portable-pty`, spawns the child and obtains input and output handles. A dedicated reader thread sends output through a bounded channel. The terminal worker owns the child, PTY and mutable terminal state.

A bridge publishes updates and posts a Win32 wake message. The interface consumes snapshots instead of reading the child stream directly:

```text
Child -> PTY output -> reader -> worker/engine -> snapshot -> UI
Child <- PTY input <- input writer <- UI input / engine replies
```

At this public revision, [`TeamyTerminalEngine`](https://github.com/TeamDman/Teamy-Studio/blob/e338ba27d778dc1b99b034fc4857c1e0a0773626/crates/teamy_studio_teamy_terminal_engine/src/teamy_terminal_engine_impl.rs) interprets the bytes. Older Ghostty notes describe a different implementation. The engine retains partial UTF-8 and escape sequences across writes, maintains screen/scrollback state and generates replies through a callback.

Existing tests cover split sequences, cursor operations, alternate screens and replies. They establish those behaviors, not complete terminal compatibility. When replacing a component, reduce the failing byte stream to a fixture and check state transitions before attributing an error to rendering. Measure responsiveness under output bursts, resizing and input together. One bounded reader channel does not make every queue bounded.

## Coordinate ownership of an existing terminal

Cloud-Terrastodon's [`TerminalCoordinator`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/terminal_coordinator.rs) lets nested terminal users cooperate. The current owner suspends its backend before acknowledging a handoff. Releasing the nested owner resumes the parent. Failed transitions poison the coordinator.

The [picker backend](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/picker/picker_tui.rs#L1033-L1050) performs raw-mode and alternate-screen transitions. This coordinator is not an OS lock and cannot control unrelated writers. Its task-local context also needs propagation across spawned-task boundaries.

## Define shutdown and stream ownership

Windows pseudoconsole channels are synchronous. Servicing and teardown can deadlock when output is not drained. Preserve ownership, servicing and shutdown together when adapting a PTY implementation. See [Microsoft's pseudoconsole hosting guidance](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session).

Connect workers to [cancellation](cancellation.md) and [logging](logging.md), while keeping terminal byte streams distinct from command-result stdout. The cited Studio modules match the inspected local implementation; newer local commits do not change the provenance of these public source references.
