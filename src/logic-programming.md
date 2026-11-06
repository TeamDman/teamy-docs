# Explore relationships with Prolog

Prolog is interesting here as a way to state relationships and ask for solutions. A Sudoku model describes cells and row, column and box constraints; search can find a completion, another completion, or show that the stated constraints have no solution. A contradiction in a partially filled grid concerns the model and supplied entries together: it does not identify the wrong entry by itself. Check the clues and add entries incrementally to locate a conflict.

[Scryer Prolog](https://www.scryer.pl/) is a Prolog implementation written mostly in Rust. Its inspected [README at `b7066b4`](https://github.com/mthom/scryer-prolog/blob/b7066b41f1474cf5ddf7d813bf1987d3cd571ccb/README.md) includes a Sudoku constraint example and browser embedding example, and points to its [Codeberg repository](https://codeberg.org/mthom/scryer-prolog). This is study and experiment prior art, not an adopted dependency. Test embedding, versioning, resource limits and error handling before use; implementation in Rust does not itself guarantee a suitable embedding API.

## Model a planning query

For a small experiment, export a finite set of registered operation facts, their input dependencies, outputs and explicit capabilities. Query which chains could produce a target from known seeds, then display the assumptions. Ask for alternative solutions on a bounded graph; guard against cycles, unbounded search and duplicate results. Keep effectful calls outside search: backtracking over an API write or file transfer could repeat it. A proposed chain still requires validation and authorization before execution. [Typed registry planning](decision-planning.md) describes the existing non-Prolog baseline and a transfer example.

Prolog can reason about supplied facts, but an opaque Rust function hides its internal rules. A string error does not automatically explain failure in machine-readable terms. Structured errors with a category, source span and expected/observed values support further analysis. Compare any translated validator against the original with positive and negative fixtures.

## SuperFactoryManager connection

A possible future experiment is to use a solver to suggest a logistics decision from a snapshot, then validate and apply it through an explicit client/server contract. World state may change between observation and action; source identity, authority, replay and conflicts matter. This is a proposed experiment, not existing Prolog integration. See [Minecraft client automation](minecraft-client-automation.md).

As adjacent design context, [SuperFactoryManager issue 436](https://github.com/TeamDman/SuperFactoryManager/issues/436#issuecomment-5199376149) discusses exploring recipe/registry information and generating valid verbose programs instead of extending the grammar with loops and functions. That discussion does not establish Prolog use.

A public indexed search for Prolog and Scryer in SuperFactoryManager issues returned no matching evidence in this review. It was not an exhaustive search of discussion bodies or comments; no conclusion about their absence follows. Record direct public issue or discussion links if found, with their context and status.
