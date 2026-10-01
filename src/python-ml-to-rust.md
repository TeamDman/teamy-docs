# Port Python ML to Rust

When a useful Python ML demonstration appears, our target is a Rust program whose behavior and execution we can inspect. Preserve the complete input/output contract before optimizing it. The port includes preprocessing, token conventions, numerical operations, weights, decoding and lifecycle management.

Use [the Rust CLI template](rust-cli-template.md) for the new tool. Connect it to [typed output](output-shape.md), [logging](logging.md), [cancellation](cancellation.md) and [GPU backend selection](gpu-inference.md). Start by [finding existing source](source-lookup.md), then pin model and code revisions separately.

## Choose how much of the runtime to replace

Our speech tools demonstrate three useful stages:

| Approach | What changes | Prior art |
| --- | --- | --- |
| Rust application around TorchScript through tch | Remove Python from product execution while retaining the exported graph and LibTorch. | The main transcriber's TorchScript runner. |
| Operations expressed in Rust over LibTorch tensors | Make graph construction, validation and decoding inspectable in Rust. | Its direct safetensors runner. |
| Rust operations and custom CUDA kernels | Control numerical operations, memory and deployment more directly. | Native TTS and the published native transcriber branch. |

Each stage removes different dependencies and creates different maintenance work. A handwritten kernel is useful when it solves a measured problem; it is not the first step required for every port.

The main transcriber's [direct runner at `d87d502`](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/native_whisper/tch_safetensors.rs#L1) validates dimensions and tensor types, retains weights in LibTorch tensors, and expresses encoder/decoder operations in Rust. The decoder caches attention state and uses greedy token selection with suppression rules. A safetensors filename alone does not prove compatibility: tokenizer, weight names, shapes and decoding conventions must agree.

## Establish parity before optimizing

Use a sequence that makes divergence easy to locate:

1. Freeze a reference workload, model revision and expected outputs.
2. Verify preprocessing and token IDs independently.
3. Compare intermediate tensors at meaningful boundaries.
4. Compare complete output and document numerical tolerances.
5. Optimize measured costs while keeping those checks.

Whisper's input begins before its first learned layer. The main transcriber's frontend implements padding, Fourier transforms, mel filters and log normalization. A sine-wave regression compares Python reference values. See [the frontend test](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/native_whisper/frontend.rs#L232). The newer native branch [supports 80 and 128 mel bins and retains an FFT plan](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/native/src/frontend.rs#L1).

For TTS, dictionary lookup, phoneme inventory, neural fallback, speaker embeddings, acoustic model and vocoder form one pipeline. Python can remain an artifact-export and independent-reference tool while the product runtime uses Rust. See [TTS artifact preparation](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/README.md#L25).

Preserve failed comparisons instead of accepting plausible output. TTS excludes experimental persistent recurrent algorithms from its normal build after they failed the strict waveform gate. This is a specific optimization that the gate blocked; it does not prove waveform parity for the default backend. Keep that experiment separate from [default CUDA and strict-FP32 comparison results](gpu-inference.md#measure-speed-together-with-correctness). See [the experimental and validation policy](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/README.md#L55).

## Separate artifact acquisition from inference

TTS [verifies downloaded archives before installation](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/src/model_sources.rs#L210). The newer transcriber branch [selects existing model packages without downloading during preparation](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/native/README.md#L41). Both make the artifact contract explicit.

For Hub-hosted models, record the model revision, expected files, license and checksums alongside the code revision. Inspect an existing cache before fetching another copy. The current Hugging Face CLI is `hf`; check its installed help for revision, download-preview and cache-verification support. See [official CLI documentation](https://huggingface.co/docs/huggingface_hub/guides/cli).

Use [Makepad's inference patterns](makepad-patterns.md) and [curated conversion references](curating-prior-art.md#use-stars-as-discovery-input) as specific study material. A model announcement or successful export starts the evaluation; parity and measured application behavior determine adoption.
