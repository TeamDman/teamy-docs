# Keep a terminal UI responsive during work

Let one UI loop own screen state and terminal input. Let background work return typed events. A model load, network request or GPU kernel should not run inside the closure that draws a frame.

Cloud-Terrastodon demonstrates two async designs at public revision [`eadf23c`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/eadf23c349c50a8a745df3e206536f4a53a419bb), inspected on 2 October 2026. Its picker runs search handlers concurrently; its object browser separates an engine from the terminal view. These are prior art to study, not a requirement that every new TUI implement the same object domain.

## Separate events, state and rendering

```text
Keyboard / resize ------+
                        |
Worker results ---------+--> UI loop --> application state --> Ratatui frame
                        |
Cancellation / logs ----+

UI request --> resident service or worker --> typed result / progress events
```

The [picker event loop](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/user_input/src/picker/picker_tui.rs) uses Crossterm `EventStream`, a 16 ms interval, a query debounce and `FuturesUnordered` handler futures. `tokio::select!` gives ready ownership controls and keyboard input priority over continuously ready background work. Resize requests a redraw; changes mark rendering dirty. Handler completion and candidate arrivals update state rather than writing directly to the terminal.

New queries receive generation identifiers. Candidate messages from an older generation are discarded, preventing a slow earlier search from replacing results for the current text. The handler futures are polled by the application loop; concurrency here does not mean every handler owns an operating-system thread.

The [object browser loop](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/ui_ratatui/src/object_browser/run.rs) uses a 256-command engine channel and drives engine and UI futures together. The terminal loop draws at 60 Hz and awaits keyboard events and ownership controls. If the engine stops while the UI is active, that is an error. Terminal and log contexts are attached to invocation futures explicitly.

## Own the input reader

Use one active terminal input path. Crossterm's [0.29 event API](https://docs.rs/crossterm/0.29.0/crossterm/event/index.html) requires either `read`/`poll` on one thread or `EventStream`; do not mix those approaches or read the same terminal concurrently from unrelated workers. `EventStream` requires its `event-stream` feature. [Ratatui's async tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) demonstrates Tokio selection, though its example dependency versions differ from our resolved lockfile.

While an owner is suspended for a nested picker, stop polling its input stream and stop drawing. Resume only after its backend has been reinitialized. [Terminal ownership](terminal-ownership.md) supplies the handoff protocol; a keybinding alone does not coordinate the competing readers.

Raw mode changes how keyboard input is delivered. Define whether Ctrl+C cancels the current inference request, closes the whole interface, or asks for confirmation. A process-wide console signal handler and a raw key event are different paths. Route both to the intended [cancellation token](cancellation.md), and keep the choice visible in on-screen help.

## Bound work and retain a resident service

For local inference, create the service once and reuse its tokenizer and runtime caches across requests. Workers send generated text or decision results to the UI. The UI stays able to draw loading progress, process resize events and request cancellation while work is running.

Choose queue and retention limits explicitly. Cloud-Terrastodon's picker candidate channel and terminal log buffer are unbounded despite the browser's bounded engine channel. A long-running interface should bound queued requests, token events, logs and conversation history separately. Capacity measured in messages alone does not bound bytes when one message can contain an arbitrary string.

Cancellation requests a cooperative stop. It does not preempt a GPU kernel or cancel a blocking read automatically. If a producer is waiting for channel capacity, cancellation must also wake that wait. Track the worker's completion before announcing that it stopped; do not detach it and leave hidden GPU work running.

teamy-tts offers [line-oriented prior art](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/src/cli/interactive.rs): a dedicated stdin thread sends lines to the async loop, which selects between input and cancellation. On cancellation it detaches a potentially blocked reader rather than joining it. This is useful for a process about to exit; it does not provide a reusable cancellable terminal reader inside a continuing application.

## Test the behavior that matters

Keep state transitions and result formatting separate from live terminal I/O. Ratatui's [TestBackend](https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html) can exercise rendering without taking control of a user's terminal. This does not prove raw-mode restoration or real host behavior.

Qualify the distinct boundaries with focused checks:

1. Input, mode changes and worker results update the intended typed state; stale results are ignored.
2. Queue saturation and cancellation can complete without deadlocking the UI.
3. Drawing or worker failure restores the terminal and propagates an unrecoverable error.
4. Logs do not clobber the screen; file logging continues; batch stdout remains parseable.
5. A real terminal session handles Unicode, paste, resize, cancellation and normal exit.

Measure request latency separately from draw latency and cold model preparation. [Performance analysis](performance-analysis.md) explains the tracing and capture evidence needed to distinguish them.
