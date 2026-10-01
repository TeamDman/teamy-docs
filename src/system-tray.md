# Add a Windows system tray icon

When you want to add a system tray icon to an app, start with Piing and tb. They demonstrate a status icon, menu callbacks, log access and an exit path. An embedded executable icon alone does not provide a tray lifecycle.

The inspected `teamy-rust-cli` template has Windows startup helpers but no tray window, notification icon or message pump. Add this capability explicitly. Keep the command-line interface available through [the template](rust-cli-template.md).

Use [the executable's embedded icon](executable-icons.md) for shared application artwork. [Native message boxes](native-message-boxes.md) covers Piing's menu-triggered reports and their manifest requirement.

## Choose the matching prior art

| Need | Start with | Implementation |
| --- | --- | --- |
| A compact tray mode, menu and global hotkey | tb's direct Win32 implementation. | [Tray setup at `975848f`](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs#L104-L126). |
| A tray application with background monitoring and richer menus | Piing's window and worker separation. | [Runtime at `d2c5065`](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/runtime.rs#L24-L66) and [menu handlers](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/tray/window_proc.rs#L241-L355). |

These public snapshots were checked on 1 October 2026. tb chooses tray mode when invoked without a subcommand. It registers a hidden window, installs a configurable hotkey, adds a notification icon and enters the message loop. Icon callbacks open its menu or toggle the taskbar.

Piing keeps the tray window and message loop on the calling thread. A separate thread runs a Tokio runtime for monitoring. Normal shutdown sends a watch-channel signal and joins that worker. This is a useful pattern for keeping background work separate from Windows event dispatch.

## Preserve the icon lifecycle

Both applications handle the registered `TaskbarCreated` message to restore the icon after Explorer restarts. Both remove the icon during normal window destruction. Piing also reapplies its latest status icon. See [its window procedure](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/tray/window_proc.rs#L377-L433).

Read the entry point, callback window, icon registration and exit path together. A copied icon-creation function is insufficient. [Window creation and event loops](window-creation.md) explains the hidden window and ownership pattern; [logging](logging.md#learn-from-piing-and-mft) explains replaying messages from a tray menu.

## Adapt the known limits

Worker separation does not make every menu operation asynchronous. Piing's configuration reload and audit handlers perform synchronous work in the window callback. Its normal post-loop worker join is bypassed if tray startup returns an error. Define how a new application stops and joins work on both success and failure paths.

Both reference message loops convert `GetMessageW` directly to a Boolean and miss its separate error result. Correct that when adopting the pattern; see [the event-loop contract](window-creation.md#handle-messages-and-resource-lifetimes).

Use [cooperative cancellation](cancellation.md) for background workers and [structured tracing](logging.md) for diagnostics. The tray supplies a control surface; the workers still own the work and its stopping boundaries.
