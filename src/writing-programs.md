# Writing programs

Start a new Teamy CLI project in Rust using [teamy-rust-cli](rust-cli-template.md). Understand its parser, [logging](logging.md), [output renderer](output-shape.md) and [cancellation](cancellation.md) before adding domain behavior. These shared decisions are the starting point for our programs.

Build the smallest useful behavior that can be demonstrated with real inputs. Record the intended result and use the template's existing capabilities to produce it.

Use [Find a solution](problem-index.md) to locate prior art by intent. Our software topics cover [command-line tools](good-cli.md), [desktop applications](desktop-applications.md), [machine learning and audio](machine-learning.md), and [source and file organization](source-and-files.md). A project can contribute useful patterns to several topics.

## Define the result

A useful problem statement names the actor, input, expected output and relevant constraint:

> An operator supplies a reviewed list of existing paths. The tool reports proposed destinations and conflicts without modifying those paths.

That statement immediately suggests a [CLI surface](good-cli.md), an input schema, a read-only report and a fixture with a conflicting destination. It does not yet require a general-purpose scripting language.

## Make effects reviewable

Separate discovery, planning and execution when a task changes valuable state. A plan should name concrete objects, actions and reasons. Execution should reject stale assumptions and report progress durably.

Use [plan records and decisions](plan-records.md) for gradual typing, passage feedback and publication boundaries. The book currently builds and checks local links; that does not establish that every code snippet compiles.

For a move tool, a synthetic plan sketch could contain:

```json
{
  "version": 1,
  "actions": [
    {
      "kind": "move",
      "source": "<source-root>/example-project",
      "destination": "<destination-root>/example-project",
      "reason": "Place the existing project in its reviewed collection"
    }
  ]
}
```

This sketch illustrates vocabulary only. For `teamy-mover`, generate a plan through its CLI and use its current schema, which includes additional fields and validation. The placeholders are not executable paths. A real plan needs explicit resolved locations and identity/conflict checks appropriate to its platform. A saved plan is data; it does not confer permission to execute its actions.

## Generate a plan with the local mover prototype

See [Plan file movement](moving-files.md) for the command hierarchy, runnable checkout examples, ordered simulation and revision history. It uses the local `teamy-mover` prototype as prior art for this problem. `plan create` creates a plan; `plan action add` edits its contents. The prototype has no executor or published release.

## Keep the feedback loop short

Use one meaningful fixture to establish correctness, then expand when an unresolved failure or requirement justifies it. Observe errors through stable diagnostics and inspectable output. Avoid a dashboard that suggests precision when measurements are incomplete.

Document assumptions next to the behavior they affect. For example, an initial same-volume move implementation should say that cross-volume transfers need a separate integrity and recovery design.

See [repository hygiene](repository-hygiene.md) for build output and scratch ownership, and [the glossary](glossary.md) for the distinction between a goal, a plan and an approval.
