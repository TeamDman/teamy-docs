# Cooperative cancellation

Pass a cancellation token through the work and check it at safe boundaries. The first Ctrl+C requests cancellation. Workers must observe the token; it does not interrupt every running function or future. The handler's repeated-signal policy is described below.

## Current 0.4 async and interruption contract

The local template inspected on 15 October 2026 now locks **teamy-cancellation 0.4.0** from crates.io, with `figue`, `arbitrary` and `async` enabled alongside the default features. Its lockfile SHA-256 is `66c8b3b805c322763510198c3025f9101182867594bee988c49c34d51481fd0d`. The downloaded [published source archive](https://static.crates.io/crates/teamy-cancellation/teamy-cancellation-0.4.0.crate) matched that checksum; its VCS metadata identifies revision `8c4dcc3bb58a30e7115671ccff9adbe944d3f958`. The older local library checkout remains 0.3.1 and cannot establish the newer async behavior. This section supersedes the older template-version statements below, while preserving that older example's qualification.

[TeamyStyle](teamystyle.md) requires a controlled first cancellation request, prompt async propagation and explicit ownership of remote work. In 0.4.0 the exact default signal policy is:

- First Ctrl+C requests cancellation with reason `Operation cancelled by Ctrl+C`.
- If another Ctrl+C arrives **at most one second after the preceding Ctrl+C**, with the default force-exit option enabled, the process calls `process::exit(130)`. The comparison is inclusive (`<=`), measured with `Instant`; it is between consecutive signals, not simply “any later second press.”
- A later signal outside that window requests graceful shutdown again and becomes the new preceding signal. `repeated_ctrl_c_window` and `should_force_exit_on_repeated_ctrl_c` are explicit handler configuration.
- Hard exit can bypass destructors, terminal restoration, checkpoints, queued logs and async cleanup. It cannot promise rollback or a final receipt. The one-second signal interval is not a deadline for finishing graceful shutdown or for answering a human prompt.

Source: [`CtrlCHandler`](https://github.com/TeamDman/teamy-cancellation/blob/8c4dcc3bb58a30e7115671ccff9adbe944d3f958/src/ctrlc_handler.rs) and [`CancellationState`](https://github.com/TeamDman/teamy-cancellation/blob/8c4dcc3bb58a30e7115671ccff9adbe944d3f958/src/cancellation_state.rs). The inspected template installs `CtrlCHandler::default()`, retaining its direct colored `^C` stderr print. **Gap:** that library print bypasses tracing, so Ctrl+C can contaminate requested NDJSON stderr or a TUI. The handler can disable that print; an adopter must route the notice through its guarded diagnostic path and qualify the result instead of claiming clean NDJSON from ordinary log tests alone.

### Wake pending async work

With the 0.4.0 `async` feature, `token.cancelled().await` waits for sticky cancellation on the token or an ancestor; each wait registers its own waker and unregisters when dropped. `token.cancellable(fallible_future).await` returns a typed `CancellationError` through the future's error type when cancellation is observed. It can wake an otherwise pending I/O wait; unlike the 0.3.1 example, polling before and after an unrelated long await is no longer the only available pattern.

Use `cancellable` only for work that is safe to abandon. It stops polling the operation, retains its inner future until the wrapper is dropped, and cannot preempt a running poll, blocking system call or GPU kernel. If the operation returns ready during its poll, that original result wins over a concurrent cancellation request. Keep the wrapper's borrowed sources alive until it is dropped. Arbitrary task abort/drop is not transactional cleanup.

Contextual 0.4.0 fragment (not compiled by this book):

```rust,ignore
let request_cancel = application_cancel.child_token();
let result: eyre::Result<Response> = request_cancel
    .cancellable(read_cancellation_safe_response())
    .await;
// Finish the operation's declared cleanup/checkpoint path before returning result.
```

Parent cancellation reaches descendants; cancelling a child does not cancel its parent or siblings. Clone a token to share the same scope. The optional `deferral` feature provides `for_operation()` and deferral guards for required completion work, but the inspected template does **not** enable that feature. Do not claim it protects a template mutation automatically. Even with deferral enabled, external abort/drop and hard process exit bypass that cooperation.

Source: [token API](https://github.com/TeamDman/teamy-cancellation/blob/8c4dcc3bb58a30e7115671ccff9adbe944d3f958/src/cancellation_token.rs), [cancellable future](https://github.com/TeamDman/teamy-cancellation/blob/8c4dcc3bb58a30e7115671ccff9adbe944d3f958/src/cancellable_future.rs) and [wait implementation](https://github.com/TeamDman/teamy-cancellation/blob/8c4dcc3bb58a30e7115671ccff9adbe944d3f958/src/cancellation_wait.rs).

### Complete the owned shutdown path

On graceful cancellation, stop accepting new work, notify owned workers and await their completion. Finish an already irreversible action or persist its declared recovery state; pair completed effects with their receipts before reporting a stop. Drain owned output/diagnostic queues under an explicit shutdown budget, report loss, restore terminal state and return nonzero if the command did not complete. A cancellation token alone does none of that orchestration.

Use a request child token for daemon calls. Send cancellation through the transport, observe acknowledgement/completion and keep an independently started daemon alive. An elevated child is a separate process: explicitly define whether the parent or child owns Ctrl+C and how cancellation crosses that boundary; console attachment and exit propagation do not establish cancellation propagation. Cancelling a client locally is not proof that remote work stopped. After forced termination, the next run must reconcile durable state rather than invent a completed receipt.

**Exit expectations:** forced signal exit is exactly 130 in this library. The current template's safe binary boundary returns `ExitCode::FAILURE` for cooperative command errors, including cancellation; it does not map every cancellation to 130. Completed domain findings still follow [typed result semantics](output-shape.md). A project promising a different interruption status must implement and test that mapping explicitly.

### Accept the behavior, not just the installed handler

| Scenario | Required evidence |
| --- | --- |
| First OS Ctrl+C during pending async work | The operation wakes, stops new work, completes cleanup/checkpoint/drain, restores the terminal and exits nonzero without forced termination. |
| Already-cancelled token, then dispatch | No operation starts; the typed cancellation reason is retained. |
| Parent versus child request cancellation | Parent wakes dependent waits; child cancellation leaves its sibling and independently owned daemon usable. |
| Consecutive signal timing | State tests cover a rapid repeat, the exact one-second boundary, a repeat outside it, and disabled force exit; process tests separately establish actual OS signal delivery and hard exit 130. |
| Cancellation after an effect, before receipt | Recovery reconciles actual state; no false success or blind replay. Test abrupt process death separately from cooperative stop. |
| Cancelled remote/elevated request | Remote completion/acknowledgement and invoking-client status agree; following work succeeds and log loss is disclosed. Console reattachment alone is insufficient evidence. |
| NDJSON or active TUI plus Ctrl+C | No direct library notice corrupts machine framing or the screen; cleanup is checked in the actual terminal host. |

The current template's `tests/async_cancellation.rs` contains parent-wake, cancelled-dispatch, duration and typed-error fixtures. The published library contains signal-state tests. Those inspected fixtures do not establish all OS signal, terminal, durable-state or elevated/daemon acceptance cases above, and were not rerun for this documentation update. The historical bounded-await example below retains its original version and validation scope.

## Use the shared library

[teamy-cancellation](https://github.com/TeamDman/teamy-cancellation) provides `CancellationToken` and `CtrlCHandler` for Rust tools. The example below targets version 0.3.1 at public revision `7c77848e759079e21f2039b25b147d780807725b`. That was the CLI template and mover baseline when the example was qualified; the current template now uses 0.4.0 as described above. The locator currently uses version 0.2.0, so optional facilities can differ.

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

The template's tracing layer also observes configured span and log-message stop conditions. Quieting a formatted log destination does not replace cancellation checks in the work. When adding a background log writer, coordinate its shutdown with the interruption policy; see [logging and buffering](logging.md#buffering-means-several-different-things).

The example propagates stdout write and flush failures through `eyre`. A result is not successfully published when its output failed.

## Preserve progress before stopping a mutation

For a future operation that changes valuable state:

1. Check the token before starting the next action.
2. If an action has started, finish it or enter its defined recovery state.
3. Record the actual outcome and a durable checkpoint before announcing a controlled stop.

Avoid a cancellation check between completing a filesystem change and recording its receipt. The checkpoint should identify completed work, any unresolved action and where an operator can resume. Its metadata needs the same audience protection as the work it describes.

The local mover prototype currently checks cancellation before dispatch. Its planning methods do not yet receive the token through their internal loops. The template checks command and output boundaries. The locator checks discovery and author walks, while its synchronous published-index query cannot be interrupted through this token. These limits matter when describing responsiveness.

## Restore an interactive terminal

An interactive application must handle cancellation while waiting for input and while work runs. In raw mode, Ctrl+C may arrive as a keyboard event rather than the usual console signal; route the event to the intended request or application token and explain its scope in on-screen help.

After cooperative work stops, restore terminal modes and screen state before printing a final error or releasing ownership to a parent interface. Restoration failure is an unrecoverable error; a nested owner must not acknowledge a safe handoff when cleanup failed. Cloud-Terrastodon's [terminal ownership protocol](terminal-ownership.md#transfer-ownership-after-restoration) and [async input/work separation](async-terminal-ui.md) are our implementation references. A token cannot preempt an executing GPU kernel or automatically interrupt a blocking stdin read.

## Keep completed outcomes typed

A completed lookup can report no matching item or exclusion by policy. Put those findings in a typed report and return exit status 0 under this CLI convention. An unrecoverable `eyre` error returns a nonzero status.

Cancellation means the promised computation stopped before completing. Represent it separately from completed domain findings and describe any partial progress. The handler's forced-exit code is an interruption policy, not a domain-result taxonomy.

The example propagates cancellation through `eyre`; a production command should document its cancellation report and exit contract. Preserve an already committed domain result if a later request arrives.

See [CLI output shapes](output-shape.md) for separating typed outcomes, human diagnostics and stable machine output.
