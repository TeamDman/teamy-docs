# Bend references for CLI parsing and Windows GPU work

Keep these references for Bend research. The [experimental Rust rewrite](formal-methods.md#learn-from-the-paused-rust-bend-rewrite) has a historical pause and a requested resumption; see its linked records for the scope of verified CPU work. This note changes no dependency pin or adoption decision; [the Rust CLI template](rust-cli-template.md) remains our starting point for new Teamy CLIs.

Evidence reviewed on 26 October 2026: public documentation and branch metadata only. Neither repository's code or installers were run.

## Design a Bend CLI parser

[viridale/bend-cli](https://github.com/viridale/bend-cli) is CLI parser prior art. Its [README at `aa4fb94`](https://github.com/viridale/bend-cli/blob/aa4fb94a0f4dba49320913edf07f962dbbbb93f2/README.md) documents `Cli.cmd` / `Cli.arg` declarations with usage strings, separate typed decoding, and generated help, completions and man pages. It requires Bend 2.0.38 or newer; compatibility with our Rust rewrite is unqualified.

Our design direction is to build our own clap/Figue-style interface. The argparse comparison is our assessment of the API style, not an upstream classification. Retain this library for parser behavior and edge-case research; it is not our chosen dependency. See [argument parsing](cli-parsing.md) for the existing Rust approach.

## Investigate CUDA and Windows support

[MattCozendey/bend](https://github.com/mattCozendey/bend) is a fork of upstream Bend to revisit for future desktop GPU/Windows work:

| Branch | Snapshot when reviewed |
| --- | --- |
| [feature/cuda-and-windows](https://github.com/MattCozendey/bend/tree/feature/cuda-and-windows) | [README at `676492b`](https://github.com/MattCozendey/bend/blob/676492b71a3020e2be5630b6ed3afbe0ccc06da8/README.md) |
| [feature/cuda-and-windows-and-trust](https://github.com/MattCozendey/bend/tree/feature/cuda-and-windows-and-trust) | [README at `608043f`](https://github.com/MattCozendey/bend/blob/608043f4c35b7b0279be67be6aa6687c84dd440a/README.md) |

Both branches exist; their README describes Bend's CPU/GPU goals. This does not establish working Windows support or compatibility with our port. Quality, correctness and performance remain unqualified; these revisions record evidence, not adopted toolchain pins.

When work resumes, inspect each branch's changes against the intended upstream baseline, check Windows compiler/CUDA requirements, and validate a small CPU/GPU parity case before considering adoption.
