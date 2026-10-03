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

The local service checkout now contains a shared Julia encoder, a source-defined Burn numerical provider and a Rust-driven ONNX session. Both providers have passed the six-case encoding and score gate described below. A locally built release CLI has also passed subprocess contract checks, an actual stdin inference check and a matched three-request benchmark. These additions are not yet a published release. Native control helps inspect operations and deployment; an ONNX graph helps reuse an existing export. Choose a performance default using comparable measurements for the intended workload.

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

## Keep qualification separate from implementation

The current evidence covers six independent publisher-Python cases with 2, 3, 5, 20, 4 and 2 options, including Unicode, longer state and duplicate descriptions with distinct IDs. Fixtures retain state, question, policy, token arrays, marker positions, intermediate tensors, logits and probabilities. They were generated independently of Rust. See [the isolated reference](python-reference-containers.md).

| Path | Current evidence |
| --- | --- |
| Publisher Python, FP32 CPU | All six cases completed; repeat runs reproduced arrays, intermediate tensors and scores exactly. |
| Publisher Python, FP32 CUDA with TF32 disabled | All six cases completed and repeated exactly; CPU/CUDA comparison passed the frozen tolerances. |
| Native Rust/Burn, FP32 CPU | All six cases passed exact encoding/order/selection checks and the frozen score tolerances. |
| Native Rust/Burn, FP32 CUDA | All six cases passed the same gate using the chunked multiply-and-sum correctness baseline. |
| Rust ONNX adapter, CPU | All six cases passed exact encoding/order/selection checks and the frozen score tolerances. |
| Rust ONNX adapter, strict CUDA without CPU fallback | Session preparation failed because graph nodes were assigned to CPU. No inference result was produced. |
| Rust ONNX adapter, explicitly mixed CPU/CUDA | All six cases passed the same gate with TF32 disabled and CPU fallback explicitly permitted. A node trace records the observed placement. |

Score acceptance uses `abs(actual - expected) <= absolute_tolerance + relative_tolerance * abs(expected)`: logits use `1e-4` absolute and relative tolerances; probabilities use `1e-5` absolute and `1e-4` relative. IDs, option order and encoding arrays must match exactly. Keep failed configurations in the record. These synthetic parity cases do not establish task accuracy, visual support or a performance winner.

The mixed-session trace recorded 882 CUDA matrix-operation events: 726 `MatMul`, 144 `FusedMatMul` and 12 `Gemm`. None of the checked heavy operations ran on CPU. Its 798 CPU events had only int64 tensor metadata and comprised `Concat`, `Slice`, `Squeeze`, `Reshape` and `Mul`, consistent with shape/index work. This evidence applies to this session and these inputs; provider registration alone would not establish it. See [GPU placement and measurement](gpu-inference.md#make-julias-precision-and-placement-explicit).

The longest tested input was 137 raw tokens, padded to 144. These results do not qualify every sequence up to the configured 1,024-token budget. The native CUDA path is a slow FP32 correctness baseline awaiting optimization. Single qualification runs include startup or JIT effects, and the ONNX mixed qualification run enabled profiling. The separate release measurements below disable profiling and reuse a resident provider.

The final local release executable, SHA-256 `0bd68ca0fb0fe14a302a55d35dc7c5509e9e1cf0c81f97b8c7ca18d7e73fd899`, passed all 40 model-free subprocess checks. They cover root and nested help/version, malformed JSON, unsupported versions and fields, unsupported images, invalid IDs/providers/flags, literal `--` handling and cancellation while stdin remains open. Both `prompt` and `benchmark` rejected `--timeout-ms 0` before missing-model access. Failure cases produced nonzero status, human stderr and empty stdout. The logging regression confirms that explicit `--debug` takes priority over inherited `RUST_LOG=warn`, with diagnostics on stderr and valid nonempty NDJSON in an explicitly selected log file. See [logging](logging.md).

An actual stdin request also scored the three-choice “preserve the unsaved work” fixture using ONNX CPU. The saved JSON result matched the independent reference's complete encoding, option order, selected ID, explicit question, fixed 1,024/256 policy and unchanged numeric tolerances. This qualifies that tested CLI path; it does not replace the separate provider parity gates or establish task accuracy.

## Local unpublished release benchmark: one matched fixture

For this short fixture, explicitly mixed ONNX CUDA produced the lowest resident request times. The current native CUDA multiply-and-sum implementation remains useful as a correctness baseline, but its measured speed does not support making it the performance default.

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
