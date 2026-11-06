# Render text with Liquid templates

[liquid-rust](https://github.com/cobalt-org/liquid-rust) implements the Liquid templating language in Rust. Use it when a template should select or format supplied data into text. The supported tags, filters, variables and resource limits belong to the application profile; a generic Liquid example does not prove that every host accepts it. Template output still requires validation appropriate to its destination.

## Project source across SFM versions

At public SFM revision [`bd7390c`](https://github.com/TeamDman/SuperFactoryManager/blob/bd7390cf855fdae5ca6f1df1d18b2e7591fd6692/platform/cli/sfm-propagate-changes/Cargo.toml), `sfm-propagate-changes` pins the Rust `liquid` crate to `=0.26.11`. This is the source-projection tool's dependency, not a claim that the Minecraft mod executes Liquid at runtime. It helps produce Java variants for selected version and feature contexts; see [migration workflows](migration-workflows.md).

SFM's [directive scanner](https://github.com/TeamDman/SuperFactoryManager/blob/bd7390cf855fdae5ca6f1df1d18b2e7591fd6692/platform/cli/sfm-propagate-changes/src/source_projection/directive_scanner.rs) recognizes deliberate whole-line controls and keeps ordinary Java chunks as opaque values. That matters because Liquid delimiters can collide with valid Java syntax. The [projection renderer](https://github.com/TeamDman/SuperFactoryManager/blob/bd7390cf855fdae5ca6f1df1d18b2e7591fd6692/platform/cli/sfm-propagate-changes/src/source_projection/mod.rs) builds a Liquid skeleton, validates referenced conditions and renders with typed context. The [context bridge](https://github.com/TeamDman/SuperFactoryManager/blob/bd7390cf855fdae5ca6f1df1d18b2e7591fd6692/platform/cli/sfm-propagate-changes/src/source_projection/context.rs) reflects Facet values into the limited Liquid object shapes needed here, without a JSON/Serde round trip. Directive-free input follows the scanner's identity path; later project-specific substitutions remain a separate step.

This scanner, validation and typed bridge are SFM-specific additions, not guarantees supplied by liquid-rust. A successful render is not a compile, semantic or runtime check. Compare generated variants with their references and run the checks required for the target context.

## Other Teamy uses

Cloud Terrastodon's [picker command](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/entrypoint/src/cli/command/pick/pick_command_cli.rs) identifies liquid-rust as an output-formatting option. The separate local prompt implementation has its own explicit profile, bounds and image tag; see [prompt templates](image-prompts.md). Do not assume that SFM directives, picker formatting and prompt image tags are interchangeable.

Treat templates, data and rendered output as distinct trust boundaries. Limit what a host exposes, avoid re-parsing inserted data as template source, bound work and output where needed, and escape or validate for the target format. Liquid rendering is not permission to perform the resulting action; see [plan records](plan-records.md).
