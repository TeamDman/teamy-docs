# Shared glossary

## CLI

A command line interface (CLI) accepts commands and arguments through a terminal or process invocation and reports results through output streams and an exit status. See [Good CLI](good-cli.md) for input, output and effect conventions.

## Other terms

| Term | Meaning in this book |
| --- | --- |
| Actor | The person or system performing an action. |
| Audience | The people or systems allowed to receive a particular piece of information. |
| Authority | The source whose rule or decision is trusted for a specified purpose. |
| Objective | The concrete outcome being sought. |
| Constraint | A condition a solution must satisfy. |
| Preference | A tradeoff that can guide a choice after constraints are satisfied. |
| Loss function | A stated measure of how far a candidate is from the preferred result; its weights express preferences. |
| Discovery | Observing candidate objects, with declared scope and acquisition effects. |
| Inventory | A recorded observation; it may become stale and does not establish ownership or permission. |
| Result shape | The typed fields and outcomes a command returns; see [typed results and errors](output-shape.md). |
| Cancellation | A request to stop work, observed at cooperative boundaries; see [teamy-cancellation](cancellation.md). |
| Plan | Versioned data describing proposed actions and their assumptions. |
| Execution | Carrying out the actions in a plan; this can change the objects the plan describes. |
| Dry run | A preview whose documented contract excludes the corresponding mutations; its other effects must still be stated. |
| Approval | Authorization for a defined action, object, destination and purpose. |
| Authentication | Evidence of an identity; it does not by itself approve an action. |
| Provider | An implementation of a defined observation or operation contract. |
| Provenance | Evidence describing where an object or measurement came from. |
| Scratch | Work material created for a temporary purpose; disposability depends on its lifecycle and retained outputs. |
| Projection | A deliberately selected representation for an audience, built only from permitted inputs. |
| Checkpoint | Durable progress information used to inspect or resume work. |

These terms are deliberately separate. Knowing a repository exists does not imply permission to read its contents. A folder name does not establish an audience policy. A fast measurement does not establish that it is current or complete.

## Reflection

Reflection lets generic code inspect descriptions of types and their fields or operations. A reflection system's available metadata determines what that code can discover.

## Facet

[Facet](https://github.com/facet-rs/facet) is a Rust ecosystem providing runtime type reflection. Its type descriptions support tools such as serializers and value inspection.

## Figue

[Figue](https://github.com/bearcove/figue) is a parser for CLI arguments, configuration files and environment variables, based on Facet. Those input mechanisms still need a documented precedence and effects contract in the consuming tool.

## Event

A tracing event records something that happened, with a level, target and structured fields. Events belong to operational communication; a command's typed result has a separate contract. See [logging](logging.md).

## Span

A tracing span represents an operation over time and carries context for the events inside it. Spans help connect related work and provide profiling boundaries. See [logging](logging.md#events-spans-and-destinations).

## Subscriber

A tracing subscriber receives events and span activity. Its layers can filter, format and write that instrumentation to different destinations. The template uses `tracing-subscriber`; see [the logging setup](logging.md#terminal-logs-ndjson-and-tracy).

## PTY

A pseudoterminal (PTY) is a terminal-like connection used by a terminal host to communicate with a program. On Windows, MFT's elevated relaunch attaches to an existing console so its logs remain visible in the caller's terminal. Console attachment and captured stdout/stderr pipes have different behavior; see [cross-process logging](logging.md#elevation-and-daemon-log-forwarding).
