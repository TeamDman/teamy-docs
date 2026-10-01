# Run GPU-enabled inference

Choose the backend that makes the complete workflow responsive while preserving its correctness contract. Start with our speech-tool prior art, the model's existing artifacts and the device you need to support. Loading, preprocessing, transfers, decoding and playback can each dominate the experience.

For a new Teamy tool, use Rust and [the CLI template](rust-cli-template.md) for its command surface. Use [Python-to-Rust porting](python-ml-to-rust.md) to preserve the model's behavior and [logging](logging.md) to expose device, preparation and inference outcomes.

## Match the backend to the problem

| Starting point | Our implementation reference | What it buys and requires |
| --- | --- | --- |
| An existing TorchScript graph | teamy-transcriber's main implementation. | A Rust runtime using tch/LibTorch can run the graph without launching Python. It still deploys LibTorch and compatible model artifacts. |
| Numerical weights with operations expressed in Rust | The main transcriber's safetensors runner. | Control over validation, tensor operations and decoding, while using LibTorch to execute them. |
| A measured need for custom GPU operations and memory control | teamy-tts's default CUDA backend and the transcriber's native branch. | Source-defined operations and CUDA kernels, with explicit precision, workspace, platform and parity responsibilities. |

Custom CUDA is our speech-tool prior art, not a requirement for every new model. Begin with a compatible reference path. Optimize the part that measurements identify.

## Reuse resident runtimes and meaningful readiness

At public revision [`595ecca`](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/Cargo.toml#L53), `teamy-tts` defaults to its source-defined CUDA backend. Rust defines the GLaDOS model operations; kernels, cuBLAS and cuDNN execute numerical work. Weights remain external safetensors files. Its optional tch build uses a different deployment path and retains a CPU option.

The native constructor loads the frontend and synthesis engine, then warms an unknown-word phonemizer path and a complete utterance before reporting readiness. Interactive mode keeps the runtime resident. Novel input shapes can still incur allocation costs. The allocator's retention setting controls memory kept between requests; it is not a total GPU memory limit. See [runtime construction](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/src/runtime_native.rs#L47) and [CUDA session and memory pool](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/kernels/ops.cu#L19).

Newer native transcriber work is on public branch `codex/native-whisper`, revision [`7d1ec22`](https://github.com/TeamDman/teamy-transcriber/blob/7d1ec222639d67056e18f233cd017a359008306b/native/README.md#L3). Main at `d87d502` still uses tch. The branch defines Whisper in Rust and custom CUDA, retains weights and decoder workspaces, and distinguishes TF32 computation from FP32 storage. Its documented support evidence is Windows x64 on an RTX 4090. Other devices and platforms need validation.

## Measure speed together with correctness

TTS records a historical LibTorch measurement of 57 ms median and 62 ms p95 for “Hello, friend,” with a separate 2,590 ms model load. That is one identified workload, not proof that every newer backend is faster. See [the recorded result](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/README.md#L260).

Its CLI correctness gate checks finite output and stable sample counts. Separate native comparison tools impose waveform parity checks. These are different levels of evidence. The native README warns that default CUDA output can differ substantially from the upstream FP32 waveform. Having a strict comparison gate does not establish that the default backend passes it. Keep default-precision and strict-FP32 results separate. See [native validation requirements](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/README.md#L67).

For a new benchmark, record these timings separately:

1. Launch to ready, including model acquisition or loading when applicable.
2. First useful output after readiness.
3. Resident inference after declared warmups.
4. Complete application time, including preprocessing, transfers and output handling.

Keep model and executable identities, device, precision, raw measurements and correctness outcomes together. A faster result that fails parity is an experiment, not an accepted optimization. Also measure the [cancellation boundary](cancellation.md); a stop request does not establish immediate interruption of GPU work.

Study [Makepad's cache, fallback and precision decisions](makepad-patterns.md) and the [curated CUDA references](curating-prior-art.md#use-stars-as-discovery-input) when adapting this design.
