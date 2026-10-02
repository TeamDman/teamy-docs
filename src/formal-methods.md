# Check grammars, relations and state transitions

Choose the conclusion you need before choosing a formal tool. Parsing, constraint search, temporal model checking and proof checking establish different properties. Keep the model's assumptions and its connection to the real Rust implementation explicit.

Use this chapter to find Prolog, NuSMV, Alloy, PT Pascal, S/SL, syntax and semantics, Bend2 and the experimental Rust Bend rewrite.

## Separate grammar from meaning

A constrained grammar defines legal representations. Semantic validation checks what those representations mean in a particular domain. A parsed move can still refer to a missing source, stale object or forbidden destination.

The official [PT Pascal and Syntax/Semantic Language distribution](https://research.cs.queensu.ca/home/cordy/pub/downloads/ssl/) supplies public compiler prior art. Study the separation of scanning, parsing, semantic analysis and code generation. S/SL describes control while host-language mechanisms implement the underlying operations. The public listing is an older distribution; it does not identify or publish personal coursework.

For a Rust plan language, retain source spans and diagnostics through those stages. Test syntactically valid but semantically invalid examples alongside parser failures. Serializing a plan is not approval to execute it. See [plan records](plan-records.md).

## Query relations with Prolog

[Scryer Prolog](https://github.com/mthom/scryer-prolog/blob/e4d9692535c9dcffc09d58fe97fca4e0efe9de6b/README.md) is Rust implementation prior art for predicates, unification and logic-programming search. Grammar rules and constraints can express relationships between candidate objects without manually enumerating every imperative branch.

Keep the query, encoded facts and search assumptions together. A returned answer is relative to that program. It does not establish every property of the application or every possible search path.

## Find relational counterexamples with Alloy

[Alloy](https://alloytools.org/about.html) describes structures through relations and constraints. Use it to inspect possible structures and counterexamples. Our [graph model](https://github.com/TeamDman/alloy-refresher/blob/36c46116b371a57c8747c9e278789015a28d9804/models/intro.als) and [directory model](https://github.com/TeamDman/alloy-refresher/blob/36c46116b371a57c8747c9e278789015a28d9804/models/relational.als) provide concrete study examples.

Record the checked scope. Finding no counterexample within it is not an unbounded proof. An assertion that repeats an assumed fact does not demonstrate useful fault detection. Weaken an assumption or introduce a known bad model as a negative control.

## Check temporal behavior with NuSMV

[NuSMV](https://nusmv.fbk.eu/) checks properties of modeled transitions. Poche's [conformance record](https://github.com/TeamDman/Poche/blob/e5e767cc6b545b725994e50984d01d69091bface/docs/nusmv-conformance.md) compares a handwritten symbolic projection with an explicit Rust micro graph. Its [SMV model](https://github.com/TeamDman/Poche/blob/e5e767cc6b545b725994e50984d01d69091bface/models/nusmv/conformance.smv) makes the projection inspectable.

Retain named properties, environment assumptions and witness traces. Test stuttering and deadlock explicitly. Agreement within the declared projection does not establish renderer, networking or whole-game equivalence.

## Learn from the paused Rust Bend rewrite

The upstream [Bend 2 proof language](https://github.com/bendlang/bend) is a separate project from our port. `teamy-bend` is an experimental Rust rewrite, started from our CLI template. It is on hold and not ready for general use or as a complete replacement. Keep it as prior art; do not infer that an incomplete port is a supported application dependency.

At public revision [`ea34479`](https://github.com/TeamDman/teamy-bend/blob/ea34479b143643454991e28e621d0408ec92477c/README.md), the repository records a proof-checking subset, native evaluation, persistent typed calls and selected JavaScript/C generation. Its [implementation plan](https://github.com/TeamDman/teamy-bend/blob/ea34479b143643454991e28e621d0408ec92477c/docs/implementation-plan.md) tracks broader language, library, runtime and GPU work.

Full compatibility and CPU/GPU parity remain unfinished. The implemented checking kernel has not itself been formally proved sound. Passing fixtures establish behavior for that subset; they do not measure a percentage of the complete port.

The current local README explicitly marks development paused and the port not ready. The linked public revision predates that notice. Documentation of the experiment does not resume its implementation.

For model-generated plans, use these tools where their conclusions fit the task. An LLM can suggest a representation; deterministic checks still establish grammar, references and permitted state transitions. [Finite-choice models](finite-choice-models.md) supply scores, not proofs.
