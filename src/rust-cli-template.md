# Start with the Rust CLI template

New Teamy CLI projects are expected to use Rust and [teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli). Start from the template and understand its existing capabilities before replacing them or adding another framework. A departure should name the requirement the template cannot meet and record the decision.

This chapter describes the implementation at public revision [`7e62d72`](https://github.com/TeamDman/teamy-rust-cli/tree/7e62d72bbbf3ea1b302e48008da565e92d4b6c93), checked on 1 October 2026. Local additions are labelled separately. The reasons below explain our preferred use of the code; they are design judgments grounded in the implementation.

## Follow one command through the program

Read these four points before building your first command:

1. [`src/lib.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/lib.rs) installs error reporting and cancellation, parses the command, initializes logging, invokes it and emits its result.
2. [`src/cli/mod.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/mod.rs) defines the command enum and dispatches work on a Tokio runtime.
3. [`src/cli/global_args.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/global_args.rs) defines shared options, including logging, output format and stop conditions.
4. [`src/cli/output.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/output.rs) renders the typed result and handles stdout writes.

Help, version and parser errors are handled by Figue's driver before logging and command dispatch. Normal work follows this route:

```text
OS arguments -> Figue + Facet schema -> command enum -> command's invoke()
                                                        |
                                                    CliOutput
                                                        |
                                              one stdout renderer
```

Keeping this route shared lets each new command concentrate on its domain behavior while using the same error, cancellation and output contracts.

## Capabilities to preserve and understand

| Capability in the template | Why we use it | Implementation |
| --- | --- | --- |
| Facet structs and enums describe arguments and commands; Figue parses OS arguments in strict mode. | Keep the command hierarchy explicit and let the parser report invalid inputs. Reflection also supports rendering results. | `Cli`, `Command` and `GlobalArgs`; [parser setup](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/lib.rs). |
| Figue builtins provide help, version and completions. Help can link to implementation source. Schema export is a parser facility for configuration roots; this template defines none. | Make a tool explain its own surface and identify the source behind that explanation. Verify the installed parser's actual options. | `FigueBuiltins` in [the CLI schema](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/mod.rs). |
| Version output includes package version, repository, branch, revision, worktree state and build time. | Two binaries with the same package version can still behave differently. Preserve build provenance. | [`build.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/build.rs) and `version()` in `src/lib.rs`. |
| `CliOutput::facet` renders a typed value as text, JSON or CSV. Omitted format selects text for a terminal and JSON for redirected stdout. | Let people and programs inspect the same result. CSV still needs a compatible value shape. | [Output renderer](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/output.rs); [output contract](output-shape.md). |
| `eyre::Result` carries invocation, serialization and output-write errors; `color-eyre` formats reports. | Preserve context and return failure when a command cannot complete. Domain findings remain in typed results. | `main()` and `CliOutput::emit`; [typed results and errors](output-shape.md). |
| `teamy-cancellation` installs Ctrl+C handling and offers duration, span and log-message stop conditions. | Use one cancellation mechanism throughout the work. Each new worker must still observe the token. | `StopAfterArgs` and entry-point checks; [cancellation](cancellation.md). |
| `tracing` and `tracing-subscriber` provide terminal logs, optional NDJSON files and optional Tracy profiling. | Keep operational context separate from command results and reuse instrumentation across destinations. | [`src/logging_init.rs`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs); [logging](logging.md). |
| Home and cache paths have explicit environment overrides and platform defaults. | Make storage configurable without embedding a person's home directory. Reading a path and creating it are separate effects. | [`src/paths`](https://github.com/TeamDman/teamy-rust-cli/tree/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/paths). |

Put each subcommand in its own directory, with an implementation module such as `inspect_cli.rs` re-exported by `mod.rs`. This is the template's [repository convention](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/AGENTS.md). Return the command's result through `CliOutput`; emit operational messages through `tracing` rather than printing them into stdout.

## Initialize deliberately

The template's `init` command builds a copy plan before writing destination files. Without `--force`, conflicting generated files reject the plan. Existing destination `README.md` and `LICENSE` are preserved. This is a scaffolding command with filesystem effects, not an inert plan export. See [the initializer](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/init/init_cli.rs).

Its source is the directory embedded as `CARGO_MANIFEST_DIR` when the binary was compiled. It recursively copies that source tree, including untracked ordinary files, with a small exclusion list: root `.git`, root `target`, the legacy initializer skill and script. It does not use a Git-tracked allowlist. Inspect the source checkout before using it; a clean, reviewed source snapshot is the preferred starting point.

After reviewing that source and choosing a new destination, this command creates the scaffold:

```powershell
# Run from the reviewed template checkout; this writes ../new-teamy-cli.
cargo run -- init ../new-teamy-cli
```

Replace the package identity, CLI description, example commands, help-source repository URL, home/cache environment names and directory names. Update the logging target from `teamy_rust_cli` to the new crate's Rust identifier. The `TODO(template)` markers identify these integration points. Copying a template does not automatically complete them.

## Check dependencies and validation

Read `Cargo.toml`, `Cargo.lock` and `check-all.ps1` in the source snapshot you are adopting. A fork revision is part of the project's behavior. Do not assume a copied CLI uses the latest registry release or that updating its template also updates existing projects.

The [validation script](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/check-all.ps1) runs nightly formatting, Clippy with all features and warnings denied, an all-feature build, and tests with Tracy excluded. Its formatting step changes files despite being labelled a check. Review the working tree afterward; use `cargo fmt --all -- --check` for a read-only formatting gate. Test new command behavior with a meaningful fixture and inspect stdout, stderr and status separately.

## Observed gaps are work items

The audit found limits that a new project must address when it relies on them:

| Area | Observed limit | Consequence |
| --- | --- | --- |
| Logging environment | The entry point has no environment parser layer and logging does not read `RUST_LOG`, despite existing comments saying it does. `--debug` does not set a backtrace environment variable. | Use explicit filters; [logging](logging.md#filters-and-the-log-level-alias) records the behavior. |
| Help | The currently pinned local Figue parser displays global options in root help but omits them from leaf-command help. | Keep root help discoverable and test subcommand help when changing the parser. |
| Cancellation | Dispatch and result boundaries check the token; the example commands do not pass it through all internal work. | Add cooperative checks inside new loops and workers; an installed handler is insufficient. |
| Cache | `cache clean` currently returns no output without invoking the existing cleanup helper. | Do not treat the displayed command name as proof of cleanup behavior. See [its implementation](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/cli/cache/clean/cache_clean_cli.rs). |
| Windows | A resource manifest is embedded, but it has no `longPathAware` declaration. UTF-8 warning calls occur before the tracing subscriber is installed. | Do not promise long-path compatibility or visible startup warnings from those helpers alone. See [Windows path length](windows-paths.md#the-rust-cli-templates-manifest). |

Input from stdin, `@file` expansion, extra help spellings and resumable execution remain command or framework features to implement and verify. They are useful [CLI requirements](good-cli.md), but reflection and a template do not supply their semantics automatically.
