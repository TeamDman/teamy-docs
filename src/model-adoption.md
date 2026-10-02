# Implement a model in a local Rust tool

Start with one useful task and one reproducible fixture. Preserve the announcement's evidence, define the model contract, then build a terminal interface before adding a GUI. This is a proposed reusable workflow, grounded in our speech tools. It does not describe an existing universal model service.

For Jev-style decisions and local Julia-1, start with [score supplied choices](finite-choice-models.md). It names the models, provider alternatives and visual-input limits. A complete Rust example with passing assertions supplies a bounded implementation target.

Use [the Rust CLI template](rust-cli-template.md), [Python-to-Rust parity work](python-ml-to-rust.md) and [backend selection](gpu-inference.md). An announcement supplies a research lead. It does not establish that its model fits our task or that we may process every available input.

## Preserve the evidence before implementing

Follow the timeline URL to the publisher's model card, source, weights and reference implementation. [Find existing local source](source-lookup.md) before fetching another copy. Record an evidence manifest outside the public book when its inputs need a restricted audience.

When a spoken name is ambiguous, inspect relevant already-approved context before asking the user to repeat it. Open tab titles and URLs, selected page contents, local source metadata and public stars can identify the intended reference. A match establishes identity; it does not verify the page's performance claims. Name the browser coordinator and distinguish assigned ownership from enforced access controls.

| Evidence field | What to retain |
| --- | --- |
| Origin | Announcement URL, publisher and primary source URLs. |
| Time | Publication time when known and separate UTC retrieval time. |
| Identity | Code commit, model revision and tokenizer revision independently. |
| Artifact | Relative artifact name, byte count, checksum and acquisition status. |
| Archive kind | Raw response, rendered page, extracted text, repository snapshot or upstream archive reference. Record what each captures or omits. |
| Rights | Code, weights and dataset licenses; attribution and redistribution conditions. Unknown remains unresolved. |
| Scope | Audience, approved processing recipients and permitted input/output locations. |

A screenshot preserves appearance; extracted text preserves selected content. Neither proves that linked source or weights were captured. Keep source evidence, interpretation and measured results distinguishable. A checksum identifies retained bytes; it does not prove the publisher is trustworthy.

## Define the typed model contract

Record task kind, required files, preprocessing, tokenizer rules, input shapes, output interpretation and numerical tolerances. Include limits, device capabilities, precision and offline behavior. A compatible filename does not prove a compatible model.

Separate backend capabilities by task:

| Task | Contract to expose |
| --- | --- |
| Finite choice | Supplied option IDs and order, scores, selected ID, encoding limits and abstention policy. |
| Generation | Tokenizer/template identity, prompt limits, decoding settings, output limits and streaming/cancellation boundaries. |
| Speech | Audio or phoneme preprocessing, sample rate, alignment/output shapes and task-specific artifacts. |

Represent observations as a `DomainSnapshot` and suggestions as an `IntentProposal`. These are proposed application types, not existing shared library APIs. Include schema version, snapshot revision, input digest, allowed candidate IDs, model identity and outcome. Keep unknown observations explicit. Model scores need evaluation; they are not guaranteed certainty or permission.

Use [typed result conventions](output-shape.md) for completed findings. Define record framing and input limits for stdin and `@file`. The template's parser does not implement those command semantics automatically.

## Build a terminal service around one reusable runtime

Follow [the documented GPU direction](gpu-inference.md): Rust-owned numerical operations and native execution serve our control goals. Preserve its model and device limits. A proposed new backend comparison needs user confirmation; the Julia native/ONNX comparison is already approved. An existing graph can provide a bounded comparison adapter. State CPU and GPU support explicitly; adding an adapter does not create fallback in a GPU-only backend.

Keep the loaded model resident between requests. Report readiness after required loading and warmup, then expose typed requests and results through a documented local protocol. Define endpoint access, bounded queues, request IDs, progress, shutdown and model replacement. A local endpoint still needs an access policy.

Our prior art includes the transcriber's [typed domain transitions](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/domain.rs#L214-L234), [replayable event records](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/domain.rs#L735-L739) and [fake backend](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/transcription.rs#L73-L112). These provide patterns to test domain behavior without a model. They do not establish a generic service protocol.

Reuse [logging](logging.md) and [cooperative cancellation](cancellation.md). Logs do not replace typed outcomes or durable receipts. Cancellation checks do not interrupt a GPU operation already running.

## Reuse the existing local service deliberately

At [public revision `cc09503`](https://github.com/TeamDman/teamy-llm-service/tree/cc0950321a13cf6a8621c574d75cca664b150880), teamy-llm-service separates core Facet types, a Burn Qwen3.5 backend, a service facade and a local CLI. Its [generation contract](https://github.com/TeamDman/teamy-llm-service/blob/cc0950321a13cf6a8621c574d75cca664b150880/crates/teamy_llm_core/src/lib.rs#L124) carries text, thinking mode, token limits and timeout. This contract does not accept images or a constrained intent schema.

The facade [retains inspected artifacts and CUDA runtimes](https://github.com/TeamDman/teamy-llm-service/blob/cc0950321a13cf6a8621c574d75cca664b150880/crates/teamy_llm_service/src/lib.rs#L104) across requests and serializes work per model. Those mutexes do not establish FIFO fairness. Its health response sets `ready: true` without loading weights, so health alone is insufficient evidence of inference readiness.

Its [continuation-ranking operation](https://github.com/TeamDman/teamy-llm-service/blob/cc0950321a13cf6a8621c574d75cca664b150880/crates/teamy_llm_service/src/lib.rs#L414) scores supplied strings through causal likelihood. This can establish a baseline for candidate selection. Its normalized likelihood is not calibrated confidence, and the operation lacks an explicit cancellation or timeout parameter. A Julia finite-choice adapter should have its own encoding and result contract.

The [CLI currently dispatches arguments manually](https://github.com/TeamDman/teamy-llm-service/blob/cc0950321a13cf6a8621c574d75cca664b150880/crates/teamy_llm_cli/src/main.rs#L15). Reusing this service does not automatically provide every [template capability](rust-cli-template.md). Define the required command and backend integration explicitly, keeping an unrelated parser rewrite outside a bounded model experiment.

## Keep permission checks independent of inference

A deterministic application prepares the permitted snapshot before model processing. It validates schema versions, object IDs, current revisions, allowed operations and scope. Unknown policy does not become permission because a model suggests a destination or label.

Treat model output as untrusted proposal data. Validate it against the current snapshot and operation catalogue. Keep approval and execution separate, with OS-enforced capabilities where needed. Do not give a proposal service an arbitrary shell-dispatch interface. The LLM does not enforce shell authorization.

Schema and policy tests must work with the model absent. Test malformed input, unknown variants, stale revisions, unrecognised candidate IDs and denied scopes through deterministic fixtures. A well-formed proposal can still be forbidden. See [CLI effects and permissions](good-cli.md#effects-and-permissions) and [publication audiences](publication.md).

## Prove parity before changing performance defaults

Freeze the reference code, model artifacts and test inputs. Compare token IDs and preprocessing first, then intermediate tensors where observable, then final outputs. Record tolerances before accepting an optimization. Test ties, overflow, invalid inputs and representative task cases.

[Containerized Python references](python-reference-containers.md) provide a proposed Podman/uv/Torch recipe for producing independent fixtures. Pin the image and dependencies, verify the intended GPU and keep acquisition separate from offline reference execution. The recipe still requires model-specific preparation and execution validation.

Keep the reference independent of the implementation under test. Agreement between two wrappers around the same faulty path is weak evidence. Preserve failed comparisons. Finite output, stable length, numerical parity and task accuracy are different checks.

Record cold loading, first useful result, resident inference and complete application time separately. Include peak memory, request size, concurrency, device, runtime/provider versions and precision. Performance claims need the workload and correctness result beside them. [GPU inference](gpu-inference.md#measure-speed-together-with-correctness) describes our existing evidence limits.

## Budget memory and check technique compatibility

NVIDIA specifies [24 GB of memory for the RTX 4090](https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/rtx-4090/). That is capacity, not an application budget. Measure current availability and leave headroom for display use and other consumers.

Account for resident weights, tokenizer/host staging, activations, workspaces, concurrent requests and allocator retention. Autoregressive generation also needs its applicable [KV cache budget](https://huggingface.co/docs/transformers/main/en/cache_explanation). Do not apply generation-specific cache assumptions to a finite-choice encoder.

Quantization, reduced precision, fused attention, batching and offload are separate candidates. Check model/operator support, artifact format, hardware, runtime and kernels before selecting them. For ONNX CUDA, verify the pinned runtime's [CUDA and cuDNN compatibility](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html). A technique's name or a smaller weight file does not establish parity, total memory use or speed.

## Use Julia as a bounded evaluation example

The publisher describes [Julia 1 at revision `a85b127`](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/README.md) as a 144.3-million-parameter mmBERT-based decision model. Each model call scores 2 to 20 supplied options. It is a finite-choice model, not a text generator.

Its separate [ONNX and WebGPU release at `82a2fad`](https://huggingface.co/SupersonicLabs/Julia-1-ONNX/blob/82a2fadf8fccfccdc5fd4e1009ba8f1a265eb7a8/README.md) includes Rust WebAssembly and N-API tokenizer source. That tokenizer does not establish an existing native Rust inference service.

The first slice is the tested finite-choice contract, followed by independent reference fixtures and native/ONNX adapters before GPU optimization. [The Rust `ort` binding](https://github.com/pykeio/ort) and [ONNX Runtime's C API](https://onnxruntime.ai/docs/get-started/with-c.html) are implementation candidates. The binding runs ONNX Runtime's C/C++ library: this removes Python from product inference without making the numerical implementation entirely Rust. Pin and verify the chosen runtime, graph operators and external weight files together. Julia has not been run or validated as part of this documentation work.

## Join parallel work at explicit checkpoints

Evidence capture, schema/policy fixtures and the terminal protocol skeleton can proceed in parallel once scope and audience are agreed. Join them when source identities, the model contract and reference fixtures are complete. Then integrate the real adapter and compare outputs.

Measure performance after parity, then record the accepted backend and its limits. A GUI can consume the same tested protocol afterward. It should not duplicate model loading or become the only place where policy is checked. [Makepad's patterns](makepad-patterns.md) supply study references; choosing its GUI framework remains a separate decision.

Keep a personal skill short: route a new announcement to this workflow, load the relevant chapters, identify the next missing artifact and resume from recorded evidence. Put maintained technical explanations in the book rather than copying them into every skill or project.
