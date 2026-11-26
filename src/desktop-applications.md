# Desktop applications

Choose the behavior your application needs, then reuse its prior art. A tray icon, a hidden event window and a rendered terminal have different responsibilities. They can share an application without sharing one implementation.

| Need | Start here | Prior art |
| --- | --- | --- |
| Choose widgets, state and rendering for a desktop UI | [UI framework comparison](ui-frameworks.md) | GPUI Kit, GPUI Fast, BUI, Makepad, Rune, Bevy, egui/eframe, Iced and raw DirectX evaluation. |
| Build an egui desktop tool or inspect an existing log pane | [egui and eframe](egui-eframe.md) | CM's collector and log widget; Hyper Backup Explorer's tiled application. |
| Render numeric charts or diagrams in an application or report | [Charts and diagrams](charts-and-diagrams.md) | Charton, charts-rs, egui-mermaid and Lini; surveyed rendering and syntax boundaries. |
| Reuse the icon embedded in an executable | [Executable icons](executable-icons.md) | The template, tb and teamy-windows. |
| Tray status, menus or a background control surface | [System tray icons](system-tray.md) | Piing and tb. |
| Show a native message box or configuration-error choices | [Native message boxes](native-message-boxes.md) | Piing's task dialogs and tb's message boxes. |
| Create a window and dispatch Windows events | [Window creation](window-creation.md) | tb and Piing's teamy-windows helpers. |
| Capture a screen region or inspect application controls | [Screen capture and window observations](screen-capture.md) | WINC, Cursor-Hero and ShareX. |
| Host interactive programs or render terminal output | [Terminal integration](terminal-integration.md) | Teamy-Studio. |

The CLI template does not currently supply a tray icon or desktop event loop. Add that capability explicitly when it serves the application. Connect background work to [logging](logging.md) and [cancellation](cancellation.md), and keep slow operations off the event thread.

## Button sounds for UI feedback

[Button Sounds](https://buttons.joshwcomeau.com/) is Josh W. Comeau’s interactive collection of recorded clicks, presses, and toggles. It uses multiple recordings of each sound so repeated presses don’t play the exact same sample. The [source repository](https://github.com/joshwcomeau/button-sounds) includes a JavaScript library and documents [standalone WAV and MP3 files](https://github.com/joshwcomeau/button-sounds/blob/main/docs/raw-audio-files.md) for use outside JavaScript.

Use it as a reference for optional tactile feedback: keep sounds quiet and easy to mute, and never make sound the only indication that an action succeeded. See the [Hacker News discussion](https://news.ycombinator.com/item?id=50638032) for the tradeoffs around audible feedback and sample variation.
