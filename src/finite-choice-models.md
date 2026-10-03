# Score supplied choices with a local model

Use a decision model when your program already knows the possible answers. Supply choices in order and read their probabilities in the same order. Use stable option IDs when a saved request needs to identify those choices across calls.

Search terms: Jev, Julia, Julia-1, System One, finite-choice classifier, decision model, probability distribution, repeated choices, stdin and native Rust inference.

## Jev explains the hosted decision interface

[Jev is TypeSafe's hosted System One model](https://docs.typesafe.ai/introduction). Its Choice primitive returns a selected option and a distribution over supplied options. Score uses an ordered rubric; Noul returns a yes probability. These are different operations, not interchangeable confidence measures.

The current [Jev input contract](https://docs.typesafe.ai/concepts/state) accepts text or structured text. It does not accept images, audio or video. Preprocessing an image into text creates a separate observation step; it does not make Jev a vision model.

Typed output constrains shape. It does not guarantee factual accuracy. TypeSafe documents [literal readings, numerical errors and adversarial inputs](https://docs.typesafe.ai/model-jaggedness/jev-1.13). Keep arithmetic, object identity and permissions in deterministic code.

Jev supplies interface prior art. Our local implementation target uses open model artifacts and Rust providers; it does not call Jev or send local records to its API.

## Julia supplies an inspectable text-choice model

[Julia-1 at revision `a85b127`](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/README.md) is Supersonic Labs' 144.3-million-parameter decision model. It uses a text encoder and scores 2 to 20 supplied options. It generates neither prose nor images.

Its [encoder and decision head](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/julia/model.py) are the numerical porting reference. Preserve tokenizer identity, marker positions, masks and option order. Compare intermediate tensors and logits with an independent reference before tuning kernels.

The separate [ONNX release at `82a2fad`](https://huggingface.co/SupersonicLabs/Julia-1-ONNX/blob/82a2fadf8fccfccdc5fd4e1009ba8f1a265eb7a8/README.md) provides the graph for our local ONNX adapter. Its Rust tokenizer is not a native Rust numerical implementation. An `ort` adapter runs the ONNX Runtime C/C++ library.

The local service checkout contains a shared Julia encoder, a source-defined Burn numerical provider and a Rust-driven ONNX session. The historical provider versions passed the six-case encoding and score gate described below. Earlier local release CLIs also passed subprocess contract checks, an actual stdin inference check and a matched three-request benchmark. The latest combined native release passed that unchanged six-case gate and three installed subsecond completion measurements. Native control helps inspect operations and deployment; an ONNX graph helps reuse an existing export. Choose a performance default using comparable measurements for the intended workload.

[Magika's Rust session](https://github.com/google/magika/blob/708249d4df4374920c663f80139340648ee71d5e/rust/lib/src/session.rs) is existing ONNX prior art. Its [builder](https://github.com/google/magika/blob/708249d4df4374920c663f80139340648ee71d5e/rust/lib/src/builder.rs) configures threading and optimization for its embedded model. That wrapper is not a generic Julia provider. Use the pinned binding's actual session and execution-provider API when building a new adapter.

For native operations, study [Teamy TTS's CUDA implementation](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/src/cuda.rs) and [Makepad's resident-weight patterns](makepad-patterns.md). Julia needs its own ModernBERT attention, positional encoding, masking and decision head. TTS kernels with different fixed dimensions are ownership and measurement prior art, not compatible model operations.

The pinned [Julia inference policy](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/inference-policy.json) permits 8,192 tokens and retains a 512-token head. The pinned ONNX wrapper defaults to 1,024 and 256 respectively. Our shared encoder and reference fixtures use 1,024 total tokens and a 256-token head, with the default question “Which option should be chosen?” Keep those budgets fixed during comparison. Oversized state, options or question are rejected rather than silently truncated. A longer accepted context is not evidence of equivalent long-context task accuracy.

## Score text choices from the terminal

The text interface is qualified in local service revision `62e92fd`. Its 91 CLI tests and strict Clippy checks pass. Actual native CUDA runs produced identical ordered probabilities through literal text, stdin, structured output and the established JSON request interface. This local addition is not yet a published service release.

Use `teamy-llm decide` with one `--prompt` and a repeated `--choice` for each answer. The typed CLI field is `pub choice: Vec<String>`. Julia-1, the native provider and CUDA are the defaults, independently of the generation model selected for `teamy-llm prompt`. An older executable may not expose the text interface.

With an existing native Julia location recorded, the complete command is:

```powershell
teamy-llm decide --prompt "How to add structured logging to a Rust command-line application." `
  --choice "Writing programs" --choice "Playing games" `
  --choice "Managing photos"
```

The text interface writes one probability per line to stdout, in the supplied choice order, including when redirected or piped. It preserves the numerical result's precision. The qualified native CUDA run above returned:

```text
0.9997739334447404
0.0001614081787158776
0.00006465837654370881
```

The numbers belong to `Writing programs`, `Playing games` and `Managing photos`, respectively. Probabilities are unit-temperature softmax of finite logits. They describe the distribution over these supplied choices, rather than calibrated confidence in a fact.

`--model-dir ./models/julia-native` explicitly overrides the recorded location. That path is a portable example for an already acquired artifact directory. Without the flag, native inference reads the version 1 `julia-native-model.json` record in the runtime's configuration directory; its `model-dir` is an absolute path. `TEAMY_LLM_SERVICE_HOME_DIR` can override the configuration directory. This record is separate from `default-model.json`, which selects the generation model. An absent or invalid record reports an error requesting `--model-dir`; inference does not search directories, download weights or create configuration. ONNX continues to require an explicit artifact directory and runtime library.

Supply 2 to 20 nonempty choice strings. Duplicate labels remain distinct choices at their original positions; the CLI generates distinct ordinal IDs. An `@` character inside prompt or choice text stays literal. Julia's encoder accepts at most 48 tokens per choice, and rejects questions, choices or state that exceed its lossless token budgets. `--question` overrides the default “Which option should be chosen?”

### Read the prompt from stdin

Use `--prompt -` to read raw UTF-8 prompt text from stdin. The choices remain explicit arguments:

```powershell
"How to add structured logging to a Rust command-line application." | teamy-llm decide --prompt - `
  --choice "Writing programs" --choice "Playing games" `
  --choice "Managing photos"
```

This is text input. `--request -` belongs to the separate JSON request interface. Diagnostics remain on stderr in both output modes; invalid input, loading failure, cancellation and output-write errors propagate as failures.

### Request structured results explicitly

Add `--output-format json` to the text shorthand when your caller needs the complete versioned result:

```powershell
teamy-llm decide --prompt "How to add structured logging to a Rust command-line application." `
  --choice "Writing programs" --choice "Playing games" `
  --choice "Managing photos" `
  --output-format json
```

The JSON result preserves choice order in `result.scores`, with `option-id`, `logit` and `probability` per entry. `best-option-id` identifies the largest logit; an exact tie retains the first option. `--include-encoding` also returns raw and padded token IDs, masks and marker positions. A selected ID remains data: executing a file plan or authorizing an action is a separate operation.

## Keep the versioned JSON request interface

Existing `--request` callers retain their JSON input and default JSON output. Do not combine `--request` with `--prompt` or `--choice`. The request version is `1`, option IDs must be unique and nonempty, and Julia accepts 2 to 20 text options. `input.text` is Julia's state; `question` can be supplied in the request or with `--question`. Unknown fields, unsupported versions and visual input are rejected before tokenizer or numerical artifact loading.

For an existing request saved as `choice.json`, quote the `@file` token in PowerShell:

```powershell
teamy-llm decide --provider native --model julia-1 `
  --model-dir ./models/julia-native --device cuda `
  --request '@choice.json' --max-tokens 1024 --head-tokens 256
```

Omitting `--request` without selecting text shorthand reads piped UTF-8 JSON from stdin; `--request -` also selects JSON stdin. The input limit is 2 MiB. A completed inference writes one versioned JSON result to stdout and exits successfully.

CUDA selection for ONNX rejects CPU graph placement by default. The tested export needs some CPU placement, so a strict CUDA session currently fails during preparation. For an explicitly mixed CPU/CUDA experiment, opt in and record placement:

```powershell
teamy-llm decide --provider onnx --model julia-1 `
  --model-dir ./models/julia-onnx `
  --runtime-library ./runtimes/onnxruntime.dll --device cuda `
  --allow-cpu-fallback --profile-prefix ./julia-profile `
  --request '@choice.json' --max-tokens 1024 --head-tokens 256
```

`--allow-cpu-fallback` is valid only for ONNX with `--device cuda`. CUDA provider registration must still succeed. `--profile-prefix` is ONNX-only and writes an ORT node trace; read the actual timestamped filename from the result's `profiling-file` field. Registered providers and permitted fallback are configuration evidence. The recorded node trace establishes what ran where. See [performance analysis](performance-analysis.md).

## Keep Julia's CUDA precision explicit

The new local native path replaces the earlier chunked multiply-and-sum baseline with an explicit whole-matrix FP32 tiled operation. `crates/teamy_llm_burn_julia/src/matmul.rs` selects Cubek's `Strategy::SimpleUnit`, using register tiles and FP32 input and output types. It avoids automatic tensor-core strategy selection, which can select TF32 for FP32 operands, and bypasses matmul autotuning. The source-defined ModernBERT layers, decision head, weights and ordered score contract stay in place. CPU remains the explicit FP32 NdArray reference device.

This is a choice about numerical execution, not evidence of equivalent results or faster startup. Rerun the unchanged six-case encoding, option-order, selected-ID, logit and probability gate after changing the matmul path. Keep the original tolerance constants; do not accept a faster configuration by loosening them.

The local CLI reuses its validated `JuliaEncoder` when loading native weights. Cloned encoders share an `Arc<Tokenizer>` instead of parsing a second tokenizer. For the published BPE bundle, the local `StreamingBpe` deserializer streams modern merge tuples into upstream Tokenizers' `BpeBuilder`, avoiding the generic JSON-value and untagged merge-table intermediates. The upstream BPE builder, encoding algorithms and other tokenizer components remain in use. Unsupported layouts retain the established generic parser. Padding and truncation remain disabled, and the required special-token identities are still checked. Grounding: `crates/teamy_llm_julia_common/src/{lib,streaming_bpe}.rs` and `NativeJuliaProvider::load_with_encoder` in `crates/teamy_llm_burn_julia/src/lib.rs`. The original `load` convenience API still loads and validates its own encoder.

The model keeps the full vocabulary embedding table in a read-only host mapping and uploads only the ordered rows needed by each request. Loading validates the declared FP32 shape, byte range and every value's finiteness, including unused rows. Gathering checks each token index and preserves duplicate tokens, padding and FP32 bits. The retained file handle excludes writes and replacement on Windows for the mapped model's lifetime; other platforms require immutable acquisition files. This reduces device preparation without replacing the vocabulary or changing the input sequence. `MappedEmbeddings` and `JuliaModel` in `crates/teamy_llm_burn_julia/src/model.rs` own the mapping, file handle and row-gather operation.

### Verify the compiler and reuse compiled kernels

Native Julia checks CUDA through library APIs instead of spawning `nvidia-smi`. On Windows, `cuda_runtime.rs` loads the required NVRTC and builtins DLLs from the selected `CUDA_PATH`, checks NVRTC and driver support for CUDA 13.3 or newer, initializes the driver and reads each visible device's compute capability. It retains the libraries for the process. Cudarc's actual compiler library must resolve both `nvrtcVersion` and `nvrtcCompileProgram` to the verified DLL's exact function addresses before CUDA setup proceeds. Mismatched or unavailable runtimes fail explicitly; CUDA selection never silently falls back to CPU. The [driver API version](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__VERSION.html) describes supported CUDA, rather than a driver release number. [NVRTC's version query](https://docs.nvidia.com/cuda/nvrtc/index.html#general-information-query) reports the selected compiler version.

The process uses a compiled-kernel cache under its resolved cache home, separate from model weights. `kernel_cache.rs` hashes the executable bytes, driver API and NVRTC versions, actual device architectures, selected compiler/builtins DLL contents and the bounded toolkit header tree to select its namespace. It holds an exclusive namespace lease for the process. A changed executable or toolkit gets a different namespace. Cache setup failure logs a warning and retains in-memory compilation; an existing foreign persistent CubeCL configuration is rejected.

The local `cubecl-cuda` patch stores native CUBIN images for this provider through the `teamy-cubin-cache` feature. Upstream PTX remains the path when that feature is disabled. Cached entries bind their expected kernel key, format, entry point, shared-memory requirement and image bytes to an XXH3-128 checksum. Bounded image and entry-point validation precedes module loading; incompatible, corrupted or unloadable entries trigger fresh compilation. The checksum detects accidental corruption. The cache remains trusted local executable material and is not authenticated against an attacker. Grounding: `crates/teamy_llm_burn_julia/src/{cuda_runtime,kernel_cache}.rs` and `vendor/cubecl-cuda/src/compute/{context,cache_artifact}.rs` in the matching local service checkout.

For fresh native decisions, `NativeJuliaProvider::load` parses the tokenizer on a scoped CPU worker while the caller prepares the numerical runtime. A second CPU worker validates a backend-independent `PreparedCheckpoint` while the caller verifies CUDA and prepares its compilation cache. The checkpoint owns its immutable file handle, mapping and validated embedding table across the join. CUDA tensors stay on the caller thread. Workers inherit the tracing context and are always joined, including on failure. Capability and policy checks precede startup; GPU model preparation may begin before token-budget and special-token checks finish. Successful encoding remains required before inference. `load-ms` includes this initial parallel preparation but excludes command-line startup, result output and process teardown. Use [Julia's NDJSON phases and external completion measurement](performance-analysis.md#diagnose-julia-with-ndjson-before-timing) to distinguish those boundaries and host submission from completed GPU work.

Resident native decisions pass the current job's cancellation token to `infer_encoded_with_cancellation`. Reusing a provider must not reuse an earlier job's token for its between-layer checks. Executing GPU kernels still cannot be preempted; [cancellation](cancellation.md) describes that boundary.

The host finite-value scan uses runtime-detected AVX2 exponent checks on supported x86 processors, with a scalar fallback and checked unaligned loads and tails. It rejects the same NaN and infinity representations as `f32::is_finite`; it does not convert weights or reduce precision. Chunk boundaries retain cancellation checks. `finite_scan.rs` contains the implementation and independent IEEE edge, vector-lane, alignment and tail fixtures.

The combined local release passed the unchanged six-case parity gate and installed fresh-process completion gate. The [qualified measurement](#local-release-measurement-subsecond-julia-decisions) records the source, executable and cache conditions. The historical CUDA measurements below describe the previous chunked correctness baseline.

## Local release measurement: subsecond Julia decisions

On 3 October 2026, the locally installed release from service source `6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a` completed the three-choice structured-logging example in under one second on an RTX 4090. Its executable SHA-256 is `9ce10b83947650fd56c5ed0144948268b011ca4ccf972f872c41498127020c91`. The source commit remains local; these are qualified local measurements rather than an assertion that the public repository or release binaries contain the change.

```powershell
teamy-llm decide --prompt "How to add structured logging to a Rust command-line application." --choice "Writing programs" --choice "Playing games" --choice "Managing photos" --log-filter warn
```

| Measurement boundary | Recorded milliseconds |
| --- | --- |
| Previous installed release: complete fresh processes | 10,273.6; 9,761.1; 9,961.2 |
| New installed release: complete fresh processes, prepared default CUBIN cache | 953.3; 894.3; 883.6 |
| First installed invocation using this executable's default kernel namespace, with diagnostic logging | 2,930.7 |
| Later decisions inside one resident provider, encoding through completed scores | 49.7; 52.5; 52.1 |

Each timed fresh process loads its own numerical model onto the GPU. Its external clock includes startup, final probability output, successful exit and both output streams reaching EOF. Those three installed runs used the default cache home, followed a separately retained first-use diagnostic and had no resident-model warmup. Operating-system file and driver caches were uncontrolled; this does not establish storage-cold performance. The resident row comes from separate `--repeat 4 --output-format json` evidence and excludes process startup, loading and output. The first resident-series inference took 69.8 ms.

All six independent publisher fixtures passed exact encoding, option order and selected-ID checks with the original absolute-plus-relative logit and probability tolerances. The structured CLI fixture passed the same gate. Validation also passed 21 native, 13 encoder and 91 CLI tests, eight common cache-recovery tests, eleven CUDA image/checksum tests and strict scoped all-target Clippy. No model replacement or dependency-version upgrade was required. These timings qualify this prompt, choice set, model and machine; longer inputs or a cache miss can take longer.

## Keep qualification separate from implementation

The current evidence covers six independent publisher-Python cases with 2, 3, 5, 20, 4 and 2 options, including Unicode, longer state and duplicate descriptions with distinct IDs. Fixtures retain state, question, policy, token arrays, marker positions, intermediate tensors, logits and probabilities. They were generated independently of Rust. See [the isolated reference](python-reference-containers.md).

| Path | Retained qualification evidence |
| --- | --- |
| Publisher Python, FP32 CPU | All six cases completed; repeat runs reproduced arrays, intermediate tensors and scores exactly. |
| Publisher Python, FP32 CUDA with TF32 disabled | All six cases completed and repeated exactly; CPU/CUDA comparison passed the frozen tolerances. |
| Native Rust/Burn, FP32 CPU | All six cases passed exact encoding/order/selection checks and the frozen score tolerances. |
| Native Rust/Burn, FP32 CUDA correctness baseline | All six cases passed the same gate using chunked multiply-and-sum. |
| Native Rust/Burn, combined FP32 tiled release | All six cases passed unchanged encoding, selection and score gates, including the shared encoder, mapped rows, parallel startup and CUBIN cache. |
| Rust ONNX adapter, CPU | All six cases passed exact encoding/order/selection checks and the frozen score tolerances. |
| Rust ONNX adapter, strict CUDA without CPU fallback | Session preparation failed because graph nodes were assigned to CPU. No inference result was produced. |
| Rust ONNX adapter, explicitly mixed CPU/CUDA | All six cases passed the same gate with TF32 disabled and CPU fallback explicitly permitted. A node trace records the observed placement. |

Score acceptance uses `abs(actual - expected) <= absolute_tolerance + relative_tolerance * abs(expected)`: logits use `1e-4` absolute and relative tolerances; probabilities use `1e-5` absolute and `1e-4` relative. IDs, option order and encoding arrays must match exactly. Keep failed configurations in the record. These synthetic parity cases do not establish task accuracy, visual support or a performance winner.

The mixed-session trace recorded 882 CUDA matrix-operation events: 726 `MatMul`, 144 `FusedMatMul` and 12 `Gemm`. None of the checked heavy operations ran on CPU. Its 798 CPU events had only int64 tensor metadata and comprised `Concat`, `Slice`, `Squeeze`, `Reshape` and `Mul`, consistent with shape/index work. This evidence applies to this session and these inputs; provider registration alone would not establish it. See [GPU placement and measurement](gpu-inference.md#make-julias-precision-and-placement-explicit).

The longest tested input was 137 raw tokens, padded to 144. These results do not qualify every sequence up to the configured 1,024-token budget. The historical native CUDA path used the slow FP32 chunked correctness baseline; the [new tiled operation](#keep-julias-cuda-precision-explicit) needs its own qualification. Single qualification runs include startup or JIT effects, and the ONNX mixed qualification run enabled profiling. The separate historical release measurements below disable profiling and reuse a resident provider.

An earlier local release executable, SHA-256 `0bd68ca0fb0fe14a302a55d35dc7c5509e9e1cf0c81f97b8c7ca18d7e73fd899`, passed all 40 model-free subprocess checks. That receipt predates the latest native startup changes. The checks cover root and nested help/version, malformed JSON, unsupported versions and fields, unsupported images, invalid IDs/providers/flags, literal `--` handling and cancellation while stdin remains open. Both `prompt` and `benchmark` rejected `--timeout-ms 0` before missing-model access. Failure cases produced nonzero status, human stderr and empty stdout. The logging regression confirms that explicit `--debug` takes priority over inherited `RUST_LOG=warn`, with diagnostics on stderr and valid nonempty NDJSON in an explicitly selected log file. See [logging](logging.md).

An actual stdin request also scored the three-choice “preserve the unsaved work” fixture using ONNX CPU. The saved JSON result matched the independent reference's complete encoding, option order, selected ID, explicit question, fixed 1,024/256 policy and unchanged numeric tolerances. This qualifies that tested CLI path; it does not replace the separate provider parity gates or establish task accuracy.

<a id="local-unpublished-release-benchmark-one-matched-fixture"></a>

## Historical local release benchmark: one matched fixture

For this short historical fixture, explicitly mixed ONNX CUDA produced the lowest resident request times. The earlier native CUDA multiply-and-sum implementation supplied a correctness baseline; these measurements do not compare the latest FP32 tiled path or establish its performance default.

On 2 October 2026, each backend ran in a fresh Windows x64 release process with `--repeat 3`. The same two-option request had 34 raw tokens, padded to 40, the default question and the fixed 1,024/256 policy. Each captured final result passed an independent comparison of complete encoding, option order, selected ID and the unchanged combined score tolerances. The CLI retains only the final result's scores, so this saved-result check does not independently qualify scores from the two earlier repetitions.

Each process constructed one provider and reused it for all three requests. Times are milliseconds, rounded to one decimal place:

| Provider | Provider load | First inference | Resident request 2 | Resident request 3 |
| --- | ---: | ---: | ---: | ---: |
| Native Burn, FP32 CPU | 1,821.8 | 256.8 | 266.6 | 294.4 |
| Native Burn, FP32 CUDA correctness baseline | 2,020.4 | 10,511.7 | 5,924.1 | 5,614.8 |
| ONNX Runtime, CPU | 1,775.6 | 40.0 | 38.2 | 42.2 |
| ONNX Runtime, explicitly mixed CPU/CUDA | 2,361.1 | 269.1 | 16.1 | 16.2 |

The measured executable was built from unpublished local additions to service revision `cc09503`; its SHA-256 is `57b31a64c6afdf54fcdcce65e68377ffa03ac0b62285e518e1d8ba246fff9e37`. Native inference used Burn 0.21.0 and CubeCL 0.10.0 with the pinned Julia source/weights revision `a85b127`. ONNX used binding `2.0.0-rc.12`, Runtime 1.24.2 with CUDA 13 support, and export revision `82a2fad`. GPU execution used an RTX 4090 with driver 610.88. TF32 remained disabled; the native CUDA path used explicit FP32 chunked multiplication and summation. Model and runtime identities were retained in the local receipts.

`load-ms` measures provider construction after an initial shared encoder check; it excludes process startup and that first check. `inference-ms` includes request encoding, deterministic result validation and completed score readback. It excludes final JSON serialization. First inference can include JIT compilation and first-use allocations. Two subsequent samples are a small resident measurement, not a steady-state distribution or p95 estimate. No tracing profiler, Cargo build or other inference run overlapped these captures. Debug stderr diagnostics and an external memory monitor were enabled.

The monitor requested 250 ms sampling of process working set and private bytes. Those sample maxima are not true peaks. GPU readings cover the whole device, including other allocations, and cannot establish per-process GPU memory. Monitoring itself adds overhead. These results support a default for this measured fixture and configuration; they do not rank all shapes, devices, precision modes or generation models. See [GPU execution boundaries](gpu-inference.md#make-julias-precision-and-placement-explicit) and [performance analysis](performance-analysis.md).

## Start with a compiling contract

The local service-core example `finite_choice_contract.rs` is the first bounded implementation target. It exercises Facet types, explicit provider capabilities, ordered scores, stable softmax and rejection of unsupported images. Its fixed synthetic provider does not run Julia or Qwen.

The synthetic contract also checks that unsupported image bytes are rejected. That fixture does not add image support to Julia.


<details>
<summary>Complete synthetic contract example</summary>

<!-- checked-choice-example:start -->
```rust
//! Synthetic contract demonstration: no Julia model, weights, image decoder, GPU or network.

use teamy_llm_core::ModelSelector;
use teamy_llm_core::finite_choice::{
    BackendCapabilities, ChoiceContent, FiniteChoiceCapabilities, FiniteChoiceError,
    FiniteChoiceErrorKind, FiniteChoiceOption, FiniteChoiceProvider, FiniteChoiceRequest,
    FiniteChoiceResult, ValidatedFiniteChoiceRequest, VisualFormat, VisualInput, finite_choice,
};

/// Deliberately fixed scores; this tests the interface, not model quality or inference correctness.
#[derive(Default)]
struct SyntheticProvider {
    calls: usize,
}

impl FiniteChoiceProvider for SyntheticProvider {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            text_generation: false,
            finite_choice: Some(FiniteChoiceCapabilities::julia_1_text_only()),
        }
    }

    fn infer(
        &mut self,
        request: &ValidatedFiniteChoiceRequest<'_>,
    ) -> Result<FiniteChoiceResult, FiniteChoiceError> {
        self.calls += 1;
        // Fixture values are in the exact order of the two options below.
        FiniteChoiceResult::from_logits(request, vec![1_000.0, 1_001.0])
    }
}

fn main() -> Result<(), FiniteChoiceError> {
    let mut provider = SyntheticProvider::default();
    let request = FiniteChoiceRequest {
        selector: ModelSelector::managed("synthetic-fixture"),
        input: ChoiceContent::text("Choose a category for this fixture."),
        options: vec![
            FiniteChoiceOption {
                id: "writing".into(),
                content: ChoiceContent::text("Writing programs"),
            },
            FiniteChoiceOption {
                id: "playing".into(),
                content: ChoiceContent::text("Playing games"),
            },
        ],
    };
    let result = finite_choice(&mut provider, &request)?;
    assert_eq!(provider.calls, 1);
    assert_eq!(result.scores[0].option_id, "writing");
    assert_eq!(result.scores[1].option_id, "playing");
    assert_eq!(result.best_option().unwrap().option_id, "playing");
    assert!(
        result
            .scores
            .iter()
            .all(|score| score.probability.is_finite())
    );

    let mut visual_request = request.clone();
    visual_request.input.visuals.push(VisualInput {
        format: VisualFormat::Png,
        // Placeholder bytes are deliberately not an image; decoding is outside this contract.
        bytes: vec![1],
    });
    let error = finite_choice(&mut provider, &visual_request).unwrap_err();
    assert_eq!(error.kind, FiniteChoiceErrorKind::UnsupportedVisual);
    assert_eq!(
        provider.calls, 1,
        "unsupported vision never reaches the provider"
    );
    println!(
        "Synthetic contract passed: ordered scores, stable softmax, unsupported vision rejected."
    );
    Ok(())
}
```
<!-- checked-choice-example:end -->

</details>

The acceptance command is run from the matching service workspace:

```powershell
cargo run --offline --locked -p teamy_llm_core --example finite_choice_contract
cargo test --offline --locked -p teamy_llm_core
```

These commands describe the local addition. The published [service revision `cc09503`](https://github.com/TeamDman/teamy-llm-service/tree/cc0950321a13cf6a8621c574d75cca664b150880) predates it. Do not expect that revision or an older installed CLI to expose the new contract.

From this documentation repository, run `pwsh -NoProfile -File scripts/check-inference-example.ps1 -ServiceRoot ../teamy-llm-service` against the matching local checkout. The harness compares the complete snippet with the actual example, then runs its core tests and executable assertions offline. It writes a local receipt under the ignored `.validation` directory. This check is separate from Pages CI while the additive service API remains local.

Passing the example proves deterministic contract behavior. Real model loading, tokenizer parity, image interpretation, cancellation and task accuracy require separate tests. A softmax distribution is normalized mathematical output; calibration must be evaluated against labelled cases.

Freeze evaluation criteria and held-out inputs before comparing providers. Retain rejected capabilities and failed cases. State project preferences beside results rather than altering the task until a preferred candidate wins. Follow [model implementation steps](model-adoption.md) and [review records](plan-records.md).
