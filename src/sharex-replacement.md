# Plan a fast Rust screenshot workflow

Build a small, dependable screenshot workflow before attempting all of ShareX. The goal is fast interaction from hotkey to a usable image, with a CLI that assistants and other programs can call. Rust is the implementation preference, not evidence that the replacement will be faster.

**Status:** planning and source review, 12 November 2026. ShareX sometimes feels slower than desired; no controlled comparison or bottleneck measurement has been performed for this plan. No replacement application is delivered by this page.

## Prior art and what to reuse

| Project | Useful contribution | Adoption boundary |
| --- | --- | --- |
| [ShareX](https://github.com/ShareX/ShareX/tree/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf) | Established capture, region selection, image editing, hotkeys and post-capture workflows. Use its behavior as the comparison baseline. | A broad product, not just a pixel-copy function. Its [GPL licence](https://github.com/ShareX/ShareX/blob/9fb873fcbf5060d789d5c14ce54180a0d13f3bbf/LICENSE.txt) needs review before code reuse; this plan copies no implementation. |
| [Cursor Hero](https://github.com/TeamDman/Cursor-Hero/tree/9ba3be8c66d468f0d3e54da5e6b1b4cd0e7718e5) | Rust/Bevy monitor textures, physical versus synthetic input, cursor interaction, window placement and UI Automation observations. | Reuse concepts and isolated components, not the entire game/rendering loop for a screenshot hotkey. UI Automation coverage is partial. |
| [WINC](https://github.com/TeamDman/winc/tree/8a752ac6a30c8698098c936f5fb0fa0141f26c96) | The authored `winc` crate, pronounced **wink**, captures Windows monitor regions. Its README explicitly aims to reduce redundant work between captures. | Candidate GDI backend, not a finished editor or guaranteed fastest backend. It derives capture work from Cursor Hero and credits screenshots-rs. Review provenance and native-resource correctness before reuse. |
| [Teamy Windows](https://github.com/TeamDman/teamy-rust-windows-utils/tree/155c5efdfc6ceaba10f2e849b9eee2b5742a27b0) | The `teamy-windows` crate lives in `teamy-rust-windows-utils`. Existing window, event-loop, tray, icon, clipboard and process helpers can support a small resident app. | Review individual helpers rather than assuming a complete application framework. The inspected clipboard API reads/writes text; image clipboard formats still need an explicit implementation. |

These are inspected source revisions, not claims about the newest releases. See [screen capture and window observations](screen-capture.md) for capture semantics, DPI and known WINC GDI-contract issues; [window creation](window-creation.md) and [system trays](system-tray.md) cover Windows integration limits.

WINC retains a compatible device context and bitmap between captures, but still allocates an output buffer and performs readback and BGRA-to-RGBA conversion. A persistent resource does not imply zero-copy capture. Do not preserve an unsafe implementation just to preserve an optimization.

## Measure the delay the user experiences

Compare the same capture region, destination, encoding and optional tasks. Record ShareX's version and settings; a capture-only path is not comparable to capture plus editor, OCR and upload. Separate cold startup from a resident hotkey path.

Instrument these boundaries with monotonic timestamps:

1. Hotkey received to selection overlay ready.
2. Selection accepted to pixels available.
3. Pixels available to clipboard image ready or file published.
4. Optional editor ready, encoding complete, and external task completion.

Report median and p95 latency, CPU time, peak working set, idle CPU and GDI-handle counts across repeated captures. Retain resolution, DPI, monitor arrangement and backend as structured benchmark metadata, using synthetic content rather than private desktop images. Measure allocation and readback costs separately from overlay initialization and image encoding. Use [the performance analysis workflow](performance-analysis.md) instead of inferring a cause from the implementation language.

Keep the benchmark small: a fixed region to PNG, a window-sized region to clipboard, and interactive region selection. Test both warm repeats and first launch. Set an acceptance target after measuring the baseline; do not invent a speedup or discard functionality to manufacture one.

## First useful vertical slice

Start any new implementation from [teamy-rust-cli](rust-cli-template.md), retaining its logging, structured output, cancellation and other applicable conventions. This page does not create a new project or install a background service.

The first slice should support:

- A CLI capture of an explicit monitor region to a local PNG.
- A resident hotkey that opens a region-selection overlay, then copies the image or saves it locally.
- Escape/cancel that dismisses the overlay without modifying the clipboard or producing a file.
- A visible tray state and a reliable exit command. Do not start continuous recording.
- Typed capture targets, output destinations, coordinate spaces and errors; runtime configuration rather than embedded personal settings.

Suggested command families are `capture`, `serve`, `status` and `stop`. They are **proposed**, not commands supplied by WINC or Teamy Windows. UI and CLI should invoke the same operations, with structured receipts identifying dimensions, timings and output status. Keep image bytes and file paths out of routine diagnostics unless needed and deliberately enabled.

Keep acquisition, selection UI, encoding, clipboard publication and optional follow-up actions separate. A lightweight client can request a capture from a resident process, avoiding repeated initialization; it must not leave stale workers or silently launch several owners. If a daemon fails, return a typed error or explicitly start the permitted owner rather than hanging.

## Backend experiments, not an early rewrite commitment

Use a corrected WINC/GDI path as one baseline. Compare it with [DXGI Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api) and [Windows.Graphics.Capture](https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture) only after identifying where time is spent. Their frame, cursor, permission and device-lifetime contracts differ. Faster GPU acquisition can still lose its advantage during CPU readback or PNG encoding.

Make the backend replaceable behind a typed capture contract. Specify physical-pixel bounds, timestamp, pixel format, cursor inclusion and failure conditions. Cache resources by compatible display/size and invalidate them on topology changes. Bound work and memory; do not run a high-rate capture loop when one requested frame will do.

## Correctness and workflow scope

Before replacing a daily workflow, test mixed DPI, negative monitor coordinates, multiple monitors, transparent/cursor behavior, display changes, cancellation, repeated hotkeys, clipboard contention and shutdown during capture. Restore GDI selections and release handles on errors. An occluded desktop rectangle is not an isolated capture of that window; specify the behavior rather than promising hidden-window pixels. Secure/protected content remains outside the capture guarantee.

Use explicit states such as idle, selecting, capturing and publishing. A second hotkey must have a defined outcome in each state. File output should not expose a partial final image. Clipboard success and save success are separate outcomes.

Defer the broader ShareX feature set until the first slice is useful:

- Annotation, redaction and editor undo/redo.
- Scrolling capture and OCR.
- Screen recording, audio, GIF/video encoding and replay/history.
- Uploaders, cloud destinations and share-link creation.

ShareX can remain installed for these features. Optional follow-up actions must be visible and separately configured; taking a screenshot must not silently upload it. Keep retained images, settings and logs outside the repository. Confirm the desired output destination and retention policy before introducing persistent history.

## Next decision

Measure the existing workflow, build the smallest capture-to-clipboard/save slice, and compare it under the same conditions. Keep the implementation only if it improves the daily interaction without losing correctness or increasing idle overhead. Extract proven reusable capture and Windows utilities into their appropriate crates; avoid making the screenshot tool depend on Cursor Hero's whole application.
