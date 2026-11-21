# Compare action plans with Teamy Capability Lab

Teamy Capability Lab is an offline Rust experiment that answers a planning question: given one artifact, where can it go, which routes need approval, and what would each route cost? Its fictional environment distinguishes a desktop, a VM, a store and individual running Codex instances. Each transfer has a direction. The sender must be able to send that artifact and the receiver must be able to accept it.

## Compare three routes

Run `cargo run -- --output-format text plan` in the repository. The fixture has three alternatives for the same artifact.

| Route | Estimated cost | Result | Reason |
| --- | ---: | --- | --- |
| Through the store | 4 | Ready | Upload and download both have the required permissions. |
| Direct | 3 | Approval needed | Permission for the transfer is unknown. |
| Through the VM | 2 | Blocked | The transfer to the VM is denied. |

Scryer Prolog enumerates paths without revisiting a node. Rust checks the artifact, endpoint capabilities and permissions, then ranks the returned paths by readiness and estimated cost. The report says when a depth, result-count or inference limit cut the search short. In that case the cheapest returned route may only be the cheapest one explored. Running the planner neither transfers the artifact nor grants approval.

## Limit what a caller can learn

Run `cargo run -- --output-format json disclosure` for the second fixture. It evaluates one selected VM and checks permission to invoke the operation separately from permission to read its effect. For an allowed request matching the owner tag, the result exposes only the VM's power state. The tag, OS version and disk label stay out of the projection. The inputs and permissions are synthetic.

For execution-time checks, expiring grants and revocation, see [Teamy Agent](teamy-agent.md).
