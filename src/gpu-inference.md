# Run GPU-enabled inference

For a new Teamy tool, follow this chapter and [the Rust CLI template](rust-cli-template.md). The proposed inference default is source-defined Rust operations with explicit GPU execution, where the model and required operators support it. That matches our goals of inspectable operations, memory ownership and lifecycle control. Its performance advantage remains a question for measurement.

Keep a compatible independent reference and use [Python-to-Rust porting](python-ml-to-rust.md) to preserve behavior. Follow the documented approach before starting a separate provider study. Ask the user before a new alternative-provider comparison; the existing [Julia native and ONNX comparison](finite-choice-models.md) is already within its approved scope.

## Match the backend to the problem

Rust is the application language in several of these strategies. It does not identify who implements the numerical runtime.

| Strategy | Weights and operations | Scheduling and kernels | Capability and fallback |
| --- | --- | --- | --- |
| Rust with custom CUDA | Our native speech tools load external safetensors; Rust defines model operations. | Our code owns execution order and workspaces; custom kernels and cuBLAS, plus cuDNN in TTS, do numerical work. | The cited native transcriber requires NVIDIA CUDA. Precision, unsupported devices and allocation failure need explicit handling. |
| Makepad's source-defined graph runtime | GGUF weights and Rust graph builders for supported Qwen architectures. | Makepad plans graphs and device memory, then dispatches Metal or CUDA kernels. | Its LLM path errors when the platform's native GPU backend cannot run. [Makepad details](makepad-patterns.md). |
| ONNX Runtime through Rust bindings | An exported graph specifies operators, with embedded or external weights. The application owns encoding and output interpretation. | The engine optimizes and partitions graphs; execution providers own supported kernels and their internal execution. The application still owns session lifetime and request queues. | Provider priority can put unsupported nodes on CPU. Confirm placement and fallback policy. [Official provider architecture](https://onnxruntime.ai/docs/execution-providers/). |
| Rust over a tensor framework | Rust can define layers over LibTorch through tch or over Burn. TorchScript can instead retain an exported graph. | The framework and backend implement tensor execution and kernels; application code retains preprocessing and decoding. | Runtime, operator and device support depend on the chosen backend. The [main transcriber's direct runner](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/native_whisper/tch_safetensors.rs#L1) still needs LibTorch. |

These are ownership tradeoffs, not a speed ranking. Source-defined inference can still use graphs and native libraries. An ONNX wrapper removes Python from product inference while retaining ONNX Runtime's C/C++ engine. A handwritten kernel needs a correctness contract and a measured purpose.

## Reuse resident runtimes and meaningful readiness

The public native transcriber branch `codex/native-whisper`, revision [`7d1ec22`](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/native/README.md#L3-L19), defines Whisper in Rust and custom CUDA. Main at `d87d502` still uses tch/LibTorch. The native branch's documented empirical support is Windows x64 on an RTX 4090; other devices and platforms need validation.

Its runtime has concrete phases:

1. A dedicated worker loads the engine, device weights and frontend, then signals readiness. Its request queue has capacity one.
2. Audio windows become mel inputs. The engine runs encoder and prompt prefill, then token decoding with resident caches and workspaces.
3. Completed transcripts return in order. The recording workflow acknowledges persistence before the next completion is delivered.
4. The GUI retains the session across recordings; changing its configuration or a failed request replaces it. Closing joins the owner thread. A one-shot CLI process loads anew.

See [worker ownership](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/src/native_whisper/cuda.rs#L36-L118), [request processing](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/src/native_whisper/cuda.rs#L227-L291), [session reuse](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/src/workflow.rs#L134-L187) and [cache, batch and persistence behavior](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/native/README.md#L79-L118). Cancellation is checked between operations; loading and an already-running kernel are not interrupted.

At [`595ecca`](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/src/runtime_native.rs#L47), TTS loads the frontend and synthesis engine, then warms an unknown-word phonemizer path and a complete utterance before readiness. Interactive mode keeps that runtime resident. New shapes can still allocate. Its allocator retention setting controls memory kept between requests, not total GPU memory. See [memory-pool ownership](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/kernels/ops.cu#L19).

## Measure speed together with correctness

This chapter was reviewed on 1 October 2026. The cited transcriber main snapshot was committed on 24 August 2026; its native branch and the TTS snapshot on 23 September 2026. Those source dates are separate from measurement dates. TTS's README records a historical LibTorch workload: 57 ms median and 62 ms p95 for “Hello, friend,” with 2,590 ms model loading. That passage does not state a measurement date. It does not compare all current backends. See [the recorded result](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/README.md#L260).

Its CLI correctness gate checks finite output and stable sample counts. Separate native comparison tools impose waveform parity checks. These are different levels of evidence. The native README warns that default CUDA output can differ substantially from the upstream FP32 waveform. Having a strict comparison gate does not establish that the default backend passes it. Keep default-precision and strict-FP32 results separate. See [native validation requirements](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/README.md#L67).

For a new benchmark, record these timings separately:

1. Launch to ready, including model acquisition or loading when applicable.
2. First useful output after readiness.
3. Resident inference after declared warmups.
4. Complete application time, including preprocessing, transfers and output handling.

For an approved native/ONNX comparison, share the same encoding fixtures, weights, shapes and output tolerances. Record provider placement, warmups, memory and transfers. Separate strict FP32 from TF32 or other reduced precision: the transcriber's native default is TF32 computation with FP32 storage, and [ONNX CUDA also exposes a TF32 setting](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html#use_tf32).

Keep model and executable identities, device, precision, raw measurements and correctness outcomes together. A faster result that fails parity is an experiment, not an accepted optimization. Measure the [cancellation boundary](cancellation.md) too. No native-versus-ONNX performance winner was established by this documentation review.
