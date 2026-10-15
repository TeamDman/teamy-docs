# Native multimodal embeddings

EmbeddingGemma 2 needs model-specific Rust math and preprocessing, with a pinned official implementation as a development oracle. A native runtime can use LibTorch through `tch` without shipping Python. Keep the SDK request types, model ownership, CLI parsing and local IPC as separate responsibilities, following the [Rust template](rust-cli-template.md) and [resident inference](resident-inference.md) guidance.

The [Google model card](https://ai.google.dev/gemma/docs/embeddinggemma/model_card_2) describes text, image, audio, video and interleaved inputs, an 8192-token shared context, 768-dimensional embeddings and 128/256/512-dimensional MRL prefixes. Renormalize after truncation. Task prefixes apply to text; document title formatting is part of the model input. Use FP32 or BF16, and reject FP16 explicitly.

Pin model artifacts and verify weights, tokenizer and configuration hashes before labeling output with a model revision. Validate architecture and tensor shapes as well. Resolve image patch budgets, actual audio framing and video frame indices before running expensive encoders, then check the expanded token count. Video sampling, stream selection and whether soundtrack audio is included must be explicit contracts.

Reference backend selection can affect correctness. At Transformers revision `e598fbad926d80bc356c0c4d96532030e3dfdb62`, [Gemma4 audio attention](https://github.com/huggingface/transformers/blob/e598fbad926d80bc356c0c4d96532030e3dfdb62/src/transformers/models/gemma4/modeling_gemma4.py) calls `logical_not()` on its mask. A forced eager backend builds an additive float mask, where valid zero entries are then suppressed. Normal SDPA selection builds the expected boolean mask; audio attention still uses its bespoke matmul/softmax equations. Record the exact reference backend per modality. This is a pinned-source finding, not a claim about all later releases.

Qualification should compare preprocessing, token expansion, intermediate tensors and final embeddings, then cover every modality, mixed batching and MRL normalization. Report cold verification/load, warm single latency, batch throughput, dtype and memory separately. A tiny synthetic parity fixture establishes implementation agreement; it does not establish retrieval quality on a dataset. Keep private asset paths and machine records out of this book.

A bounded job queue alone does not bound an IPC daemon. Check frame lengths before allocating payload buffers, bound live connections and handshake time, propagate client cancellation and drain model ownership on shutdown. A native package must launch from a neutral directory without the validation environment or Python on its path.

## Vector memory and search compression references

Revisit when a vector index exhausts RAM or local semantic search needs a smaller retrieval footprint. These are discovery references, not adopted dependencies or scheduled experiments. [KShivendu_'s post](https://x.com/KShivendu_/status/2107558721568321784) returned HTTP 403 on 7 October 2026; its contents remain unverified. The independently reviewed [Qdrant article](https://qdrant.tech/blog/embeddinggemma-2/) (6 October 2026) reports:

| Technique | Relative nDCG@10 retained | Vector RAM reduction |
| --- | --- | --- |
| 768 dimensions, 1-bit TurboQuant, no rescoring | 99% | 30x |
| 256-dimensional MRL prefix, 1-bit TurboQuant, 4x oversampling and rescoring | 94.5% | 77x |

Retention is averaged over five BEIR text datasets: SciFact, NFCorpus, ArguAna, SCIDOCS and FiQA, against exact 768-dimensional FP32 search. Vector RAM excludes graph indexes, payloads and stored originals. These retrieval results establish neither faster model inference nor multimodal retrieval quality; relative nDCG is not identical-neighbor recovery.

[Standard quantization rescoring](https://qdrant.tech/documentation/manage-data/quantization/) uses the same-dimensional originals. Reranking 256-dimensional candidates with full 768-dimensional vectors would be a separate possible experiment, not the reported result. Keeping full 768-dimensional FP32 originals permits rebuilding shorter or differently quantized indexes without re-embedding, at a storage cost.

[Qdrant Edge](https://qdrant.tech/documentation/edge/) is optional embedded Rust retrieval prior art if a separate server becomes burdensome. Its documentation labels it beta; Windows compatibility and local integration are unverified here.
