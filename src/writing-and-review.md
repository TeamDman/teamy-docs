# Write and review precise prose

Use these skills to remove filler and repetitive AI writing patterns from documentation, release notes, PR descriptions and other prose. Keep the supported meaning and the writer's voice. For technical material, prefer neutral wording, observable behavior and exact quantities.

## Choose a review

| Need | Skill and source | Evidence and role |
| --- | --- | --- |
| Audit stock phrases, staged contrasts, repetitive cadence, padding and document structure | `de-slop` from [llm-smells](https://github.com/shitijkarsolia/llm-smells/blob/a3c4da34f1f22f954d08317662a9df178d30c50a/SKILL.md) | Inspected revision `a3c4da34f1f22f954d08317662a9df178d30c50a`. Broad checklists for prose, comments, formatting and visual defaults. The upstream skill name is `de-slop`, although the repository and README use `llm-smells`. |
| Rewrite prose while preserving facts, technical meaning and a supplied writing sample | [Humanizer](https://github.com/blader/humanizer/blob/8c307f367c91bd974002f10f2be2965d642c13df/SKILL.md), skill name `humanizer` | Inspected version 3.1.0, revision `8c307f367c91bd974002f10f2be2965d642c13df`. Prose-focused review of 26 patterns; unsupported added details and lost claims are errors. |

Both skills are imported for prose review. Their source files and licenses have been inspected; their effect on writing quality has not been benchmarked. A style audit does not establish factual accuracy, authorship or software correctness. Choose the review that fits the document.

`de-slop` uses draft, audit, rewrite and verification passes. In review mode, it reports the location, reason and concrete replacement for each issue. Humanizer preserves supported claims, compares the rewrite with the source and uses a writing sample when supplied. Technical and reference prose stays plain and neutral.

## Review a technical release report

For each change, state the situation, observed behavior and expected behavior. Name the component or operation that performs the action. Explain what changes for the user, then give the evidence needed to assess it.

- Replace personification and metaphors with the actual operation: a connection receives a notification, a cache returns a stale value, or a handler skips an update. Identify which one the source demonstrates.
- Preserve exact versions, identifiers, counts, units, timings and conditions. Do not replace a recorded number with "many", "faster" or "significant".
- Distinguish measured results from expected behavior and untested possibilities. A successful example establishes that example's result.
- Remove praise, staged openers, manufactured contrasts and sentences that repeat a heading or example without adding a fact.
- Verify the rewrite against source changes, issue reports and test results. If a necessary detail is missing, retrieve it or state the narrower supported claim.

A synthetic example: replace "Let the cache remember when settings change" with "Invalidate cached results after configuration reload" if the implementation confirms that operation and trigger. The accompanying explanation still needs the triggering situation, previous result, new result and validation. Do not infer the implementation from the title.

For example, ask:

```text
Apply de-slop to review this release report. Report the location,
the unclear claim and a concrete rewrite for each issue.
Preserve every supported identifier, quantity and condition.
For each change, check situation, observed behavior, expected behavior
and validation. Flag missing evidence; do not invent it.
```

For a prose-focused rewrite, invoke `humanizer` with the source text and a representative writing sample. Keep identifiers, code, data, frontmatter and link targets intact unless the task explicitly changes them.

## Retain provenance when installing

Keep a canonical imported directory for each skill in the skills checkout:

```text
<skills-checkout>/.agents/skills/de-slop/
<skills-checkout>/.agents/skills/humanizer/
```

Each imported directory contains:

- The unchanged upstream `SKILL.md` and MIT `LICENSE`.
- `source.link`, with the upstream skill URL and its immutable revision URL, following the existing imported-skill sidecar convention.
- `source.md`, with the repository, revision, original file paths, import date, copyright holder, SHA-256 hashes and local-change status.
- For Humanizer, the unchanged upstream `agents/openai.yaml` display metadata.

The skills checkout's top-level license does not replace these upstream MIT notices. The llm-smells research archive and the Humanizer plugin manifests are outside this portable skill import.

Expose each canonical directory through a per-skill Windows junction under `%USERPROFILE%/.agents/skills/`, with the same folder name as its frontmatter `name`. Follow the existing skills checkout's installation process: inspect the source and destination, create a junction only when the destination is absent, then check its `LinkType`, target and readable `SKILL.md`. Preserve unrelated destinations. A listed link name alone does not confirm a working installation.

This arrangement keeps one canonical copy to review and update. Import a new upstream revision deliberately, compare its behavior and license, refresh the attribution record and verify the profile links again. Installation status belongs in the private installation receipt; this public chapter describes the reusable arrangement.

## Sources and credits

- [llm-smells README](https://github.com/shitijkarsolia/llm-smells/blob/a3c4da34f1f22f954d08317662a9df178d30c50a/README.md) credits Shiv's [Various LLM smells](https://shvbsle.in/various-llm-smells/) and the accompanying [Hacker News discussion](https://news.ycombinator.com/item?id=48313810). Its [MIT license](https://github.com/shitijkarsolia/llm-smells/blob/a3c4da34f1f22f954d08317662a9df178d30c50a/LICENSE) names copyright 2026 shitijkarsolia.
- [Humanizer README](https://github.com/blader/humanizer/blob/8c307f367c91bd974002f10f2be2965d642c13df/README.md) identifies Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) as its basis. Its [MIT license](https://github.com/blader/humanizer/blob/8c307f367c91bd974002f10f2be2965d642c13df/LICENSE) names copyright 2025 Siqi Chen.

See [curating prior art](curating-prior-art.md) for evidence and adoption status, and [publication and audiences](publication.md) for the public documentation boundary.
