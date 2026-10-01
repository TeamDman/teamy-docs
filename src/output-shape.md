# Typed results and errors

Return a typed result when a command completes. Use `eyre` to propagate an unrecoverable error. The process status tells a caller whether the command completed; the result tells them what it found.

## Results and human communication use separate streams

| Channel | Purpose |
| --- | --- |
| Standard output | The command's declared result, rendered in the chosen format. |
| Standard error | Human progress, notices, warnings and error diagnostics. |
| Exit 0 | The command completed and emitted its result, if the command declares one. |
| Nonzero exit | The command could not complete its contract. |

`--output-format json` governs stdout. It does not suppress stderr or require stderr to be JSON. An informative stderr message is compatible with successful JSON output. Scripts should inspect the process status and parse stdout separately; merging stderr into stdout can corrupt the JSON stream.

Specify whether stdout contains one JSON document or a stream of records. Keep logs, terminal control sequences and progress bars out of that stream. Text and JSON should project the same typed value rather than run different implementations of the operation.

Use [the template's logging layers](logging.md) for operational events. A separate NDJSON log file is an event stream, not the JSON command result or its schema.

## Findings belong in the result

A completed check can find a conflict. That finding is a useful result, not an unrecoverable failure to run the check. A caller should inspect a field such as `ready`, or match a documented outcome enum, instead of decoding special exit numbers.

For example, a shortened mover dry-run report can contain:

```json
{
  "version": 1,
  "ready": false,
  "read_only": true,
  "execution_available": false,
  "actions": [
    {
      "action_id": 1,
      "valid": false,
      "issues": ["exact destination is already occupied"]
    }
  ]
}
```

This is an excerpt, not the complete report schema. The check completed, so it exits 0. `ready: false` explains that the proposed moves cannot all succeed under the observations made. It does not approve execution.

When partial coverage is an intentional result, put its missing observations and limitations in the declared shape. Do not claim complete coverage. If the command cannot produce a valid result at all, return an error instead.

Use Rust structs and enums to define fields and outcomes. Facet lets the output layer render those values. Version persisted formats explicitly, and document compatibility for callers. Unknown fields, absent measurements and outcome variants should have defined meanings; consumers should not parse human diagnostic strings to discover them.

## Propagate unrecoverable errors through eyre

Malformed input, an unsupported schema, an unreadable required input or a failed output write can prevent a command from fulfilling its contract. Return `eyre::Result<T>` and use `?` to propagate those failures. Add context where it helps explain which operation failed.

The top-level entry point returns `eyre::Result<()>`. Return `Ok(())` only after invocation and output rendering succeed. Do not print an error and then return success. Do not attach a special failure code to an otherwise completed domain report.

Rust's `Result` implementation of `Termination` reports an error and returns a failure status when `main` returns `Err`. `eyre` supplies the contextual error report; it does not decide the exit status by itself. See the [eyre documentation](https://docs.rs/eyre/latest/eyre/) and [Rust Termination contract](https://doc.rust-lang.org/std/process/trait.Termination.html).

The [teamy-rust-cli template](https://github.com/TeamDman/teamy-rust-cli) provides a central output layer that renders typed values and propagates serialization and stdout write errors. The local mover follows the same pattern, with completed dry-run findings carried in its report.

## Cancellation and incomplete output

Cancellation observed before completion propagates as an error rather than pretending the operation completed. See [cancellation with teamy-cancellation](cancellation.md) for token propagation and cleanup boundaries.

A failure can leave partial stdout, especially if an output write fails. A nonzero exit means the caller must not treat captured stdout as a completed result, even if some bytes can be parsed. Check status before consuming the result.
