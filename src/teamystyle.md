# TeamyStyle

TeamyStyle is our prescriptive guide for building software that is explicit, inspectable and recoverable. Link a section in a review when it explains a concrete design decision. The detailed chapters linked here own their implementation contracts and qualification limits.

**MUST** identifies established Teamy requirements. **SHOULD** identifies the default recommendation: explain a departure in the project decision record. A **proposal** requires review before adoption. Requirements describe the intended standard; they do not claim that every historical repository already complies. Apply changes deliberately and preserve existing work.

## Start from our working foundation

New Teamy CLI projects **MUST** use Rust and start from [teamy-rust-cli](rust-cli-template.md). Inspect the selected revision, retain the shared capabilities and record the source revision. A reference repository is evidence for a particular problem, not permission to replace the foundation wholesale. If the user asks for independent projects, **MUST** preserve that independence rather than silently merging their packages or lifecycles.

The [template chapter](rust-cli-template.md) distinguishes the public `7e62d72` baseline from local additions. The [code standards](code-standards.md) document locally qualified lint and writer changes. Copying an older snapshot does not adopt newer behavior. Changes to the starting point require an explicit recorded decision, rather than an accidental framework substitution.

## Make the CLI the primary interface

Commands **MUST** expose meaningful work inputs as first-class Figue arguments, with generated help from the typed schema. Use Facet as the main integration for types, parsing and serialization; **MUST NOT** introduce serde into new first-party Rust code. External dependencies may internally use serde; that is a dependency-review concern, not a claim that the entire dependency graph is serde-free.

A JSON file can be a versioned artifact or an optional interchange route. It **MUST NOT** be the only way to discover and supply ordinary work parameters. A caller should learn the operation from `--help`, without reverse-engineering a request payload.

Bad, hypothetical interface:

```text
example-tool execute --request request.json
# The file secretly chooses source, destination, overwrite policy and operation.
```

Good, hypothetical interface:

```text
example-tool plan action add --help
example-tool plan action add --plan plan.json --source input/a.txt --destination output/a.txt
example-tool plan validate --plan plan.json --output-format json
```

Here the plan is an explicit artifact, and the action's parameters remain discoverable. These are design examples, not installed commands. See [Good CLI](good-cli.md) and [argument parsing](cli-parsing.md) for stream, hierarchy, help and input contracts. Test the exact help a binary supplies: the documented parser snapshot has a leaf-help limitation.

## Separate arguments, requests and operations

Code **MUST** preserve the CLI → typed request → operation boundary. CLI types describe invocation syntax. Requests describe domain intent. Operations validate intent against current state and produce typed results. A library operation must not depend on a CLI struct, acquire process stdout, or parse its own argument list.

Use domain objects to distinguish identities, policies and states rather than repeatedly passing interchangeable strings and booleans. Validate invariants at construction or the relevant boundary; a newtype alone does not prove validity.

Bad contextual Rust fragment:

```rust,ignore
// CLI parsing, effects and rendering are entangled.
async fn run(args: Vec<String>) -> eyre::Result<()> {
    let source = &args[1];
    std::fs::rename(source, &args[2])?;
    println!("moved {source}");
    Ok(())
}
```

Good contextual Rust fragment, with the referenced types defined in their own files:

```rust,ignore
impl AddActionArgs {
    pub fn into_request(self) -> eyre::Result<AddActionRequest> {
        Ok(AddActionRequest {
            plan: PlanPath::try_from(self.plan)?,
            source: SourcePath::try_from(self.source)?,
            destination: DestinationPath::try_from(self.destination)?,
        })
    }
}

pub async fn add_action(
    request: AddActionRequest,
    store: &PlanStore,
    cancellation: CancellationToken,
) -> eyre::Result<AddActionReport> {
    cancellation.bail_if_cancelled()?;
    store.add_validated_action(request).await
}
```

This illustrates ownership and dependency direction, not a provided API or a complete persistence design. Parsing a path does not prove target identity or authorize a move. Keep those checks at the effect boundary. The [plan-record stages](plan-records.md#adopt-typed-records-gradually) explain the distinction between readable, typed and executable contracts.

## Put one type in one discoverable file

Rust code **MUST generally use one file per type**. Name the file after its type in snake case. Keep the type's inherent implementations and focused unit tests beside it. Use module declarations and re-exports to expose a coherent domain; do not turn `mod.rs` into a bag of type definitions.

Good illustrative layout:

```text
src/
  cli/
    mod.rs
    add_action_args.rs       # AddActionArgs and conversion impl
  domain/
    mod.rs
    add_action_request.rs    # AddActionRequest
    add_action_report.rs     # AddActionReport
    source_path.rs           # SourcePath, validation, unit tests
    destination_path.rs      # DestinationPath, validation, unit tests
  operations/
    mod.rs
    add_action.rs            # operation function and focused tests
```

A small `domain/mod.rs` exposes names without concealing their definitions:

```rust,ignore
mod add_action_request;
mod source_path;

pub use add_action_request::AddActionRequest;
pub use source_path::SourcePath;
```

Bad: an expanding `types.rs` or `mod.rs` containing all requests, reports, states and IDs. A reviewer should be able to find `SourcePath` by filename. Colocate trait implementations when coherence permits; integration tests can remain outside the type file when they exercise several components together.

Exceptions **MUST** be narrow and explained beside the affected code, for example generated bindings whose generator controls file layout. Convenience alone is not a blanket exception. Historical reference files sometimes contain several types; studying their parser or logging behavior does not adopt their layout.

## Keep one meaning across front ends

The CLI **MUST** remain a usable primary route. A daemon is optional; when present it **MUST** reuse the same typed requests, validation and operation semantics. Transport-specific connection and authentication handling belong at the adapter boundary. Do not duplicate business rules in an IPC handler or require a daemon for work that the project promises to run locally.

See [resident inference](resident-inference.md) for explicit daemon selection and its current boundaries. Cloud-Terrastodon's [public app integration](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/eadf23c349c50a8a745df3e206536f4a53a419bb/crates/app/src/lib.rs) separates parse-only use from process startup. Its [terminal ownership](terminal-ownership.md) and [async work](async-terminal-ui.md) are specific implementation references, not mandatory dependencies or universal daemon designs.

## Own output and preserve useful evidence

Operations **MUST** return typed domain results. Human and machine formats render the same result. The CLI owns stdout; reusable renderers receive a checked writer. Diagnostics and progress use structured `tracing` events with meaningful operation, phase and outcome fields. **MUST NOT** hide failures behind success logs or contaminate JSON stdout with progress.

Completed findings belong in the result; unrecoverable errors propagate through `eyre`. Follow [typed results and errors](output-shape.md), [logging](logging.md) and [the output lint policy](code-standards.md). Logging a request indiscriminately can disclose credentials or private content; choose fields deliberately.

Locks and receipts **MUST** retain applicable provenance: format/schema version, tool or artifact version, source repository URL, immutable revision or digest, and the operation/outcome identity needed to interpret the record. **SHOULD** record capture time and verification method where they affect reproducibility. State unknown provenance explicitly; a URL or checksum alone is not proof of authorship or trust.

## Justify every dependency

Dependencies **MUST** be justified by a named capability and reviewed in the actual pinned graph. Minimize dependencies; do not add a framework for a small helper or adopt every crate used by a reference project. Consider maintenance, licensing, platform behavior, build effects and the cost of owning the integration. Use [source lookup](source-lookup.md), [template dependency guidance](rust-cli-template.md#check-dependencies-and-validation) and [curated prior art](curating-prior-art.md).

For a concrete supply-chain example, this book's [bootstrap](https://github.com/TeamDman/teamy-docs/blob/ff7230176bf464670b0a9afa8f1cf4fff31d800f/README.md) pins mdBook's archive and checks a digest before extraction. Its receipt verifies the local executable against that download. The README explicitly limits the claim: agreement with official release metadata is not an independent signature. Apply the same precision to model weights, helper executables and protocol libraries; do not label a download trusted merely because it completed.

## Make effects durable and recoverable

Code **MUST** preserve dirty work and avoid blind destructive actions. Before changing valuable state, identify scope and current objects, expose the proposed effects, and validate assumptions again at execution. An inert plan does not grant authorization. A dry run **MUST** disclose its own effects and must not execute the proposed mutation.

Stateful operations **MUST** define partial progress, cancellation and recovery. A success receipt must correspond to the completed effect; a crash between mutation and receipt needs a documented reconciliation path. **SHOULD** use staged writes, atomic replacement where supported, durable journals or idempotent operations according to the failure model. These are design options, not a claim that rename or a transaction makes every external effect atomic.

Pass cancellation through loops, workers and waits, and observe it at safe boundaries. Cleanup must preserve already committed state. Follow [cooperative cancellation](cancellation.md), [file movement](moving-files.md), [plan records](plan-records.md) and [repository hygiene](repository-hygiene.md). Describe power-loss and cross-volume limits when relevant rather than promising universal rollback.

Human interaction deadlines **MUST** reflect realistic human timescales, often minutes or hours; do not impose arbitrary tiny timeouts on authentication, review or input. **SHOULD** distinguish those waits from network connection deadlines and worker shutdown budgets, with explicit cancellation and recoverable timeout outcomes.

## Review safety claims in their domain

The established general rule is to prove the effects and guarantees the program actually promises. The following are **proposed review requirements for VPN/torrent work**, not universal requirements for every CLI and not claims of implemented behavior:

- Define what “traffic stays inside the VPN” covers: application sockets, peer discovery, DNS, IPv4/IPv6, reconnects and startup/shutdown transitions. A successful tunneled HTTP request alone does not establish leak prevention.
- If fail-closed networking is adopted, test route changes, tunnel loss and restart while traffic is active; capture externally observable traffic as evidence. Define what elevated firewall changes persist after cancellation or crashes and how they are reconciled.
- Treat torrent metadata and peer input as untrusted. Review path containment, resource bounds and destination conflicts before writing files. Piece verification does not establish publisher identity or permission to overwrite local content.
- Record platform, topology and observation limits. An isolated fixture can establish behavior in that setup; it does not certify every host firewall, resolver or VPN provider.

[The networking survey](networking-survey.md) distinguishes reachability from authorization and records candidate adoption limits. Neither a reference survey nor a working prototype constitutes a security audit. Accept and record domain-specific guarantees before presenting them as product policy.

## Test claims and state the limits

Tests **MUST** exercise the actual claim with observable evidence. A serialization round trip does not prove authorization, recovery or network isolation. **SHOULD** start with a meaningful fixture, then broaden coverage when a requirement, failure or unresolved risk warrants it. Test invalid inputs, cancellation and partial effects when the contract depends on them.

Good: test that a failed writer propagates an error, that a stale plan cannot mutate its targets, or that the exact binary exposes promised nested help. Bad: test only a helper that mirrors the implementation, then claim the whole workflow works. See [code-standard qualification](code-standards.md#run-the-policy-checks-explicitly) and [what the book validates](plan-records.md#know-what-the-book-validates).

These Rust fragments are contextual illustrations, explicitly skipped by rustdoc; this chapter does not claim they compile. The book build and local-link checks validate navigation, not program behavior. Public documentation **MUST** respect the [publication boundary](publication.md); publishing a description does not publish the referenced local source.

## Specify observable CLI contracts

The template explains **how** we implement these requirements; it does not replace their specification. A copied framework, passing derive or successful build does not prove the resulting binary satisfies them. These are required behavior for new and deliberately updated Teamy CLIs; historical implementations and known gaps remain labelled below.

### Keep result and diagnostic controls independent

`--output-format` **MUST** select only the command result on stdout. Supported template values are `text`, `json` and `csv`; each command must document any shape restrictions. When omitted, the inspected template selects text for terminal stdout and JSON for redirected stdout. An explicit selection wins over terminal detection.

`--log-format` **MUST** independently select stderr diagnostics: `text` by default, or `ndjson` (one JSON event per line). Choosing JSON results must not imply NDJSON logs, suppress stderr or change the operation. Likewise, choosing NDJSON logs must not change result framing. Do not merge streams when validating either format.

The optional `--log-file <path>` sink **MUST** be independently controlled by `--log-file-format` (default `ndjson`) and `--log-file-filter` (default `debug`). It appends; omission creates no file sink. Stderr filter precedence is explicit `--log-filter`/`--log-level`, then `--debug`, then `RUST_LOG`, then `info`. A stderr filter of `off` must not disable a requested file. Reject invalid filters before creating that file. A file sink is not a promise of rotation or crash durability.

Acceptance examples, using a synthetic tool and a command that emits a known diagnostic:

```powershell
example-tool --output-format json --log-format text inspect --source fixture.txt > result.json 2> diagnostics.txt
example-tool --output-format text --log-format ndjson inspect --source fixture.txt > result.txt 2> diagnostics.ndjson
example-tool --output-format json --log-format ndjson --log-filter off --log-file capture.ndjson inspect --source fixture.txt > result.json 2> quiet.txt
```

In the first case, parse stdout as the declared result and require the fixture diagnostic only in stderr. In the second, require text results and parse each diagnostic line independently as JSON, with no ANSI escapes. In the third, require empty stderr while the independently admitted file event remains present. Check exit status first; a partial stdout write is not a completed result. Test all supported combinations, explicit formats under redirection, and failing writes/flushes. See [independent sinks](logging.md#local-independent-sink-rollout) and [typed output](output-shape.md).

### Put interactive screens on stderr

New Teamy CLI TUIs **MUST** render through stderr, reserving stdout for the command's result. Check stderr's terminal status, not stdout's. If stderr is redirected, reject interactive mode clearly or use an explicitly documented noninteractive route; do not write screen escapes into a log file. Result formats do not serialize the live screen. The policy for `--log-format ndjson` with a TUI must be explicit: use a separate terminal/log destination or reject the combination before entering raw mode, rather than mixing screen controls into NDJSON.

While the screen owns the terminal, route human logs through its guarded buffer/view; keep file events flowing independently. Restore terminal modes and leave the alternate screen before returning the selected value or final error. Cancellation, initialization failure and nested ownership need the same cleanup contract. **Acceptance:** redirect stdout to a file during a real terminal session, interact and cancel, and require no escape bytes in the result, restored input behavior and visible diagnostics without screen corruption. Captured subprocess output alone does not qualify a TUI.

Cloud-Terrastodon's stderr picker supplies prior art. Its historical stdout object browser is an observed implementation, not the new stream requirement. See [terminal stream choices](terminal-interfaces.md#decide-which-stream-owns-the-screen) and [terminal ownership](terminal-ownership.md).

### Make help and build identity actionable

`--version` **MUST** succeed without invoking domain work or opening a requested log file. Include executable/package identity, package version, repository URL, branch (or detached/tag identity), revision, tracked-worktree status and build timestamp. The inspected local template formats the version body as:

```text
<package-version> (repo <repository-url>, branch <branch>, rev <short-revision>, worktree <clean|dirty|unknown>, built <local-build-time>)
```

Figue supplies the executable presentation around that body. Unknown source/build metadata must remain visibly unknown. The template captures a UTC build instant and renders it in local time; timestamps need their zone. Its dirty check excludes untracked files, so `clean` is not proof of a reproducible release or an empty worktree. Short revisions identify the display; retain full revisions/digests in release receipts. Sanitize credential-bearing remote URLs before embedding them, and set the generated project's own identity instead of retaining the template's URL.

Root and nested `--help` **MUST** succeed without running the command, authenticating, loading a model, changing inspected objects or initializing a log file. Generated help must expose command purpose, usage, arguments, defaults, available subcommands and applicable shared options. It must explain effects and input/output contracts directly or link their detailed documentation. Include the rich version/build identity and implementation source references: the inspected template requests `include_implementation_source_file(true)` and GitHub links tied to its revision. Replace that repository coordinate in generated projects; a source link to the wrong project is misleading.

**Acceptance:** inspect root and nested help, check every promised argument/default and source link, and run help/version with a nonexistent log destination and otherwise invalid work arguments. No destination should appear and no operation should execute. Test how help flags interact with `--` and opaque values; content containing `--help` is not automatically a help request. Compare reported identity to the actual build inputs. Existing version tests check version/repository/branch/revision/worktree and a `built` label; they do not independently establish timestamp freshness, source-link correctness or all nested-help fields. The documented pinned parser's missing leaf global options remains a gap to resolve, not an exception that weakens this contract.

### Keep remote work visible to the invoking console

Elevated and daemon work **MUST** deliver relevant diagnostics back to the invoking client's console/logging path while preserving result stdout. Preserve source level, target, location, request/correlation identity and useful fields; distinguish source time from receipt time. Client-side rendering must honor the selected diagnostic format and terminal ownership. Do not replace structured source events with an opaque JSON string merely to display them.

The invoking process **MUST** observe remote completion and report failure. Elevation refusal or attachment/transport failure cannot become silent success. Define and propagate cancellation to the actual remote operation, wait for an acknowledged outcome, then drain or explicitly report lost diagnostics. Cancelling one request must not silently stop an independently owned daemon. Shutdown/log-drain budgets are separate from human UAC/review waits.

**Acceptance:** trigger a known INFO/WARN event in direct, elevated and daemon routes; capture stdout, stderr and status separately; repeat with NDJSON, cancellation and remote failure. Require the original request identity in forwarded events, no duplicate replay, correct nonzero failure and a following healthy request. Exercise a slow or disconnected log receiver and check loss reporting without blocking cancellation indefinitely. A Windows console-only test does not establish redirected-pipe or every PTY behavior.

MFT's [elevation and daemon forwarding](logging.md#elevation-and-daemon-log-forwarding) implements two distinct mechanisms: console reattachment with wait/exit propagation, and typed RPC events replayed through the client subscriber. Its reattachment rebinds streams to `CONOUT$`/`CONIN$`; that does not preserve arbitrary shell redirects. Its bounded history and broadcast can lose logs, and forwarder shutdown is bounded. Source inspection does not establish complete elevated cancellation. These are implementation limits that new integrations must address and test. The local [request-scoped inference route](resident-inference.md) is further prior art; it is not a durable audit channel.

## Keep the template's decisions reviewable

Use the [template decision and enforcement inventory](teamystyle-template-audit.md) to review the actual Cargo, Clippy, build, path, parser, error, logging, output and test choices. It maps each decision to its reason, source and acceptance evidence, including missing enforcement. Copying a lint configuration without enabling its lint does not establish a gate; banning direct serde APIs does not audit transitive dependencies.

## Review checklist

- [ ] New Rust CLI starts from the recorded template revision and preserves its shared contracts.
- [ ] First-class Figue arguments and generated help expose the work; Facet is the main integration and new first-party code introduces no serde.
- [ ] CLI, typed request, operation and result boundaries are clear; domain invariants have an owner.
- [ ] Each type has a discoverable file, colocated impl/tests and deliberate re-exports; exceptions are narrow and explained.
- [ ] Optional daemon and other adapters share domain semantics; requested project independence is preserved.
- [ ] Output and log formats are independent; stderr TUIs preserve stdout and restore the terminal.
- [ ] Root/nested help and rich version metadata identify the actual build without domain side effects.
- [ ] Elevated/daemon diagnostics reach the invoking console; completion, failure, cancellation and loss are tested.
- [ ] Cargo/Clippy decisions have matching enabled gates; direct API checks are not described as a transitive dependency audit.
- [ ] Locks/receipts identify versions, source repository and immutable provenance; dependencies have a stated purpose.
- [ ] Dirty work is preserved; effects, authorization, partial state and recovery are reviewable.
- [ ] Human waits use realistic budgets; domain-specific safety guarantees are accepted and tested explicitly.
- [ ] Validation supports the stated claims, limitations remain visible, and publication has not been inferred from a local build.
