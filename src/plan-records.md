# Review plans and retain decisions

Make a plan concrete enough to review before executing it. Separate proposed actions, review comments, accepted decisions and execution receipts. Each has a different lifecycle and authority.

For new tools, start with [our Rust CLI template](rust-cli-template.md). Use explicit commands and versioned [typed results](output-shape.md). Our [local mover prototype](moving-files.md) supplies prior art for strict plan parsing and ordered simulation; it has no executor.

## Adopt typed records gradually

| Stage | Required evidence |
| --- | --- |
| Readable outline | Stable requirements, scope, assumptions, decisions and source references. |
| Versioned data | Required fields, object identities, unknown-version behavior and explicit migrations. |
| Rust contracts | Actual Facet types, strict parsing, semantic validation and round-trip fixtures. |
| Executable examples | The documented invocation runs against the intended build and a synthetic fixture. |
| Public decision record | Approved fields and sources, a named audience and a publication receipt. |

Serialization proves a record can be read. It does not establish that a source exists, a destination is safe, an action is authorized or the plan is current. Test those domain properties separately.

A decision record should retain its status, scope, options, selected option, rationale, evidence, deciding authority and acceptance time. Give it a stable ID and a `supersedes` relationship when a later decision replaces it. A recommendation remains proposed until the relevant authority accepts it.

The record shape is a design contract to establish for each tool. This book does not yet supply a shared plan-record library or schema.

## Attach feedback to the reviewed text

A passage comment needs the document identity and revision, quoted text, surrounding context and a position within its source block. Keep thread messages and resolution status separate from a section's overall response.

If the source changes, report a stale or ambiguous anchor. Do not silently attach the comment to another paragraph. Retain the original quote so the reader can recover its meaning. A screenshot annotation should also identify its source capture and viewport; coordinates alone cannot identify a passage after layout changes.

Review exports and portable reviewed documents need their own schema versions and input limits. Render comments as text. Keep review data separate from executable actions and from [approval](glossary.md).

## Know what the book validates

The current [build script](https://github.com/TeamDman/teamy-docs/blob/b5cc4a110e32a19fe6e24908cc4c9b28506c93f5/scripts/build.ps1) verifies the pinned mdBook binary and builds the book. The [link checker](https://github.com/TeamDman/teamy-docs/blob/b5cc4a110e32a19fe6e24908cc4c9b28506c93f5/scripts/check-links.ps1) checks generated local targets and fragments. It skips external URLs.

The [deployment workflow](https://github.com/TeamDman/teamy-docs/blob/b5cc4a110e32a19fe6e24908cc4c9b28506c93f5/.github/workflows/pages.yml) runs those checks. It does not run `mdbook test` or compile every code fence. A successful Pages deployment proves publication of the built artifact, not correctness of every example.

[mdBook testing](https://rust-lang.github.io/mdBook/cli/test.html) can test Rust examples. [Rustdoc attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) distinguish runnable tests, `no_run` examples that compile, and `ignore` examples that are skipped. This does not validate PowerShell commands, JSON fixtures or conceptual notation.

The [finite-choice example harness](finite-choice-models.md#start-with-a-compiling-contract) compares one complete book snippet against the actual local service-core example. It runs core tests and executable assertions offline. This establishes a bounded checked example; it neither runs model inference nor extends Pages CI to every code fence. The matching service API is currently a local addition.

Broaden validation gradually. Record each example's purpose, dependencies, edition, target and validation mode. Parse actual JSON/schema fixtures and run CLI examples against synthetic inputs. Label contextual fragments and conceptual sketches explicitly; report them outside the verified coverage count.

## Publish an approved projection

The book contains public unclassified guidance. Actual inventories, private prompts, local paths and review comments belong in private records unless separately approved for that audience. A plan's existence does not make it public.

Prepare public decisions from approved fields and public sources. Review both source and generated output, including search indexes and copied assets. Keep a reference to the private decision's identity when appropriate without disclosing its private contents. Follow [publication and audiences](publication.md).

Deterministic checks can catch known secret formats, identifiers and path patterns. A model can flag contextual sensitivity or suggest a redaction, with evidence and uncertainty. Neither method guarantees complete detection. A classification result does not authorize publication.

Before adopting a classifier, define its labels, audience, representative examples, false-negative cost and adjudication rule. Evaluate held-out records before fine-tuning. Our [model-adoption workflow](model-adoption.md) supplies the contract and measurement route; model selection remains separate from release authority.
