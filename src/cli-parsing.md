# Parse command-line arguments

Start a new Teamy CLI from [teamy-rust-cli](rust-cli-template.md). Add its command to the typed schema and dispatch path before considering another parser. Read the dependency revision the project actually pins; current upstream documentation can describe different behavior.

Facet describes Rust types and their fields. Figue uses those descriptions to construct a CLI schema, parse arguments, generate help and produce typed values. In the template, `Cli` flattens `GlobalArgs` and `FigueBuiltins`, then selects an explicit `Command` enum. Named, positional and subcommand annotations express the surface. Doc comments supply explanations.

Grounding: [the public template schema at `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/mod.rs) and [entry point](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/lib.rs).

## Find the parser component for the problem

The local template currently pins a public Facet/Figue fork revision, `23ad4dcd0`. These implementation links describe that dependency, not the older public template's dependency pin:

| Change you need | Read this implementation |
| --- | --- |
| Argument annotations and type-to-schema mapping | [Facet schema construction](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/schema/from_schema.rs). |
| Token handling and strict parsing | [CLI input layer](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/layers/cli.rs). |
| Input sources and precedence | [Builder](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/builder.rs), [driver](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/driver.rs) and [merge](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/merge.rs). |
| Generated help and completions | [Help](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/help.rs) and [completions](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/completions.rs). |
| Argument serialization and round trips | [Argument serialization](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/to_args.rs) and [arbitrary checks](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/figue/src/arbitrary_checks.rs). |

The template supplies OS arguments in strict mode. Figue's driver handles help, version and parser failures before normal logging and invocation. Parser tests can use `DriverOutcome::into_result()` to inspect those outcomes without exiting the test process.

## Enable input layers deliberately

The inspected fork merges configured file, environment and CLI values in increasing priority, then fills missing defaults. Objects merge recursively; arrays and scalars are replaced by higher-priority values. The consuming application must configure those sources.

The template enables no environment/file parser layer and currently ignores `RUST_LOG`. Stdin and `@file` expansion require command input semantics; they are not supplied automatically by a derive. See [logging](logging.md#filters-and-the-log-level-alias) and [CLI input conventions](good-cli.md#file-standard-input-and-argument-inputs).

Our local mover demonstrates why the hierarchy matters: `plan create` creates the plan; `plan action add` edits its contents. Root positional shorthand could reinterpret a typo as an operation. Reject that ambiguity. See [movement planning](moving-files.md).

## Reuse another integration with its boundaries intact

Cloud-Terrastodon's [`App::parse_from`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/app/src/lib.rs) separates parsing from process startup. It pins a different standalone Figue fork. Reuse that separation when useful; do not silently substitute its dependency versions or startup ownership for the template's.

Test the exact help spellings, aliases, nested commands, invalid inputs and input precedence you promise. [Good CLI](good-cli.md) records our intended behavior; source tests establish what a particular binary actually provides.
