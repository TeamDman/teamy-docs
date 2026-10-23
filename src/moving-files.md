# Plan file movement

When reorganizing existing files, record their exact proposed destinations before moving anything. Our current solution is the local `teamy-mover` planner. It represents ordered actions as versioned data and checks a simulated future state. It has no executor.

The implementation is still a local prototype as of 1 October 2026, with no published repository or release. The public book describes its reviewed design and synthetic command examples. Real plans can disclose filenames and storage locations; keep them in records for their intended audience.

A separate [cloud-to-desktop transfer proposal](#proposed-cloud-to-desktop-transfer), dated 2026-10-23, extends the design discussion. It does not add network transfer or execution to the existing planner.

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

## Proposed cloud-to-desktop transfer

**Proposed, 2026-10-23; not implemented or transfer-tested.** Extend Teamy Mover with a native Rust receiver/accept operation on the desktop, confined in Podman, and a cloud sender that initiates an approved outbound connection. This addresses the [managed VPN's direction limits](networking-survey.md#codex-cloud-private-networking) and provides an owned artifact copy without depending on Python Library helpers or a WSL workaround. Current video iteration can continue through the [producing chat](making-videos.md#view-and-deliver-a-cloud-render).

Use [teamy-rust-cli](rust-cli-template.md), first-class Figue arguments, Facet types and a versioned Vox service contract. Propose two transport adapters: direct Vox over TLS through approved CONNECT where required, and a future Veilid adapter. Both share the same Facet/Vox upload service, authorization, integrity, idempotency and atomic-commit semantics; neither is implemented or tested. The existing [Vox log-stream reference](logging.md#elevation-and-daemon-log-forwarding) is prior art for typed communication, not proof of a ready-made secure file-transfer transport. Qualify framing, payload limits, backpressure, cancellation and completion acknowledgements for each adapter. A local socket flush must not be reported as a committed remote artifact.

The direct adapter's intended direction is:

```text
Cloud sender -> approved outbound proxy/VPN path -> desktop Rust receiver
                                                      |
                                              dedicated inbox volume
```

### Establish server trust and upload authority separately

The direct adapter terminates TLS inside Rust. Keep the receiver's private key protected on the receiving side; do not send it to the cloud, bake it into an image or write it to logs. Provision the cloud client with a trusted CA certificate and hostname validation, or a deliberately pinned server public key through an authenticated setup channel. Specify certificate/key rotation and verify the actual TLS peer on the approved network path. Do not weaken verification to make a proxy mismatch disappear.

A CSR is a request for certification, containing a public key and signed request information; it is not a trust anchor or proof that a particular server should be trusted. See [PKCS #10 / RFC 2986](https://www.rfc-editor.org/rfc/rfc2986.html). TLS authenticates the selected server and protects the connection; upload authorization remains a separate application decision.

The proposed acceptance operation issues an opaque, unguessable capability after the receiver approves a bounded transfer. Keep the capability out of ordinary URLs and logs. Store its authority on the receiver, including the accepted content identity and limits; the cloud sender cannot broaden them by editing request fields.

| Proposed contract | Receiver responsibility |
| --- | --- |
| Size and expiry | Enforce maximum bytes while streaming, expiry and any aggregate quota; reject excess before committing. |
| Content hash | Bind approval to the expected digest and verify the completed bytes. A matching hash proves agreement with the approved content, not that the content is safe to execute. |
| Fixed destination root | Choose the inbox destination on the receiver. Reject traversal, symlink/reparse-point escapes and arbitrary sender-supplied host paths. |
| Idempotency | Bind a retry key to the same capability, digest and destination; return the recorded outcome for an identical retry and reject conflicting reuse. |
| Atomic commit | Stage inside the destination filesystem, verify length/hash, then commit without exposing partial output or silently overwriting unrelated files. Define crash recovery and durability explicitly. |
| Bounded authority | Limit reuse, permit revocation and make completion consume or close the grant according to the reviewed contract. Never grant shell execution or general filesystem access. |

These are acceptance requirements for future implementation, not claims about the planning-only prototype. Meaningful tests should cover denied/expired grants, wrong certificates and hashes, oversized uploads, path escapes, cancellation, lost acknowledgements, duplicate retries and interrupted commits. Verify that a following valid transfer still works. Record passed, failed and unrun cases separately.

### Isolate the build and the receiver independently

**Officially documented primitives:** [Podman's runtime options](https://docs.podman.io/en/latest/markdown/podman-run.1.html) include a read-only root filesystem, restricted users/capabilities and explicit volume mounts. [Podman build](https://docs.podman.io/en/latest/markdown/podman-build.1.html) has its own build context, mounts and network controls. Runtime restrictions do not retroactively isolate a build.

**Proposed Teamy boundary:** build reviewed, pinned Rust sources with a minimal context and only the needed dependency access. Treat writable compiler/package caches as a trust boundary: build code can alter their contents and influence a later build. Use dedicated caches with an explicit trust scope, not a shared writable host cache containing unrelated work. Do not expose the host home, container-engine socket or host credentials to either build or runtime.

Run the receiver as an unprivileged user with a read-only image, dropped capabilities, no privilege escalation and bounded temporary storage. Expose only the approved listener and a dedicated inbox as the persistent writable data volume. Deliver only the receiver-specific TLS material through a narrowly scoped secret mechanism; keep the key separate from uploaded data. Do not mount general desktop folders, source trees or unrelated credentials. An upload lands as data in the inbox, not as executable code or an automatically opened file.

[Podman machine](https://docs.podman.io/en/latest/markdown/podman-machine.1.html) uses a Linux VM on Windows; the documented providers include WSL and Hyper-V. That container backend is a separate choice from running a Python Library helper inside WSL, which was rejected for this workflow. Qualify the selected backend's volume mapping, ownership and listener reachability; no Windows receiver/container configuration was deployed or tested by this proposal.

### Propose a Veilid adapter; retain Stigmerge as a reference

The [Stigmerge README at `8f26b50`](https://github.com/cmars/stigmerge/blob/8f26b50ffa058eaa009c61eccabe32b2a756de4c/README.md), inspected on 2026-10-23, describes file seeding/fetching over Veilid. Fetching peers also seed data. Stigmerge remains reference-only for content distribution and peer discovery, with materially different exposure and lifecycle from a single approved inbox upload; no Stigmerge integration is proposed.

**Desired future adapter, 2026-10-23:** Veilid would carry the same upload service alongside the direct adapter. Its [routing documentation](https://veilid.gitlab.io/developer-book/concepts/private-routing.html) distinguishes safety routes, designed to protect the sender's network location, from private routes, designed to protect the receiver's location. Neither is synonymous with a private tailnet link. The adapter needs bounded message fragmentation and reassembly, ordering, deduplication and backpressure, plus an independently verified identity bootstrap binding the intended peer to the upload authority. Route discovery alone does not establish that identity.

This adapter is proposed, not implemented or tested. Qualify Veilid's networking and message behavior in the target environment; do not assume direct one-hop cloud connectivity or stock Veilid support for the platform's CONNECT path. [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve) remains an optional alternative reference; Rust-owned TLS remains the direct adapter's chosen termination design.
