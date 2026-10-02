# Reuse Makepad's inference patterns

Study Makepad when we need control of model operations, device memory and a resident inference session. Its LLM implementation provides concrete prior art for those goals. Adopting a pattern does not require adopting its GUI framework. Compare it with [our GPU strategies](gpu-inference.md) before choosing an integration boundary.

This chapter was reviewed on 1 October 2026 against public Makepad revision [`3b1be702`](https://github.com/makepad/makepad/tree/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1), committed on 25 September 2026. That is the source snapshot date, not a benchmark date or a claim that every inference component changed that day.

## Understand the LLM execution path

`makepad-ai-llm` loads GGUF model metadata, tokenizer data and weights. Rust builds its own execution graphs and binds them to Metal on macOS/iOS or CUDA on Windows/Linux. It controls graph planning, caches, buffers and token decoding; CUDA kernels and GPU libraries still perform numerical work. This is a source-defined graph runtime, distinct from handing an exported ONNX graph to ONNX Runtime. See the [model crate](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/Cargo.toml#L1-L19) and [execution seam](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/exec.rs#L1-L18).

The session exposes separate stages: parse, vocabulary and plan construction, weight mapping or reading, device initialization, cache allocation, memory reservation, upload and graph preparation. CUDA uploads the weight region once; Metal can wrap the host arena. Prompt tokens then enter batched prefill, followed by repeated token selection and decoding to text. See [session preparation](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/session.rs#L3725-L3888), [device buffers](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/exec.rs#L148-L165) and [generation](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/session.rs#L1301-L1330).

Its in-process chat worker creates and keeps the session on one thread. Appended turns reuse conversation state. Loading progress, readiness and generated events reach the consumer through channels; cancellation is checked between generated tokens. Those channels are unbounded, so copy the ownership pattern only with an explicit queue policy for our service. See [worker ownership and channels](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/hub/src/local_llm.rs#L1-L14), [startup](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/hub/src/local_llm.rs#L135-L171) and [token cancellation](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/hub/src/local_llm.rs#L441-L475).

## Distinguish vision and fallback capabilities

The separate Qwen-VL vision tower loads an `mmproj` GGUF, preprocesses RGB pixels and produces image embeddings for LLM prefill. Supported text-and-image inputs can therefore lead to generated text. The cited LLM path does not generate images. This does not add vision to a text-only model such as Julia; it requires a compatible vision tower, projector and language model. See [vision preprocessing](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/vision.rs#L1-L24) and [embedding injection](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/session.rs#L1787-L1813).

Fallback belongs to a particular path:

| Path | Observed behavior |
| --- | --- |
| LLM execution | Missing native backend, kernels or memory produces an error; there is no CPU or cross-backend fallback. [Backend selection](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/exec.rs#L71-L94). |
| Vision tower | Unpinned selection tries CUDA, then Metal; explicitly pinning a backend makes its failure an error. [Selection](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/vision.rs#L426-L469). |
| Whisper primitives | An unavailable accelerator returns control to the caller's CPU implementation. [Dispatch](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/models/speech/src/whisper/accel.rs#L1-L25). |

## Separate compilation from preparation

Makepad's CUDA build invokes `nvcc` and archives kernel objects. Runtime graph preparation and CUDA graph capture are additional stages. A failed capture can retain eager CUDA dispatch; that is not CPU fallback. Metal supports precompiled shaders and runtime source compilation when precompilation is unavailable. See [CUDA build](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/cuda/build.rs#L229-L372), [capture handling](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/llm/src/cuda_exec/real.rs#L2402-L2405) and [Metal build](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/ai/metal/build.rs#L106-L153).

The inspected Teamy speech manifests do not declare Makepad as a dependency. The transcriber's [implementation ledger](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/PLAN.md#L42) records burnt-apple's Burn frontend and loader as its original native Whisper reference. Preserve both that lineage and Makepad's role as study material. Source comments explain design choices; acceptance needs our own [parity and performance evidence](gpu-inference.md#measure-speed-together-with-correctness).
