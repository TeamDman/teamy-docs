# Reuse Makepad's inference patterns

Makepad has informed the direction of our Rust inference work. Treat it as prior art for gaining control of execution, then record the concrete pattern we adopt and how we validate it. Inspiration, a dependency and copied code have different provenance.

The inspected Teamy speech-tool manifests do not declare Makepad as a dependency. The transcriber's [public implementation ledger](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/PLAN.md#L42) identifies burnt-apple's Burn frontend and model loader as the original native Whisper reference. Preserve that recorded lineage alongside Makepad's role as inspiration and a study reference.

## Study the decisions in code

Makepad's public revision `8e82a8e` gives us concrete implementation references:

| Problem | Makepad's approach | What to inspect |
| --- | --- | --- |
| Select acceleration that is actually available | Attempt Metal or CUDA operations and otherwise use the CPU implementation. Distinguish build-time kernels from runtime devices. | [Acceleration dispatch](https://github.com/makepad/makepad/blob/8e82a8e695af39c582fde30e37e888f375b31d1a/libs/ai/models/speech/src/whisper/accel.rs#L1). |
| Avoid transfer and dispatch overhead | Keep dequantized weights resident and cache cross-attention data; small operations can still favor the CPU. | [CUDA backend rationale](https://github.com/makepad/makepad/blob/8e82a8e695af39c582fde30e37e888f375b31d1a/libs/ai/models/speech/src/whisper/cuda/backend.rs#L1). |
| Prevent stale cache reuse | Use a content fingerprint when a host allocation can be recycled. | Weight-cache identity in the same CUDA backend. |
| Make precision a deliberate choice | Use dequantized FP32 weights and FP32 matrix operations to stay close to a CPU reference. | Precision commentary in the same backend. |

These choices explain why “move everything to the GPU” is insufficient. The speed of an operation includes its transfers, dispatch and cache behavior. A cache also needs a reliable identity, not just a remembered pointer.

The newer native transcriber branch defaults to TF32 computation with FP32 storage. That differs from Makepad's cited precision choice. Neither establishes a universal winner; compare correctness and latency for the actual model and device. See [our GPU inference references](gpu-inference.md).

## Bring the pattern into our tools

Keep a reference path, expose backend capability, state precision and measure the complete operation. Reuse the decision that solves our problem, with an explicit link to our implementation and its evidence.

Our speech tools demonstrate resident runtimes, inspectable model operations and stricter validation of optimized paths. [Python-to-Rust porting](python-ml-to-rust.md) connects those steps to preprocessing and output parity. [GPU inference](gpu-inference.md#measure-speed-together-with-correctness) distinguishes cold loading, first output, warm inference and complete application time.

Makepad's comments explain its rationale; they are not benchmark receipts for our tools. Claims that an adopted pattern accelerated TTS or transcription should link the measured workload and correctness result. Record an unmeasured benefit as a design reason until those measurements exist.
