# Build desktop tools with egui and eframe

[egui](ui-frameworks.md#egui-and-eframe) supplies immediate-mode UI; eframe hosts an app and its window, input and renderer. Application state and background work remain the application's responsibility. The [framework comparison](ui-frameworks.md) describes alternatives and qualification gaps. The examples below establish existing use, not a requirement that every Teamy application use this stack.

## Existing projects

| Project | Observed use | Reference |
| --- | --- | --- |
| CM | eframe 0.33 with persistence, egui_tiles 0.14.1 and a logs tile using the TeamDman egui_tracing development fork. Its subscriber combines an event collector, stderr and an optional JSON file. | [Manifest](https://github.com/TeamDman/cm/blob/b69ca7988218bfc0df8337b0dff6db0801b1e21d/Cargo.toml), [subscriber](https://github.com/TeamDman/cm/blob/b69ca7988218bfc0df8337b0dff6db0801b1e21d/src/tracing.rs), [logs tile](https://github.com/TeamDman/cm/blob/b69ca7988218bfc0df8337b0dff6db0801b1e21d/src/gui/tiles/logs.rs), [GUI](https://github.com/TeamDman/cm/blob/b69ca7988218bfc0df8337b0dff6db0801b1e21d/src/gui/mod.rs). |
| Teamy Hyper Backup Explorer | eframe 0.33 and egui_tiles 0.14.1 for a tiled backup explorer; its application integrates an event loop and tracing. | The inspected local revision `f1bbf84a1d8da9a04ea8bf0fdd707fda51091b86` includes local commits; its availability remotely has not been verified. The [repository](https://github.com/TeamDman/teamy-hyper-backup-explorer) contains the project. |

These versions describe the inspected implementations, not a shared pin or an upgrade recommendation. Compatibility of egui and companion crates must be checked together. Neither this review nor an existing GUI proves accessibility, input-method behavior, or performance for a new application.

## Reuse the log UI with care

CM's collector is a `tracing_subscriber` layer, and its logs tile passes a collector handle to `egui_tracing::Logs`. The [inspected fork](https://github.com/TeamDman/egui_tracing/tree/1042b72161d2462ab8dc89095663456e2f08ad5a) provides a table with level and target filters and a clear action. CM's inspected manifest also has a local path patch to that fork; a dependency declaration alone does not establish a reproducible shared build.

The inspected collector stores an unbounded vector behind a mutex, clones its entire history for reads, formats fields as debug strings and does not use span context. Its crate also depends on Serde. It is useful presentation prior art, but those storage and type-loss choices need review before reuse for high-volume or cross-process telemetry. The [proposed typed telemetry contract](logging.md#preserve-field-types-before-crossing-ipc) describes a bounded, source-aware alternative; it is not yet an implementation.

Keep network and disk work off the eframe update path. A reusable egui widget should consume an application-owned, bounded data source and request repaint when new data arrives; the eframe application owns window lifecycle and presentation. Test accessibility, large histories, selection, shutdown and failure handling for the chosen workload.
