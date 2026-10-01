# Capture a screen region or inspect a window

Choose whether you need pixels or application state. A screenshot records visible pixels. UI Automation can expose controls and values without interpreting an image. Use the smallest permitted target, then keep acquisition separate from model processing and actions.

These are Windows-specific references. The inspected Rust CLI template does not supply screen capture or UI Automation. Start with [the template](rust-cli-template.md), then add the required acquisition capability explicitly.

## Choose the matching prior art

| Need | Start with | Public implementation |
| --- | --- | --- |
| Capture a monitor region into an RGBA image | WINC, pronounced 'wink'. | [Region construction and capture at `8a752ac`](https://github.com/TeamDman/winc/blob/8a752ac6a30c8698098c936f5fb0fa0141f26c96/src/monitor_region_capturer.rs#L44-L163). |
| Inspect controls through typed application snapshots | Cursor-Hero's UI Automation types and collector. | [Snapshot types at `9ba3be8`](https://github.com/TeamDman/Cursor-Hero/blob/9ba3be8c66d468f0d3e54da5e6b1b4cd0e7718e5/crates/ui_automation_types/src/ui_automation_types.rs#L31-L55) and [collector](https://github.com/TeamDman/Cursor-Hero/blob/9ba3be8c66d468f0d3e54da5e6b1b4cd0e7718e5/crates/ui_automation/src/take_snapshot.rs#L7-L28). |
| Select window bounds and capture their desktop rectangle | ShareX's capture library. | [Screenshot methods at `9fb873f`](https://github.com/ShareX/ShareX/blob/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf/ShareX.ScreenCaptureLib/Screenshot.cs#L42-L150). |

These public snapshots were checked on 1 October 2026. WINC extracts Cursor-Hero's capture approach and reuses GDI contexts between captures. A similarly named `wink` stub is not capture runtime prior art. ShareX is a C# reference with a [GPL licence](https://github.com/ShareX/ShareX/blob/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf/LICENSE.txt); review reuse conditions before copying implementation code.

## Keep coordinates and visible pixels explicit

WINC accepts a monitor and rectangle. ShareX's window path resolves a window or client rectangle, then copies that region from the desktop. Another window covering the target can therefore appear in the result. Neither inspected path establishes isolated rendering of an occluded window.

Record the coordinate space, capture bounds and pixel dimensions. Test mixed monitor scaling and monitors with negative coordinates. [GetWindowRect is virtualized for DPI](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect), while DWM extended frame bounds are not adjusted for DPI. ShareX [prefers DWM bounds and falls back to GetWindowRect](https://github.com/ShareX/ShareX/blob/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf/ShareX.HelpersLib/Helpers/CaptureHelpers.cs#L320-L339), so coordinate conversion needs an explicit policy.

Capture APIs own native resources as well as returned images. WINC retains a bitmap and device context; its destructor attempts cleanup. However, the inspected code leaves the bitmap selected during `GetDIBits`, contrary to [that API's contract](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdibits). Check creation failures, restore selected objects, validate buffer sizes and define cleanup before adopting it. ShareX's normal path restores the selected bitmap and releases its GDI resources; it is still prior art, not a complete error-handling contract.

## Treat typed observations as partial coverage

Cursor-Hero uses Serde-derived structs and enums for application snapshots. Its collector traverses root application windows and omits unknown applications. [VS Code resolution is explicitly disabled](https://github.com/TeamDman/Cursor-Hero/blob/9ba3be8c66d468f0d3e54da5e6b1b4cd0e7718e5/crates/ui_automation/src/resolve_app.rs#L18-L31), despite the snapshot variant existing. Missing entries do not prove a window is absent.

Reuse those domain shapes with a selected-target collector and explicit unknown outcomes. Stable object IDs, snapshot revisions, Facet schemas and constrained intent proposals are a [proposed model boundary](model-adoption.md#define-the-typed-model-contract), not capabilities already supplied by this Serde implementation. Process IDs and window handles help locate a target; define identity and freshness separately.

## Keep capture separate from workflows

ShareX's [CLI dispatcher can upload files or download and upload URLs](https://github.com/ShareX/ShareX/blob/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf/ShareX/ShareXCLIManager.cs#L46-L72). Use a local capture API when the intended operation is acquisition alone. Its optional taskbar-hiding setting also changes desktop state during window capture.

Start with one explicit target and one frame. Specify whether pixels may be saved or passed to a model. Add bounded repetition, [cancellation](cancellation.md) and [typed outcomes](output-shape.md) deliberately. A captured frame is evidence for a proposal; it does not authorize input injection, uploads or other actions.
