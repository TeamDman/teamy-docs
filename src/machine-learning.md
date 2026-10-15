# Machine learning and audio

When a model or Python demonstration becomes useful, our target is a Rust program we can inspect, test and control. Start from the model's input/output contract and reuse our speech-tool prior art. Preserve correctness while measuring the parts that consume time or memory.

| Need | Start here | Prior art |
| --- | --- | --- |
| Turn an announcement into reproducible local inference | [Model adoption workflow](model-adoption.md) | Evidence capture, typed contracts, terminal services and parity gates. |
| Score supplied choices or compare Julia, Qwen and Clef | [Finite-choice decisions](finite-choice-models.md) | Trained heads, restricted label logits, input reuse and explicit visual capabilities. |
| Select an inference backend and device | [GPU-enabled inference](gpu-inference.md) | teamy-tts and teamy-transcriber. |
| Generate images or evaluate a pixel-space vision prior | [Evaluate image-generation models](image-generation.md) | Iris-3B generation, depth and restoration; source-reviewed candidate. |
| Describe an image or compare screenshots | [Prompt with local images](image-prompts.md) | Liquid prompt rendering and the Makepad vision encoder and projector. |
| Adapt a Python model implementation | [Port Python ML to Rust](python-ml-to-rust.md) | Weight loading, preprocessing, parity and profiling. |
| Reuse a useful source implementation | [Makepad's inference patterns](makepad-patterns.md) | Makepad study references and our speech-tool implementations. |

Use [the Rust CLI template](rust-cli-template.md) for the command surface and shared behavior. [Find existing sources](source-lookup.md) before downloading more. Track model and code revisions separately, and record which backend and branch a result came from.
