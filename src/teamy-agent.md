# Authorize agent actions with Teamy Agent

Teamy Agent models what happens after a model proposes an action. The host authenticates the account, a typed request identifies the operation and its inputs, and a broker checks the account's grants before executing it. The prototype uses a scripted model, synthetic documents and recording destinations. Its scope begins with a structured request. Turning a natural-language prompt into a plan is a separate problem. [Foundation design](https://github.com/TeamDman/teamy-agent/blob/bef2691e771db8b9c54e793c9295886d27425b33/docs/foundation.md)

## Follow a private summary

Suppose an agent can read private notes and asks to send a summary to an external model. The broker checks the read against the document ID. The summary carries both the original document ID and its local-only label. When the agent submits the summary, the broker checks the destination and source permissions again. The recording destination receives nothing because it accepts only public data. If read access is later revoked, replaying a saved result also requires a fresh permission check. [Acceptance scenario](https://github.com/TeamDman/teamy-agent/blob/bef2691e771db8b9c54e793c9295886d27425b33/crates/runtime/src/acceptance.rs)

## Trace an authorization decision

The registry describes eight operations covering document reads, summaries, model submissions, output writes, account creation, grant issuance, revocation and impersonation. Each grant can limit the action, document, destination, impersonation target and expiry. A child grant can narrow its parent's scope. Revoking an ancestor invalidates the child's authority, including after the broker restarts. Use the CLI's `schema` command to inspect the operations and `acceptance` with a fresh empty directory to exercise the synthetic scenario. [Registry](https://github.com/TeamDman/teamy-agent/tree/bef2691e771db8b9c54e793c9295886d27425b33/crates/registry) · [CLI examples](https://github.com/TeamDman/teamy-agent/blob/bef2691e771db8b9c54e793c9295886d27425b33/README.md)

For an example of finding candidate actions before execution, see [Teamy Capability Lab](teamy-capability-lab.md).
