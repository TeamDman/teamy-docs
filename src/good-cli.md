# Good CLI

A [command line interface (CLI)](glossary.md#cli) should make its inputs, outputs and effects predictable for a person, a script and an agent. The examples below describe a hypothetical `example-tool`; they are design conventions, not existing commands.

## An explicit command surface

Prefer subcommands whose intent is clear:

```text
example-tool --help
example-tool --version
example-tool inspect --help
example-tool plan create --output plan.json
example-tool plan action add --plan plan.json --source "<source>" --destination "<destination>"
example-tool plan validate --plan plan.json --output-format json
example-tool plan apply --plan plan.json --approval "<approval-reference>"
```

An unknown subcommand should fail with a useful diagnostic. Do not reinterpret a typo as a shorthand destructive operation. Help should state the default scope, required options, exit status and effects. Version output should identify the tool/schema compatibility a report can rely on.

Use the hierarchy to name the object being changed. `plan create` creates the plan; `plan action add` adds an action inside it. Keep `plan list` distinct from `plan action list`.

## File, standard-input and argument inputs

Support explicit input sources where useful:

| Form | Contract to document |
| --- | --- |
| `--input requests.json` | Read the named file as the declared versioned schema. |
| `--input -` | Read UTF-8 input from standard input; do not prompt on that stream. |
| `--input '@requests.json'` | Optional response-file convention, interpreted by this tool only. |
| `--source "<source>"` | A literal path argument, not a shell expression or glob. |

An `@file` convention needs defined escaping, encoding, size limits and nesting rules. It is not universal CLI syntax. Quote it in PowerShell so it is passed as a literal rather than treated as splatting. Avoid hidden recursive expansion or executable content.

Use argument arrays when launching another process. Do not construct a shell command by interpolating paths; quoting for one shell is not a portable serializer. Expand a wildcard only through an explicit option and freeze the individual resolved objects in the resulting plan.

## Stable output

Keep command results on standard output and human progress, notices and diagnostics on standard error. `--output-format json` selects the stdout format; it does not silence stderr. A command may emit a JSON result while talking to the user on stderr. Only stdout must remain free of progress bars, log lines and other decoration.

Text, JSON and other formats should render the same typed result. Document its schema, version compatibility, UTF-8 encoding and record framing. For a measurement report, represent unknown size as unknown, not zero, and include capture time and coverage where they affect interpretation.

Use exit 0 when the command completes and produces its declared result. Put findings such as conflicts, incomplete observations and readiness in typed fields the caller can inspect. An unrecoverable error propagates through `eyre` and exits nonzero. Callers should not need an exit-code taxonomy to distinguish domain outcomes. Stderr output alone does not indicate failure.

See [typed results and errors](output-shape.md) for this contract and [cancellation with teamy-cancellation](cancellation.md) for stopping work safely.

## Effects and permissions

Document the effects of the behaviors your CLI actually supports. The following are examples, not a required set of subcommands:

| Mode | Expected effects |
| --- | --- |
| Help/version | Report information without changing inspected objects. |
| Read and list | Observe objects within the declared scope and return a report. Save a separate report file only when an output destination is explicitly requested. |
| Plan | Write inert plan data; do not apply its actions. |
| Dry run | Preview the specified operation without modifying its targets; disclose any cache/report effects. |
| Execute | Perform only the reviewed actions after validation and authorization. |

An inventory is a recorded description of observed objects, such as the repository catalogue returned by the locator. It is a general reporting pattern, not a command every new CLI must implement. `teamy-mover` has plan-reading and dry-run commands; it has no inventory subcommand.

Network access, elevation, content reads, cache refresh and service/configuration changes are independent effects. A dry-run flag is a useful contract, but OS-enforced capabilities provide a stronger boundary than a flag alone.

Authentication identifies a caller. Approval authorizes a particular action. An apply boundary can bind approval to a digest of the complete reviewed plan, including staging locations and destination semantics. Validate object identities and conflicts again immediately before mutation; elapsed time makes plans stale.

See [writing programs](writing-programs.md) for an inert plan example and [repository hygiene](repository-hygiene.md) for previewing cleanup.

Concrete local prototype surfaces appear in [mover planning](writing-programs.md#generate-a-plan-with-the-local-mover-prototype) and [source lookup](source-lookup.md#local-locator-prototype). Keep their implemented capabilities distinct from the hypothetical full interface above.
