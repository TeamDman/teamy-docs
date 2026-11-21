# Prompts and skills

Teamy Docs is the broader reference for useful prompts, their purpose and their sources. Keep a short prompt short. Package it as a skill only when repeated use needs additional instructions or supporting tools. [TeamDman/skills](https://github.com/TeamDman/skills) holds the reusable workspace skills; their executable instructions live there, while this book retains the wider explanation and links.

## Find hidden state machines

[Trevin Chow (@trevin), 6 November 2026](https://x.com/trevin/status/2118690987153186845), expanded [Steve Ruiz (@steveruizok), 31 October 2026](https://x.com/steveruizok/status/2116552016218710389): "find all of the implicit state machines in my app".

> Find the app’s explicit and implicit state machines and document their states, invariants, and transition tables in one file. Identify invalid states or transitions the code can actually reach, with evidence, including through retries and concurrency. Prioritize confirmed bugs by impact, group related fixes into issues, then fix them with regression tests and keep the document updated.

This is Trevin's prompt, preserved as a small copyable snippet. Use it to investigate lifecycle bugs that a happy-path walkthrough can miss. Scope the target repository and permitted changes before running it; the quoted request to create issues and fix bugs does not override the current task's authorization.

Source check: the wording and quote relationship were verified on 6 November 2026 through indexed copies of [Trevin's post](https://teapawt.org/trevin/status/2118690987153186845) and [Steve's post](https://teapawt.org/steveruizok/status/2116552016218710389). The original X links are retained above; direct X retrieval was unavailable. This records provenance, not a claim that the prompt always finds real bugs.

## Describe system behavior through CLI commands

Teamy's complementary framing: everything the system does should map to CLI commands. Describe the behavior of the system with CLI commands, including how to observe and control the whole app.

A short companion prompt:

> Map the system's behavior and control to CLI commands. For each state machine, show how commands observe its state and trigger its transitions. Distinguish commands that exist from proposed commands, and identify behavior that cannot yet be observed or controlled through the CLI.

Use the state-machine document to connect commands to preconditions, effects and observable results. Include background work and recovery as well as user-triggered actions; describe automatic transitions without pretending each requires a manual command. A command map should expose missing controls and unclear behavior before implementation.

See [command-line tools](good-cli.md), [typed results and errors](output-shape.md), and [reviewable effects](writing-programs.md#make-effects-reviewable) for the implementation contracts. This companion framing is separate from Trevin's quoted prompt; it is not attributed to either tweet author.

## Benchmark local LLM inference

[Inference optimisation](https://github.com/sybil-solutions/inference-optimisation-skill) is an external agent skill from Sybil Solutions for tuning a local LLM server. It measures startup, time to first token, prompt processing (prefill), and generation throughput for one or multiple users. It also covers KV-cache sizing and speculative decoding.

The useful pattern is to measure a baseline, estimate the performance ceiling, and compare one change at a time against interleaved control runs. Keep an experiment ledger and check the best configuration at the target context length and memory limit before adopting it. For a Teamy benchmark, pair that approach with [GPU inference correctness and timing](gpu-inference.md#measure-speed-together-with-correctness) and [resident inference](resident-inference.md).

Status: external reference only. We have not adopted the skill, run its harness, or reproduced its performance claims.

Discovery source: [@0xSero on X](https://x.com/0xSero/status/2124238286457917649). Use the original thread for feedback if a benchmark makes the skill worth recommending.
