# Plan file movement

When reorganizing existing files, record their exact proposed destinations before moving anything. Our current solution is the local `teamy-mover` planner. It represents ordered actions as versioned data and checks a simulated future state. It has no executor.

The implementation is still a local prototype as of 1 October 2026, with no published repository or release. The public book describes its reviewed design and synthetic command examples. Real plans can disclose filenames and storage locations; keep them in records for their intended audience.

## Name the plan and its actions separately

`plan create` creates the plan. `plan action add` adds an action inside it. This prevents two similar verbs from hiding which object is being changed.

| Object | Current commands |
| --- | --- |
| Plan | `plan create`, `plan list`, `plan show`, `plan dry-run` |
| Action | `plan action add`, `plan action list`, `plan action show`, `plan action remove` |

Each move records an action ID, source, exact destination and reason. Relative input paths resolve against the plan's existing base directory; stored paths are absolute. A destination includes the new filename. It does not mean “place this source inside that directory.” Parent creation and wildcard expansion are not implicit.

From the local mover checkout, this example writes plan metadata only. Replace `<existing-root>` with an approved root containing `existing.txt`:

```powershell
$planFile = "review.plan.json"
cargo run -- plan create --plan-file $planFile `
  --name "Review" --base-dir "<existing-root>"
cargo run -- plan action add --plan-file $planFile `
  --source existing.txt --destination desired.txt `
  --reason "Place the existing file in its reviewed location"
cargo run -- plan action list --plan-file $planFile
cargo run -- plan action show --plan-file $planFile --action-id 1
cargo run -- --output-format json plan dry-run --plan-file $planFile
```

Root positional shorthand such as `teamy-mover src.txt dest.txt` is rejected. An unknown subcommand must not become an accidental move. See [argument parsing](cli-parsing.md).

## Check the future state in order

The planner checks metadata and simulates each valid action. `A -> B`, followed by `B -> C`, can be valid even when `B` does not exist yet. Moving an occupied destination away first can make it available to a later action. The report records those dependencies by action ID.

It reports missing sources, occupied exact destinations, missing parents, repeated source use and unsafe containment. Invalid actions do not alter the simulated state. A completed check returns exit 0 and typed fields such as `ready`, `valid`, `depends_on` and `issues`; an unrecoverable error propagates through `eyre`. See [output shapes](output-shape.md).

A successful preview is an observation, not approval or a guarantee of execution. This prototype does not lock or hash source objects, measure free space or prove permissions. Reparse points and symlinks are conservatively rejected. Cross-volume movement and robust filesystem identity remain executor design work.

## Preserve the editing history

Action IDs stay stable when another action is removed. New actions do not reuse removed IDs. Removing an action edits the plan and can invalidate later dependencies; run the preview again.

```powershell
cargo run -- plan action remove --plan-file review.plan.json --action-id 1
```

The example removes plan metadata, not a source file. Use the ID returned by `plan action list`.

Plan creation refuses an existing file. Edits take an exclusive sidecar lock, retain the previous JSON revision, write and sync a same-directory pending file, and replace the plan JSON. An interrupted edit can leave sidecars for an operator to inspect. This is metadata history, not resumable file execution or automatic recovery.

The local implementation boundaries are `src/planning.rs`, the `src/cli/plan/` command modules, and `schemas/move-plan-v1.schema.json`. The planner follows [the Rust CLI template](rust-cli-template.md) for typed output, tracing and command dispatch. Its command internals still need the cooperative checks described in [cancellation](cancellation.md).

## Read JSON from arguments, files or stdin

`plan action add` accepts either all three literal fields or one JSON `--input`, never both. The input can be literal JSON, a quoted `@file` reference or `-` for stdin. Read-only plan commands also accept `@file` and stdin; write commands require a literal plan path.

```powershell
cargo run -- plan action add --plan-file review.plan.json `
  --input '@examples/move-request.json'
Get-Content -Raw examples/move-request.json |
  cargo run -- plan action add --plan-file review.plan.json --input -
```

These are alternative input forms, not two steps to add the same action. UTF-8 JSON is limited to 16 MiB. Unsupported versions, unknown fields/operations and malformed requests fail before the plan update. This command-specific behavior implements the [CLI input conventions](good-cli.md#file-standard-input-and-argument-inputs); the base template does not add it automatically.
