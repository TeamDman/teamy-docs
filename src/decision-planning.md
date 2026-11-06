# Find possible actions from typed capabilities

Start with a concrete target and known inputs. A registry can describe operations as one input type and one output type: a request struct packages required arguments, optional fields and authentication context. An output can likewise contain many values. A type match suggests an operation to investigate; it does not establish that values refer to the same tenant, are authorized, or exist.

## Existing Cloud Terrastodon prior art

At public revision [`fd989bd`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/commit/fd989bd357cbfc5d60ad06aff2a7677a3a46491f), [registered functions](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/registry/src/function.rs) expose Facet input/output shapes, operation kind, effects and an executor. [Registered types](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/registry_values/src/thing.rs) use LinkMe distributed slices. These are implementation references, not a promise that an external project can consume the crates without integration work.

The [discovery planner](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/registry/src/discovery_plan.rs) builds an offline, target-directed graph from an explicit function catalog and seed shapes. It does not invoke requests, defaults, conversions or credential lookup. It admits unattended reads and pure synchronous conversions and projections; declared effects still need auditing. For a reachable target it retains one deterministic derivation and all required inputs, rather than ranking every possible solution. A blocked target retains candidate producers and missing dependencies. The [dump-plan command](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/crates/entrypoint/src/cli/command/dump/dump_plan_cli.rs) can export an Excalidraw graph for inspection.

Reachability means potential *type* coverage. An empty list, absent option, inactive enum variant, failed conversion or API error can prevent live work. Shapes alone do not prove tenant or organization correlation. A plan is neither an executable schedule nor evidence of a successful API call.

For the distinct question of what an actor may invoke, see [delegated authority](delegated-authority.md). For coordination using shared traces or messages, see [shared coordination](shared-coordination.md).

## Choose among real alternatives

For an inventory, a targeted request and a broad resource query followed by filtering may both appear to yield a desired type. Fewer requests is only one cost: account for data volume, permissions, latency, rate limits, freshness, pagination, tenant scope and reuse across multiple targets. Estimate and measure these costs; avoid claiming an optimum without a complete model. A resource-group-to-subscription derivation can miss subscriptions with no returned groups. Broad resource inventory may also omit container kinds; verify the endpoint and scope before treating it as a covering set.

For dependency investigation, a proposed evidence chain might connect a local file and Git revision to a repository, pipeline definition and deployment resource. Each edge needs evidence: a revision in a branch that a pipeline could build is not proof that a particular run built or deployed it. Pipeline configuration, historical runs and deployment provenance may be missing or expired. Return uncertainty with the chain.

## A bounded file-transfer slice

Model synthetic devices, a file, a goal location, and time-scoped facts such as network direction, authenticated reachability and write access. Offer candidate actions: an approved connector, a controlled file server and client, or a purpose-built mover. A route must satisfy all prerequisites, not just join matching types. Trust in maintained code can be an explicit preference, with evidence and alternatives visible; neither Rust nor author identity automatically proves safety or performance.

If a cloud client can reach a desktop but the reverse direction is unverified, a desktop server can offer a selected file for a cloud pull; a desktop receiver and cloud sender can support the opposite transfer. Test each direction independently, with authentication, a bounded selection, integrity verification and destination handling. A service that only implements upload cannot be assumed to support pull. [Execution boundaries](execution-environments.md) describes why capabilities belong to specific instances.

A new experiment should start from [teamy-rust-cli](rust-cli-template.md), register synthetic types and no-op or explicitly non-executable operations, and show a reachable goal, an unmet capability and a cyclic or misleading candidate. Compare the existing offline planner with a bounded [Prolog](logic-programming.md) query before choosing a dependency or attempting live effects. Include an explanation of rejected alternatives and structured failure reasons. This is proposed work; no such standalone integration is asserted here.
