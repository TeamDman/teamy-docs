# Render charts and diagrams

Choose the rendering boundary around the job: plot numeric data, export report assets, display a diagram inside a UI, or compile a diagram language. These references were reviewed on 18 October 2026 from official documentation and source. No project was installed, built or executed here; performance and application compatibility remain untested. Inclusion is a survey record, not dependency adoption or a TeamyStyle requirement.

| Need | Reference | Output and integration boundary |
| --- | --- | --- |
| Compose statistical plots from data, layers and encodings | [Charton](#charton) | SVG by default; optional PNG/PDF and native/browser wgpu paths, with GUI integration recipes. |
| Produce chart assets from typed Rust or JSON | [charts-rs](#charts-rs) | SVG plus feature-gated raster formats; useful for reports and server-rendered dashboards. |
| Display Mermaid diagrams inside an egui application | [egui-mermaid](#egui-mermaid) | Native egui painting and Markdown integration; a supported syntax subset, not arbitrary Mermaid compatibility. |
| Author technical figures in a text DSL | [Lini](#lini) | SVG or an HTML wrapper through a CLI/library; Wasm and documentation integrations. |

## Charton

[Charton](https://github.com/wangjiawen2013/charton) provides an Altair-inspired Rust API for marks, encodings, composed layers and facets. The [README at `8c84dcc`](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/README.md) documents point/line/bar plots, distributions, heatmaps, polar and geographic examples, Polars conversion macros, Jupyter SVG display and a Nushell plugin. It uses its own Rust API; check the documented Polars macro/version pairing.

The [manifest](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/Cargo.toml) keeps SVG available by default, with optional CPU PNG, PDF and wgpu backends. Official [egui integration](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/docs/src/gui/egui.md) and [shared-surface guidance](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/docs/src/gui/zero_copy.md) describe UI adapters: direct surfaces need compatible host wgpu versions and host composition of deferred text. The separate renderer's [GPU-to-CPU readback](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/docs/src/concepts/gpu.md) adds copying; do not infer a measured frame budget from the architecture. Code is [Apache-2.0](https://github.com/wangjiawen2013/charton/blob/8c84dccf6c3100cc38907e707007cdc97138671f/LICENSE).

## charts-rs

[charts-rs](https://github.com/vicanso/charts-rs), reviewed at [`241667b`](https://github.com/vicanso/charts-rs/blob/241667b5c2b8e34bf36a4ba375b31fe34e195893/README.md), generates charts from typed Rust constructors or its own validated [JSON options](https://github.com/vicanso/charts-rs/blob/241667b5c2b8e34bf36a4ba375b31fe34e195893/docs/json.md). The README documents 29 chart types, including bars, lines, pies, candlesticks, heatmaps, Sankey, Gantt, maps and composites. ECharts inspiration does not establish general ECharts configuration compatibility.

SVG is the core output. The [guide](https://github.com/vicanso/charts-rs/blob/241667b5c2b8e34bf36a4ba375b31fe34e195893/docs/guide.md) and [manifest](https://github.com/vicanso/charts-rs/blob/241667b5c2b8e34bf36a4ba375b31fe34e195893/Cargo.toml) document optional PNG, JPEG, WebP and AVIF export and Rust 1.88+. A UI consumes the generated SVG or image; no dedicated egui/Iced/GPUI adapter was verified in this review. SVG animation and CSS tooltips depend on the viewer; raster output is static. Raster features add encoding/font considerations, and upstream timings are not local measurements. Code is [Apache-2.0](https://github.com/vicanso/charts-rs/blob/241667b5c2b8e34bf36a4ba375b31fe34e195893/LICENSE).

## egui-mermaid

[egui-mermaid](https://github.com/ahcm/egui-mermaid), reviewed at [`f7a7153`](https://github.com/ahcm/egui-mermaid/blob/f7a715398b041815cf372e53b3331e4d5db603a9/README.md), parses Mermaid text and paints through egui without a Mermaid JavaScript runtime or webview. Its [API](https://github.com/ahcm/egui-mermaid/blob/f7a715398b041815cf372e53b3331e4d5db603a9/src/lib.rs) supplies `diagram()`, geometry-only layout access and default-feature Markdown integration using `egui_commonmark`.

The [parser dispatch and implementation](https://github.com/ahcm/egui-mermaid/blob/f7a715398b041815cf372e53b3331e4d5db603a9/src/layout.rs) cover flowcharts, sequence diagrams, pie/Gantt charts, journeys and git graphs; state/class/ER/mindmap diagrams use flowchart rendering. This is a syntax subset: flowchart styling/click directives and sequence activation/box directives are skipped. Unsupported kinds display source with a notice in Markdown; syntax within accepted kinds can also be simplified or omitted. Check actual diagrams rather than assuming Mermaid-wide compatibility.

The [manifest](https://github.com/ahcm/egui-mermaid/blob/f7a715398b041815cf372e53b3331e4d5db603a9/Cargo.toml) declares Rust 1.95 and egui 0.36; align application versions. The README advertises wasm32 support, without local platform verification here. License metadata differs: README and shipped [LICENSE-MIT](https://github.com/ahcm/egui-mermaid/blob/f7a715398b041815cf372e53b3331e4d5db603a9/LICENSE-MIT) say MIT; the manifest says `MIT OR Apache-2.0`, but no Apache license file was found in the inspected tree.

## Lini

[Lini](https://github.com/monfa-red/lini) is a Rust compiler/library for its own text diagram DSL. The [README at `5dab4bf`](https://github.com/monfa-red/lini/blob/5dab4bf4da1d567f2f06425b565840b7d9776417/README.md) documents flowcharts, trees/mindmaps, tables, ER schemas, sequences, charts and technical drawings, with themeable SVG output. It lists mdBook, rehype, Astro, remark and Wasm integrations; the Obsidian plugin is community-maintained.

The [CLI](https://github.com/monfa-red/lini/blob/5dab4bf4da1d567f2f06425b565840b7d9776417/src/main.rs) emits SVG or an HTML wrapper, while [Wasm bindings](https://github.com/monfa-red/lini/blob/5dab4bf4da1d567f2f06425b565840b7d9776417/crates/lini-wasm/src/lib.rs) expose browser compilation. The [specification](https://github.com/monfa-red/lini/blob/5dab4bf4da1d567f2f06425b565840b7d9776417/SPEC.md) defers general graph/DAG layout, native PNG/WebP export and animation. RTL/bidi shaping is unsupported; embedded fonts target browsers, and static outlined text has a bundled-character-set limit. Its language is separate from Mermaid. The compiler is [MIT licensed](https://github.com/monfa-red/lini/blob/5dab4bf4da1d567f2f06425b565840b7d9776417/LICENSE), with separate [font and icon notices](https://github.com/monfa-red/lini/tree/5dab4bf4da1d567f2f06425b565840b7d9776417/LICENSES).

For animated diagrams and narrated motion graphics, see [Make videos with code](making-videos.md).
