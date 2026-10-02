# Score supplied choices with a local model

Use a decision model when your program already knows the possible answers. Supply stable option IDs and return their scores in the same order. Keep text generation behind a separate contract.

Search terms: Jev, Julia, Julia-1, System One, finite-choice classifier, decision model, probability distribution, visual classifier, Qwen and native Rust inference.

## Jev explains the hosted decision interface

[Jev is TypeSafe's hosted System One model](https://docs.typesafe.ai/introduction). Its Choice primitive returns a selected option and a distribution over supplied options. Score uses an ordered rubric; Noul returns a yes probability. These are different operations, not interchangeable confidence measures.

The current [Jev input contract](https://docs.typesafe.ai/concepts/state) accepts text or structured text. It does not accept images, audio or video. Preprocessing an image into text creates a separate observation step; it does not make Jev a vision model.

Typed output constrains shape. It does not guarantee factual accuracy. TypeSafe documents [literal readings, numerical errors and adversarial inputs](https://docs.typesafe.ai/model-jaggedness/jev-1.13). Keep arithmetic, object identity and permissions in deterministic code.

Jev supplies interface prior art. Our local implementation target uses open model artifacts and Rust providers; it does not call Jev or send local records to its API.

## Julia supplies an inspectable text-choice model

[Julia-1 at revision `a85b127`](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/README.md) is Supersonic Labs' 144.3-million-parameter decision model. It uses a text encoder and scores 2 to 20 supplied options. It generates neither prose nor images.

Its [encoder and decision head](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/julia/model.py) are the numerical porting reference. Preserve tokenizer identity, marker positions, masks and option order. Compare intermediate tensors and logits with an independent reference before tuning kernels.

The separate [ONNX release at `82a2fad`](https://huggingface.co/SupersonicLabs/Julia-1-ONNX/blob/82a2fadf8fccfccdc5fd4e1009ba8f1a265eb7a8/README.md) provides another provider candidate. Its Rust tokenizer is not a native Rust numerical implementation. An `ort` adapter runs the ONNX Runtime C/C++ library.

Our two provider targets are a source-defined Rust numerical implementation and a Rust-driven ONNX session. Both must pass the same encoding and output fixtures. Native control helps inspect operations and deployment; an ONNX graph helps reuse an existing export. Neither approach wins on speed without measurements.

[Magika's Rust session](https://github.com/google/magika/blob/708249d4df4374920c663f80139340648ee71d5e/rust/lib/src/session.rs) is existing ONNX prior art. Its [builder](https://github.com/google/magika/blob/708249d4df4374920c663f80139340648ee71d5e/rust/lib/src/builder.rs) configures threading and optimization for its embedded model. That wrapper is not a generic Julia provider. Use the pinned binding's actual session and execution-provider API when building a new adapter.

For native operations, study [Teamy TTS's CUDA implementation](https://github.com/TeamDman/teamy-tts/blob/595ecca2c6429dc69d2552a851c467a9a19dcce3/native/src/cuda.rs) and [Makepad's resident-weight patterns](makepad-patterns.md). Julia needs its own ModernBERT attention, positional encoding, masking and decision head. TTS kernels with different fixed dimensions are ownership and measurement prior art, not compatible model operations.

The pinned [Julia inference policy](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/inference-policy.json) permits 8,192 tokens and retains a 512-token head. The pinned ONNX wrapper defaults to 1,024 and 256 respectively. Align these settings explicitly; a longer accepted context is not evidence of equivalent long-context task accuracy.

## Keep visual input explicit

The target service should support text generation with visual input and choice scoring with visual input. Each candidate may contain its own text and image. Julia's current text-only encoder cannot satisfy that complete target.

Represent visual artifacts as explicit references. Record their content identity, media type and preprocessing. A provider must state whether it accepts images in the observation, candidates or both. Reject unsupported requests before inference. Never remove images to make a request fit a text-only provider.

For a Qwen generation provider, verify the actual vision encoder, projector, image preprocessing, tokenizer and runtime. A model card's vision claim and a separate projector file do not prove that the selected loader uses them. See [GPU inference](gpu-inference.md) and [Python-to-Rust parity](python-ml-to-rust.md).

## Start with a compiling contract

The local service-core example `finite_choice_contract.rs` is the first bounded implementation target. It exercises Facet types, explicit provider capabilities, ordered scores, stable softmax and rejection of unsupported images. Its fixed synthetic provider does not run Julia or Qwen.

The current contract carries encoded visual bytes in memory. It does not read paths, fetch URLs or decode images. Content references and verified preprocessing belong to the next adapter layer.


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
