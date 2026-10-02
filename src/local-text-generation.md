# Generate text with a local model

Use `teamy-llm prompt` when you want generated text rather than [scores for supplied choices](finite-choice-models.md). Choose the model explicitly, bound the output and keep its weights, tokenizer and chat template together. A model name alone does not identify a working numerical runtime.

This chapter describes unpublished local additions to `teamy-llm-service`, reviewed and exercised on 2 October 2026. The [public service revision `cc09503`](https://github.com/TeamDman/teamy-llm-service/tree/cc0950321a13cf6a8621c574d75cca664b150880) predates them. Use the matching locally built executable; an older installed CLI may not expose this model or these safeguards.

## Select the model before prompting

Inspect the configured models and select the named OrcaRouter Q4_K_M model:

```powershell
teamy-llm model list
teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
  --no-thinking --max-new-tokens 32 --timeout-ms 600000 `
  -- "Why is the sky blue?"
```

The example requires already prepared model artifacts. Prompting does not download weights or search a Downloads folder. `--model` selects its managed model directory; `--model-dir ./models/orca` selects an explicit prepared directory instead. These flags cannot be combined. Model preparation is a separate acquisition operation.

Omitting both flags retains the configured default, falling back to the existing `qwopus-3.5-9b-coder-q4-k-m` model when no default is registered. `prompt` warns about implicit selection and lists available models. Adding OrcaRouter did not silently replace that default.

`prompt` streams generated text to stdout. Diagnostics use stderr; `--debug --log-file ./generation.ndjson` adds an explicitly selected NDJSON diagnostic file. Errors can leave partial generated text if streaming has already begun. Use [logging](logging.md) and [cancellation](cancellation.md) when integrating the command into another application.

## Preserve the artifact and prompt contract

The qualified weight file is [OrcaRouter's Qwen3.8-27B-Uncensored Q4_K_M GGUF at `fc437a3`](https://huggingface.co/orcarouter/Qwen3.8-27B-Uncensored-GGUF/blob/fc437a3374c9977bdc339a8ec106dcd9a0357001/Qwen3.8-27B-Uncensored-Q4_K_M.gguf). Its 16,810,714,496 bytes have SHA-256 `3445102e9cde5d562508642c100a2f5ac3368a5a3f748442811d7a95daee3bec`. The adapter verifies that identity on initial runtime loading, before parsing the model and preparing its GPU session. Q4_K_M includes mixed tensor types; it does not mean every tensor is four-bit.

Tokenization and single-turn rendering use the separate [publisher source at `8cb32d7`](https://huggingface.co/orcarouter/Qwen3.8-27B-Uncensored/blob/8cb32d72080f6a47bf34dc5adf8067daa6c63a31/tokenizer_config.json). Independent publisher-template fixtures cover Direct and Thinking modes, with and without a system message. The Rust renderer and Hugging Face tokenizer matched those fixture strings and token IDs exactly.

For each service request, the adapter compares GGUF-vocabulary token IDs with independently encoded Hugging Face IDs before GPU prefill. Any token or ordering mismatch rejects the request. Session preparation can precede this comparison; it is not a promise that no GPU allocation has occurred. The numerical runtime uses [Makepad revision `9e5e3d2`](https://github.com/makepad/makepad/tree/9e5e3d2b03214f8c2c37adffa0e308c815662060), with a resident quantized CUDA session. See [Makepad's execution patterns](makepad-patterns.md).

This local slice is text-only and single-turn. It does not run a vision projector, accept tool calls or preserve conversation history through the prompt command. Its context limit is 1,024 tokens, counting the rendered prompt plus requested output. Oversized requests fail rather than truncate silently.

`prompt` defaults to Direct mode and 256 new tokens. Explicit named OrcaRouter selection with `--thinking` defaults to 512; the legacy Thinking default remains 1,024. An explicit model directory does not receive that named-model default, so set `--max-new-tokens` yourself. Reduce the output budget for longer prompts.

A real CUDA Thinking run asked “What does café mean in English?” It completed successfully with 59 prompt tokens and the requested 24 generated tokens. Its output was bounded reasoning without a final answer when that token limit was reached. This establishes that tested Unicode/Thinking execution path, not completed task accuracy or a useful default output budget.

`--timeout-ms` bounds generation, excluding queue wait and model loading. `--stop-after-duration` requests cooperative cancellation from CLI startup. Cancellation checks occur between operations; they do not preempt an active load operation or GPU kernel. Generated text is an observation, not permission to execute a command.

The final local release passed all 40 model-free subprocess checks. Both `prompt --timeout-ms 0` and `benchmark --timeout-ms 0` failed with nonzero status and human diagnostics before missing-model access. Real generation runs and model-free argument checks remain separate evidence.

## Reuse the runtime within one process

A new `prompt` process starts cold. Runtime reuse applies to repeated calls through one service instance or the benchmark's `--repeat` loop. These commands do not automatically connect to a running daemon or route requests to a remote service.

To measure that distinction for the same 32-token Direct workload:

```powershell
teamy-llm benchmark --model qwen3.8-27b-uncensored-q4-k-m `
  --no-thinking --max-new-tokens 32 --repeat 3 --timeout-ms 600000 `
  --debug --log-file ./generation-phases.ndjson `
  -- "Why is the sky blue?"
```

Select `--no-thinking` explicitly: benchmark defaults to testing both modes, unlike prompt. Its output is readable benchmark lines. Time to first token measures the first token event from request submission, including loading when needed. UTF-8 event buffering and callbacks affect that boundary; it is not pure decoder or kernel time.

## Local release measurement: tokenizer reuse

Two fresh Windows x64 release processes ran the same prompt, 18 prompt tokens and three 32-token completions on an RTX 4090 with driver 610.88. No Cargo build or other inference run overlapped either capture. Debug stderr, NDJSON phase logs and external memory sampling were enabled.

| Time to first token event, milliseconds | Before tokenizer caching | With tokenizer caching |
| --- | ---: | ---: |
| First request in a cold process | 22,814.061 | 22,800.184 |
| Resident request 2 | 842.844 | 228.415 |
| Resident request 3 | 821.847 | 186.938 |

The cache reduced repeated tokenizer setup; cold startup remained about 22.8 seconds. The optimized capture's phase spans recorded about 10.1 seconds for weight hashing, 49 milliseconds for model parsing and 11.4 seconds for session preparation. Its first `llm_tokenize` span took about 402 milliseconds; later spans took 405 and 423 microseconds. These logged CPU-side phase durations are not GPU kernel timings or an additive account of every startup cost.

The optimized executable's SHA-256 is `0bd68ca0fb0fe14a302a55d35dc7c5509e9e1cf0c81f97b8c7ca18d7e73fd899`; the earlier executable was `57b31a64c6afdf54fcdcce65e68377ffa03ac0b62285e518e1d8ba246fff9e37`. The tokenizer cache retains encoding settings and reloads when file length or modification time changes. Those freshness hints are not immutable content checks; replacing a file while preserving both requires explicit invalidation.

These are two resident samples per executable, not a latency distribution. Process memory was sampled at a requested 250 ms interval; sample maxima are not true peaks. GPU readings cover the whole device and do not establish per-process allocation. The model completed real CUDA generation and passed independent prompt/token parity. Independent Python-to-Makepad numerical or logit parity was not established, nor was an output-quality ranking against the older 9B model. Follow [performance analysis](performance-analysis.md) before attributing another delay to the same cause.
