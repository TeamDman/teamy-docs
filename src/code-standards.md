# Enforce our Rust code standards

Use [teamy-rust-cli](rust-cli-template.md) for a new CLI and keep its code policy alongside its implementation. A useful rule explains its purpose, has an executable check and names any narrow exceptions. Review output ownership as well as passing the linter.

This chapter records additions qualified in the local template and `teamy-llm-service` on 3 October 2026. They are committed locally as `TeamDman/teamy-rust-cli` revision `99460d927c745a43285b0c5206fc2d42cec33ebc` and `TeamDman/teamy-llm-service` revision `89ec9907d0d525703cec9c7bd807b8e2f0931521`; these source commits have not been published. The template was tested with its existing local dependency and logging updates preserved. They are not part of its older published [`7e62d72` baseline](https://github.com/TeamDman/teamy-rust-cli/tree/7e62d72bbbf3ea1b302e48008da565e92d4b6c93), and copying an earlier template does not adopt them.

## Keep reusable code independent of standard streams

Command implementations return [typed results](output-shape.md). Renderers receive a `std::io::Write` sink. The process entry point chooses stdout, checks whether that destination is a terminal and passes both decisions to the renderer.

The local template's `CliOutput::emit_to(writer, requested_format, output_is_terminal)` follows this contract. `src/lib.rs` acquires stdout; `src/cli/output.rs` renders without acquiring an operating-system stream. A test can supply a `Vec<u8>` or a writer that fails. The same code can serve a pipe, file or terminal.

Operational events use [tracing](logging.md). Their destination belongs to the subscriber, which applies filters, NDJSON output and any terminal buffer. Replacing a command result with a log event would lose the command's output contract.

The impact assessment found 13 Cargo-protocol `println!` calls and one logging-bootstrap `eprintln!` in the template, with no print macros in its command implementations. `teamy-llm` had no print macros in production `src/`; its eight build-script calls become one Cargo instruction helper. Six print calls in other packages' standalone numerical examples are outside this CLI adoption. These boundaries make the rule useful without exempting ordinary commands.

| Kind of output | Owner and mechanism |
| --- | --- |
| Command result | CLI boundary chooses the writer; rendering uses checked `Write` operations. |
| Progress and diagnostics | Emit structured `tracing` events; the logger chooses their destinations. |
| Worker protocol | Process transport chooses pipes; protocol code writes complete framed records to its supplied writer. |
| Interactive screen | A terminal owner holds the Ratatui backend and performs checked screen writes and restoration. |

Keep `write!` and `writeln!` available for supplied writers, strings and formatters. A [Ratatui backend](terminal-interfaces.md) is a valid writer. [Terminal ownership](terminal-ownership.md) controls when screen writes and human logs may reach the same terminal.

## Reject plain print macros

The local policy denies `print!`, `println!`, `eprint!` and `eprintln!` through `clippy::disallowed_macros`. These macros select process streams inside the call. They can bypass logging filters, file logs or terminal ownership.

Two files provide different parts of the check:

```toml
# Cargo.toml: choose the lint level.
[lints.rust]
unfulfilled_lint_expectations = "deny"

[lints.clippy]
disallowed_macros = "deny"
```

```toml
# clippy.toml: identify the macros and explain replacements.
disallowed-macros = [
  { path = "std::print", reason = "Use tracing for diagnostics or a supplied Write sink for results." },
  { path = "std::println", reason = "Use tracing for diagnostics or a supplied Write sink for results." },
  { path = "std::eprint", reason = "Use tracing so filters, file logs and terminal ownership apply." },
  { path = "std::eprintln", reason = "Use tracing so filters, file logs and terminal ownership apply." },
]
```

These excerpts describe the output rule; retain the template's other configured rules too. [Clippy's macro lint](https://rust-lang.github.io/rust-clippy/master/index.html#disallowed_macros) requires configured entries. [Its configuration reference](https://doc.rust-lang.org/clippy/lint_configuration.html#disallowed-macros) defines their paths and reasons. A TOML list alone does not establish a failing validation gate.

Clippy searches for its configuration from the configured directory or package directory, then walks parent directories. An explicit `CLIPPY_CONF_DIR` can change that selection. Verify the file used by the gate, particularly in a workspace. See [Clippy configuration discovery](https://doc.rust-lang.org/clippy/configuration.html).

## Keep exceptions narrow and accountable

Use `deny`, which rejects violations while permitting a justified local expectation. `forbid` would prevent those local exceptions. An `#[expect(..., reason = "...")]` suppresses an expected lint; if the violation disappears, `unfulfilled_lint_expectations` exposes the stale exception. See [Rust lint expectations](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-expect-attribute).

The template has two specific exceptions:

| Location | Reason |
| --- | --- |
| `build.rs` Cargo instruction helper | [Cargo consumes build-script stdout](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script) as a compilation protocol. Sending those instructions through tracing would break it. |
| `src/logging_init.rs` logging-failure helper | Subscriber installation failed. The bootstrap diagnostic cannot depend on the subscriber it failed to install. |

Each helper contains one intentional print macro. Put its expectation on that helper function, not on the macro statement: the compile fixtures showed a statement attribute was unused and did not suppress this Clippy lint on the tested toolchain. Neither exception grants permission to print throughout its module. The bootstrap exception also does not guarantee reliable log delivery: the current template still returns success after subscriber installation fails. Treat that behavior as a separate [logging limitation](logging.md#terminal-logs-ndjson-and-tracy).

## Make output failure observable

Check writes and flush at the declared output boundary. Propagate an error through `eyre` when serialization, writing or flushing prevents completion. A broken pipe must not quietly become a successful complete result.

The local template tests captured JSON, CSV and terminal formatting with supplied writers. Its failure fixtures cover partial payload writes, newline writes and final flushes. A completed write cannot retract bytes already received by a caller. Callers must inspect process status before treating partial stdout as a valid result.

The pinned Facet CSV implementation accepts one flat struct row and rejects a top-level vector of rows. Tests cover the supported row and verify that a rejected shape propagates its serialization error before writing or flushing. Writer injection preserves this existing format constraint; it does not broaden serializer support.

Terminal cleanup has a different boundary. Attempt restoration before releasing ownership, including error and panic paths. A failed restoration must not grant a new owner access to a terminal whose state is uncertain. See [terminal restoration](terminal-ownership.md).

## Run the policy checks explicitly

The local template's validation gate uses these commands:

```powershell
rustup run nightly -- cargo fmt --all -- --check
cargo clippy --all-targets --all-features --no-deps -- -D warnings
./check-lint-policy.ps1
```

Formatting checks do not rewrite source. Clippy checks the enabled targets and features. The policy script uses isolated compile fixtures to verify rejection of ordinary, qualified and renamed print macros, including build, test and example targets. Positive fixtures exercise writer output and the scoped bootstrap and Cargo exceptions; a stale expectation must fail. It derives the output directory from Cargo metadata so workspace-member fixtures stay with the workspace's build artifacts.

Qualification passed the full template `check-all.ps1`: formatting, strict Clippy, all 14 policy fixtures, the all-feature build and 18 tests, including nine writer-contract tests. Applying the same fixture runner to the `teamy_llm_cli` manifest passed all 14 cases; that package also passed release, offline, locked, all-target Clippy through the existing CUDA build wrapper. These checks establish the inspected output policy, not a security boundary or support for every future feature combination.

`cargo build` does not run Clippy. Compiler lints and Clippy lints have different checks. In a workspace, `[workspace.lints]` also needs each participating package to opt in with `[lints] workspace = true`; defining a root policy alone is insufficient. See [Cargo lint inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table).

## Know the adoption boundary

The local template adopts this output policy across its package. `teamy-llm-service` adopts it only in the first-party `teamy_llm_cli` package, using package-local Cargo lint settings and `clippy.toml`. Its numerical examples, other core packages and vendored runtime are outside this adoption. Do not infer that every existing Teamy project has the policy.

The lint rejects the configured macros. It does not reject `write!(stderr, ...)`, stream acquisition or arbitrary native output. Review where code obtains stdout, stderr and terminal handles. Figue's help and parser diagnostics, Rust's error termination and dependencies retain their own output behavior. The lint is a code-quality gate, not an operating-system access boundary.

The broader template lint configuration closely matches [Microsoft's static verification guidance](https://microsoft.github.io/rust-guidelines/guidelines/universal/index.html#M-STATIC-VERIFICATION). That guidance is useful inspiration for maintaining the policy; the match does not prove why its historical configuration was chosen.

[Microsoft's logging guidance](https://microsoft.github.io/rust-guidelines/guidelines/libs/resilience/index.html#M-LOG-NOT-PRINT) directs operational diagnostics through telemetry while permitting intentional CLI stdout. Our template keeps that distinction and adds an explicit-writer convention for command results. Its recommendation to [use reasoned lint expectations](https://microsoft.github.io/rust-guidelines/guidelines/universal/index.html#M-LINT-OVERRIDE-EXPECT) also informs our narrow exceptions.

Grounding in the matching local checkouts: the template's `Cargo.toml`, `clippy.toml`, `build.rs`, `src/lib.rs`, `src/cli/output.rs`, `src/logging_init.rs`, `check-all.ps1` and `check-lint-policy.ps1`; the service's `crates/teamy_llm_cli/Cargo.toml`, `clippy.toml`, `build.rs`, `src/interactive/worker.rs`, `tui.rs` and `terminal_output.rs`. The local source identities above distinguish these validated additions from the published template baseline. Publishing this explanation does not publish or release those source commits.
