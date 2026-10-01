# Desktop applications

Choose the behavior your application needs, then reuse its prior art. A tray icon, a hidden event window and a rendered terminal have different responsibilities. They can share an application without sharing one implementation.

| Need | Start here | Prior art |
| --- | --- | --- |
| Tray status, menus or a background control surface | [System tray icons](system-tray.md) | Piing and tb. |
| Create a window and dispatch Windows events | [Window creation](window-creation.md) | tb and Piing's teamy-windows helpers. |
| Host interactive programs or render terminal output | [Terminal integration](terminal-integration.md) | Teamy-Studio. |

The CLI template does not currently supply a tray icon or desktop event loop. Add that capability explicitly when it serves the application. Connect background work to [logging](logging.md) and [cancellation](cancellation.md), and keep slow operations off the event thread.
