# Curate prior art by problem

Record a useful solution under the problem it solves. A reader asking “add a tray icon” should reach the tray chapter, then Piing or tb. They should not have to recognize either repository name first.

Use [the problem index](problem-index.md) to discover our established approaches. A chapter should explain what worked, why we keep it, where it lives in code and what needs adapting.

## Write a reusable implementation reference

| Field | What to record |
| --- | --- |
| Problem | The operation or outcome a person will ask for. Include common search wording. |
| Solution | Our Rust approach and the boundary where it joins the CLI template or other shared code. |
| Evidence | Public repository, immutable revision, source module and relevant test or measurement. |
| Status | Adopted implementation, published branch work, local prototype or candidate for evaluation. |
| Limits | Platform assumptions, unsupported behavior and what the evidence does not establish. |

Link one implementation from several topics when appropriate. Piing belongs under tray icons, window events and log replay. The speech tools belong under GPU inference, Python-to-Rust ports and performance validation. This is a problem-to-implementation relationship, not an exclusive project category.

Source inspection demonstrates design. A successful test demonstrates its checked behavior. A benchmark demonstrates performance for its recorded workload and machine. Keep those claims distinct, and link the evidence used.

## Use stars as discovery input

GitHub stars record interest and can surface references worth reviewing. They do not establish adoption or suitability for every project. The [GitHub starring API](https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-a-user) supplies repository metadata for this discovery step.

On 1 October 2026, we sampled 600 recent public stars and reviewed topic candidates. This was a partial snapshot, not a complete account export. The following references have a specific place in the book:

| Reference found in the sample | Why to inspect it | Status and next check |
| --- | --- | --- |
| [cudarc](https://github.com/chelsea0x3b/cudarc) | Rust CUDA APIs, kernel launches and device-memory transfers for [GPU inference](gpu-inference.md). | Use the backend decisions and exact dependency version in the consuming project; current upstream examples are not automatically compatible with that pin. |
| [PINTO model zoo](https://github.com/PINTO0309/PINTO_model_zoo) | Conversion recipes across ML frameworks for [Python-to-Rust ports](python-ml-to-rust.md). | A research candidate. Check the specific model, preprocessing, export operators, license and parity before adoption. |
| [Windows Terminal shaders](https://github.com/Hammster/windows-terminal-shaders) | Examples of terminal appearance effects. | A visual reference for [terminal work](terminal-integration.md), not a terminal parser or PTY implementation. |

Add a recommendation after evaluating it for a named problem. Record what we adopted and what remains a candidate. Keep broad star dumps, local checkout inventories and private review records outside the public book.

## Maintain the connection to code

When a project gains a capability, update the topic's implementation reference and our [problem index](problem-index.md). If reusable code moves into the template or a library, update its availability there. An old copied scaffold does not gain new capabilities automatically.

When a new ML announcement appears, begin with [porting Python ML to Rust](python-ml-to-rust.md), then [backend selection](gpu-inference.md). Locate and pin its sources, establish correctness and measure a real workload before promoting it into our preferred approach.
