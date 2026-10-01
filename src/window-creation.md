# Create a window and handle events

A Windows app can need a window even without a visible interface. Tray callbacks, hotkeys and system broadcasts need an event receiver. The window procedure handles those events; the message loop dispatches them.

Use tb's direct Win32 setup or Piing's versioned helpers as prior art. For [system tray icons](system-tray.md), their hidden windows are part of the solution. For a rendered desktop terminal, continue to [terminal integration](terminal-integration.md).

## Register the class and create the right window

tb registers a `WNDCLASSW` with its callback, checks registration and calls `CreateWindowExW`. Piing uses `teamy-windows` 0.7.0 to register a `WNDCLASSEXW` and create its window. Both create hidden, normal top-level windows. Neither uses an `HWND_MESSAGE` message-only window. This matters for their `TaskbarCreated` broadcast handling.

Compare [tb's window setup at `975848f`](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs#L171-L215) with [the published window helper](https://docs.rs/crate/teamy-windows/0.7.0/source/src/window/create_window_for_tray.rs) and [tray helper](https://docs.rs/crate/teamy-windows/0.7.0/source/src/tray/add.rs). These window and tray helpers use a fixed class name, icon identifier and shared state; they do not establish a general multi-window framework.

Both attach application state through `GWLP_USERDATA`. `WM_CREATE` allocates it, callbacks retrieve it, and `WM_DESTROY` clears the pointer and reclaims the allocation. This is explicit unsafe ownership. Review callback reentrancy and every access path when adapting it. See [tb's state and callbacks](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs#L402-L501).

## Handle messages and resource lifetimes

`WM_CLOSE` calls `DestroyWindow`. Destruction cleans up and posts `WM_QUIT`. Resources have separate lifetimes:

| Resource | Matching cleanup |
| --- | --- |
| Window | `DestroyWindow`. |
| Popup menu | `DestroyMenu`; attached submenus are destroyed with their parent. |
| Registered hotkey | `UnregisterHotKey`. |
| Shared `LoadIconW` icon | Do not call `DestroyIcon` on it. |

Microsoft documents [menu destruction](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroymenu) and [shared-icon ownership](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroyicon).

Distinguish all three `GetMessageW` outcomes: an error (`-1`), quit (`0`) and an available message (positive). Both inspected reference loops currently use a Boolean conversion and miss the error distinction. The published helper also checks class registration with a debug assertion rather than a production error return. Use the [GetMessageW contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew) when adapting that code.

Keep slow work outside the event callback and define worker shutdown explicitly. Piing demonstrates that separation, with limits described in [the tray chapter](system-tray.md#adapt-the-known-limits). These implementations are discoverable prior art, with concrete adaptation work.
