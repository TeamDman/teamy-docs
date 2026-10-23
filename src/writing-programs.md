# Writing programs

Read [TeamyStyle](teamystyle.md) for our prescriptive engineering rules and review checklist, then use the detailed chapters below for implementation contracts.

Start a new Teamy CLI project in Rust using [teamy-rust-cli](rust-cli-template.md). Understand its parser, [logging](logging.md), [output renderer](output-shape.md) and [cancellation](cancellation.md) before adding domain behavior. These shared decisions are the starting point for our programs.

Build the smallest useful behavior that can be demonstrated with real inputs. Record the intended result and use the template's existing capabilities to produce it.

Use [Find a solution](problem-index.md) to locate prior art by intent. Our software topics cover [command-line tools](good-cli.md), [desktop applications](desktop-applications.md), [machine learning and audio](machine-learning.md), and [source and file organization](source-and-files.md). A project can contribute useful patterns to several topics.

For email inspection, composition, SMTP submission and IMAP synchronization, see [Plan an owned email CLI](email-planning.md). Its offline-first stages are proposals, with Makepad and rust-imap retained as source references.

For coding-agent execution and long-lived conversation memory, see [Evaluate agent harnesses and memory](agent-harnesses.md). It distinguishes OpenCode's harness from OptChat's OMP extension and retains their qualification limits.

For reusable coding setups and desktop/cloud handoffs, see [Work across Codex cloud environments](cloud-environments.md). Include this book as a reference checkout in each new Teamy coding environment and verify the actual source revisions before applying its standards.

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

For sustained conversion work, use [migration tooling and measured iteration loops](migration-workflows.md). Generate the burndown and next action from current checks. Profile repeated selector work and compare reasoning or delegation choices by accepted results, elapsed time and usage.

Use one meaningful fixture to establish correctness, then expand when an unresolved failure or requirement justifies it. Observe errors through stable diagnostics and inspectable output. Avoid a dashboard that suggests precision when measurements are incomplete.

Document assumptions next to the behavior they affect. For example, an initial same-volume move implementation should say that cross-volume transfers need a separate integrity and recovery design.

See [repository hygiene](repository-hygiene.md) for build output and scratch ownership, and [the glossary](glossary.md) for the distinction between a goal, a plan and an approval.
