# Desktop applications

Choose the behavior your application needs, then reuse its prior art. A tray icon, a hidden event window and a rendered terminal have different responsibilities. They can share an application without sharing one implementation.

| Need | Start here | Prior art |
| --- | --- | --- |
| Choose widgets, state and rendering for a desktop UI | [UI framework comparison](ui-frameworks.md) | GPUI Kit, GPUI Fast, Makepad, Bevy, egui/eframe, Iced and raw DirectX evaluation. |
| Render numeric charts or diagrams in an application or report | [Charts and diagrams](charts-and-diagrams.md) | Charton, charts-rs, egui-mermaid and Lini; surveyed rendering and syntax boundaries. |
| Reuse the icon embedded in an executable | [Executable icons](executable-icons.md) | The template, tb and teamy-windows. |
| Tray status, menus or a background control surface | [System tray icons](system-tray.md) | Piing and tb. |
| Show a native message box or configuration-error choices | [Native message boxes](native-message-boxes.md) | Piing's task dialogs and tb's message boxes. |
| Create a window and dispatch Windows events | [Window creation](window-creation.md) | tb and Piing's teamy-windows helpers. |
| Capture a screen region or inspect application controls | [Screen capture and window observations](screen-capture.md) | WINC, Cursor-Hero and ShareX. |
| Host interactive programs or render terminal output | [Terminal integration](terminal-integration.md) | Teamy-Studio. |

The CLI template does not currently supply a tray icon or desktop event loop. Add that capability explicitly when it serves the application. Connect background work to [logging](logging.md) and [cancellation](cancellation.md), and keep slow operations off the event thread.
