# Writing programs

Build the smallest useful behavior that can be demonstrated with real inputs. Record the intended result before choosing a framework or generating a large scaffold.

## Define the result

A useful problem statement names the actor, input, expected output and relevant constraint:

> An operator supplies a reviewed list of existing paths. The tool reports proposed destinations and conflicts without modifying those paths.

That statement immediately suggests a [CLI surface](good-cli.md), an input schema, a read-only report and a fixture with a conflicting destination. It does not yet require a general-purpose scripting language.

## Make effects reviewable

Separate discovery, planning and execution when a task changes valuable state. A plan should name concrete objects, actions and reasons. Execution should reject stale assumptions and report progress durably.

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

The local prototype exposes planning commands. Replace `<existing-root>` with an approved existing root; the relative source/destination examples resolve within that base. These commands create or update plan data and inspect proposed moves. They do not move the source files.

`plan create` creates a plan. `plan action add` adds a proposed move inside that plan. The command hierarchy distinguishes the plan from its individual actions:

| Object | Commands |
| --- | --- |
| Plan | `plan create`, `plan list`, `plan show`, `plan dry-run` |
| Action within a plan | `plan action add`, `plan action list`, `plan action show`, `plan action remove` |

```powershell
$planFile = "review.plan.json"
teamy-mover plan create --plan-file $planFile `
  --name "Review" --base-dir "<existing-root>"
teamy-mover plan action add --plan-file $planFile `
  --source existing.txt --destination desired.txt `
  --reason "Place the existing file in its reviewed location"
teamy-mover plan action list --plan-file $planFile
teamy-mover plan action show --plan-file $planFile --action-id 1
teamy-mover plan show --plan-file $planFile
teamy-mover plan dry-run --plan-file $planFile
teamy-mover plan list --directory "."
```

To withdraw a proposed action, remove it from the plan:

```powershell
teamy-mover plan action remove --plan-file $planFile --action-id 1
```

Removal changes plan metadata and preserves the previous plan revision. It does not remove the source file. Action IDs remain stable; use the ID returned by `plan action list`.

This prototype has no execution command. Its plan schema is the authoritative input format; the conceptual sketch above is for explanation. These examples make no release or publication claim.

## Keep the feedback loop short

Use one meaningful fixture to establish correctness, then expand when an unresolved failure or requirement justifies it. Observe errors through stable diagnostics and inspectable output. Avoid a dashboard that suggests precision when measurements are incomplete.

Document assumptions next to the behavior they affect. For example, an initial same-volume move implementation should say that cross-volume transfers need a separate integrity and recovery design.

See [repository hygiene](repository-hygiene.md) for build output and scratch ownership, and [the glossary](glossary.md) for the distinction between a goal, a plan and an approval.
