# Keep a migration aligned and fast

Give a repeated migration a short, verifiable loop: ask the tooling for the next item, transform it, validate it, and repeat. Derive progress from the current source and checks. Keep the original goal and completion criteria visible throughout the work.

Our SFM source migration exposed two failures. A goal alone did not prevent drift. Adding burndown tooling restored direction, but slow tooling and excessive coordination still limited throughput.

## Learn from the SFM migration

SFM's migration reconstructs shared Liquid source from existing Java implementations across several version and feature contexts. The oracle compares generated source with pinned references and identifies remaining work.

| Failure | Evidence and consequence | Remediation |
| --- | --- | --- |
| The agent drifted from the original conversion goal | The operator reported drift during sustained migration work. Deciding what to do next and maintaining progress reports became recurring agent work. | Put remaining-work discovery, priority and next-item selection in deterministic tooling. Keep the goal's scope and correctness checks fixed. |
| The new core loop was still slow | The operator reported one `source oracle next` invocation taking 80 seconds. The reviewed implementation calls full `status` before selecting one item. | Profile the selector, remove repeated computation and revalidate only affected work where correctness permits. |
| Reasoning and handoffs added overhead | The operator identified Ultra reasoning and subagent communication as contributors. A workflow review also identified small handoffs, repeated reviews and checkpoint ceremony. | Trial simpler execution for routine transformations. Measure accepted work, elapsed time and usage, including integration and repairs. |

The 80-second run establishes an expensive observed selector call. It does not isolate the cost of reasoning or prove a model comparison. The assessment of coordination overhead is useful evidence for an experiment, not a quantified speedup result.

Assess these delays alongside accepted migration work during the same interval. Profiling and optimization can happen while conversions continue. An expensive selector call alone does not establish that optimization stopped the main task.

At that reported latency, 100 selector calls alone take 2 hours, 13 minutes and 20 seconds. This is arithmetic for a repeated 80-second cost, not a measured migration total.

## Put next-item selection in the tool

Keep one authoritative goal and [decision record](plan-records.md). Encode required contexts, ordering, exceptions and completion checks in the migration tooling. Historical conversations explain earlier choices; they must not become new prerequisites or replace the active goal.

A selector should return a bounded item or coherent batch, explain its priority, and expose blockers. Distinguish actionable work, incomplete observation, stale state and verified completion. These are design requirements; they are not a claim that every existing selector exposes this exact interface.

Follow this loop:

1. Measure the selector against the current source and intended repository/reference identities; record its selected work.
2. Perform its bounded transformation using the established patterns.
3. Validate affected contexts and relevant feature boundaries.
4. Measure the selector again on the changed state to verify the previous item and identify remaining work.
5. At a coherent checkpoint, run the full correctness checks and save a concise generated receipt.

The local oracle-enabled SFM CLI supplies the command shape:

```powershell
# From the matching SFM checkout, using a build that contains source oracle.
sfm-propagate-changes.exe --output-format json source oracle next --repo-root "."
```

Check the executable's help and build identity before relying on this local command. This example does not establish that a published release contains it.

Count accepted work rather than messages, drafts or tool calls. Label unique files and file/context comparisons separately. A source match, generated-output freshness, compilation and runtime validation establish different things. The selector reduces bookkeeping; it does not replace those checks or authorize publication.

## Measure the complete iteration

Use [tracing and performance analysis](performance-analysis.md) to measure selection, transformation, validation, coordination and integration separately. In a serial loop, their non-overlapping durations contribute to total iteration time. With parallel work, measure elapsed time and the critical path instead of adding overlapping spans.

For SFM's selector, begin with coarse phases: startup, input discovery, Git reads, Liquid rendering, comparison, output inspection and next-item selection. Use `--log-file` timing captures where supported, or Tracy with [teamy-profiler](performance-analysis.md#capture-and-inspect-with-teamy-profiler). Add finer instrumentation where the capture identifies a bottleneck.

Record accepted units per elapsed hour and usage per accepted unit where usage is available. Include repairs and newly introduced regressions. A faster first attempt that requires more correction can reduce throughput.

The SFM performance target is under one second for routine repeated selector calls, including calls after representative edits. Measure fresh process startup through output and exit. Keep cold index creation separate and retain failed samples. No subsecond selector result is established by this chapter.

Encode a reasonable duration in the selector itself using the template's [simple runtime warning pattern](performance-analysis.md#make-slow-commands-warn-during-normal-use). That makes an unexpectedly slow iteration visible during normal operation, before repeated calls accumulate hours of overhead. A warning complements measured optimization and correctness checks; it does not cancel work or establish cache freshness.

[Optimize the selector while advancing source conversion](performance-analysis.md#optimize-while-advancing-useful-work): measure the next-item command, complete and validate its work, then measure the command on the changed state. Use those observations to improve the tooling in parallel or at bounded checkpoints. Repeated queries on an unchanged state can diagnose performance, but cannot qualify fresh selection after completed work.

[Bound the optimization effort](performance-analysis.md#bound-optimization-while-advancing-dependent-work) with a measured speed target and an allowance converted once into an absolute UTC end DateTime. Preserve it across resumes and check it with `Get-Date` before experiments. Continue conversions throughout; stop additional optimization when the target is met or the cutoff arrives. Retain the best validated selector and record accepted migration progress, a missed speed target and any correctness blocker separately.

## Reuse computation without hiding changed work

Choose algorithmic improvements from the profile. Reuse unchanged inventories, parsed templates, pinned Git objects and comparison results. Maintain dependencies so an edit invalidates the contexts it can affect. Keep expensive whole-matrix checks at explicit checkpoints rather than silently repeating them for every filename request.

A persisted index or cursor can help, but a cursor alone is insufficient. Revalidate the previously selected item before advancing. Detect additions, deletions, changed configuration and regressions in earlier items.

Bind cache entries to repository and reference identities, relevant contents, feature/context configuration, comparison rules and tool/schema versions. File timestamps are useful discovery hints; they are not sufficient evidence that contents remain unchanged.

Update persistent state atomically and recover explicitly from missing, corrupt or interrupted state. Keep derived cache data outside authored and generated source. Preserve a cache-bypassing full comparison as the correctness reference.

Compare incremental and full results across unchanged calls, typical edits, earlier regressions, preserved-timestamp edits, context changes, additions and deletions. A stale cached count must never establish completion. Report what was revalidated and what remains historical or unobserved.

## Match reasoning and delegation to the work

For routine transformations with known source, established patterns and strong checks, trial a lower reasoning setting on a coherent batch. Keep the same acceptance gates. Escalate for concrete uncertainty such as optional-feature ownership, comparator correctness or unexplained cross-version failures.

Lower reasoning effort can reduce latency and token usage, but supported settings and results vary by model. [Official OpenAI reasoning guidance](https://developers.openai.com/api/docs/guides/reasoning#reasoning-effort) explains that tradeoff; it does not establish which setting wins on this migration.

Use delegation when independent work can save more time than assignment, duplicated context, handoff, review and integration consume. Compare with a single-agent batch. Avoid many small handoffs around serial transformations or shared metadata. Higher reasoning and more agents are options to evaluate, not measures of progress.

Record model and reasoning settings with the experiment. Keep goal text focused on outcomes, priorities and acceptance criteria; harness settings belong in execution configuration.

## Inspect the prior art

[SuperFactoryManager](https://github.com/TeamDman/SuperFactoryManager) is the migration reference. In the local source-projection checkout reviewed on 4 October 2026, inspect `platform/cli/sfm-propagate-changes/src/source_projection/oracle.rs`: `next` calls `status`, then `next_from_status` selects unresolved work. `status` discovers inputs and compares the required contexts. The command dispatch lives in `src/cli/source/oracle_cli.rs` within that CLI package.

This source observation explains why the selector repeats broad work. It does not prove which phase accounts for most of the 80 seconds. Performance work in that checkout is ongoing. Read the matching implementation and current measurements before adopting an optimization.

This chapter retains the reusable lessons. Raw conversations, local paths and operational receipts remain outside the public book under the [publication boundary](publication.md).
