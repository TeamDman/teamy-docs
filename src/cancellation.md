# Cooperative cancellation

Pass a cancellation token through the work and check it at safe boundaries. Installing a Ctrl+C handler only requests cancellation. It does not interrupt every running function or future.

## Use the shared library

[teamy-cancellation](https://github.com/TeamDman/teamy-cancellation) provides `CancellationToken` and `CtrlCHandler` for Rust tools. The example below targets version 0.3.1 at public revision `7c77848e759079e21f2039b25b147d780807725b`. The local CLI template and mover pin that revision. The locator currently uses version 0.2.0, so optional facilities can differ.

Install the process-wide handler once at the command entry point. Installation returns `eyre::Result<CancellationToken>` and can fail. Pass the token, or a clone sharing its state, into each worker.

`request_cancel(reason)` requests a stop. `bail_if_cancelled()` returns an error when that token is cancelled. With the inheritance feature, a parent can propagate cancellation to a child token; cancelling the child does not cancel its parent.

Sources: [pinned library identity](https://github.com/TeamDman/teamy-cancellation/commit/7c77848e759079e21f2039b25b147d780807725b), [handler implementation](https://github.com/TeamDman/teamy-cancellation/blob/7c77848e759079e21f2039b25b147d780807725b/src/ctrlc_handler.rs), [token implementation](https://github.com/TeamDman/teamy-cancellation/blob/7c77848e759079e21f2039b25b147d780807725b/src/cancellation_token.rs).

## Check async and CPU boundaries

This complete illustrative example uses `eyre`, the library's `ctrlc` feature, and Tokio's `macros`, `rt` and `time` features. Its API calls were checked against the pinned source and local usage. The snippet compiled to Rust metadata against the existing pinned dependencies; it was not linked or executed.

```rust,no_run
use eyre::WrapErr;
use std::io::Write;
use std::time::Duration;
use teamy_cancellation::{CancellationToken, CtrlCHandler};

async fn total_numbers(cancel: CancellationToken) -> eyre::Result<u64> {
    cancel.bail_if_cancelled()?;
    tokio::time::sleep(Duration::from_millis(10)).await;
    cancel.bail_if_cancelled()?;

    let numbers: Vec<u64> = (0..4096).collect();
    let mut total = 0;
    for batch in numbers.chunks(256) {
        cancel.bail_if_cancelled()?;
        total += batch.iter().copied().sum::<u64>();
    }
    cancel.bail_if_cancelled()?;
    Ok(total)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let cancel = CtrlCHandler {
        should_eprintln_on_ctrl_c: true,
        should_force_exit_on_repeated_ctrl_c: false,
        ..CtrlCHandler::default()
    }
    .install()?;

    let total = total_numbers(cancel.clone()).await?;
    cancel.bail_if_cancelled()?;
    let output = format!(r#"{{"version":1,"total":{total}}}"#) + "\n";
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(output.as_bytes()).wrap_err("writing JSON result")?;
    stdout.flush().wrap_err("flushing JSON result")?;
    Ok(())
}
```

The worker observes cancellation before and after the bounded await, between CPU batches, and before publishing its result. The token has no async `cancelled()` future. A token check does not interrupt an await or blocking call already in progress. Use the operation's own cancellation mechanism where available, and pass the token into spawned workers.

Keep CPU batches small enough for the intended response time. Cancellation hooks run synchronously and should return promptly.

## Choose the interruption policy

In the pinned library, the first Ctrl+C requests cancellation. The default handler prints to stderr and enables forced exit on a repeated signal within one second. That policy calls `process::exit(130)`; it can bypass normal completion and cleanup.

The example explicitly disables that repeated-signal force exit. This is an example policy, not the current mover or template configuration. A command promising a controlled stop must define how it reaches and records a safe boundary.

Human progress and cancellation notices can remain on stderr when stdout contains JSON. Keep those notices out of the machine-readable result stream.

The example propagates stdout write and flush failures through `eyre`. A result is not successfully published when its output failed.

## Preserve progress before stopping a mutation

For a future operation that changes valuable state:

1. Check the token before starting the next action.
2. If an action has started, finish it or enter its defined recovery state.
3. Record the actual outcome and a durable checkpoint before announcing a controlled stop.

Avoid a cancellation check between completing a filesystem change and recording its receipt. The checkpoint should identify completed work, any unresolved action and where an operator can resume. Its metadata needs the same audience protection as the work it describes.

The local mover prototype currently checks cancellation before dispatch. Its planning methods do not yet receive the token through their internal loops. The template checks command and output boundaries. The locator checks discovery and author walks, while its synchronous published-index query cannot be interrupted through this token. These limits matter when describing responsiveness.

## Keep completed outcomes typed

A completed lookup can report no matching item or exclusion by policy. Put those findings in a typed report and return exit status 0 under this CLI convention. An unrecoverable `eyre` error returns a nonzero status.

Cancellation means the promised computation stopped before completing. Represent it separately from completed domain findings and describe any partial progress. The handler's forced-exit code is an interruption policy, not a domain-result taxonomy.

The example propagates cancellation through `eyre`; a production command should document its cancellation report and exit contract. Preserve an already committed domain result if a later request arrives.

See [CLI output shapes](output-shape.md) for separating typed outcomes, human diagnostics and stable machine output.
