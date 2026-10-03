# Generate text with a local model

Use `teamy-llm prompt` when you want generated text rather than [scores for supplied choices](finite-choice-models.md). Choose the model explicitly, bound the output and keep its weights, tokenizer and chat template together. A model name alone does not identify a working numerical runtime.

This chapter describes unpublished local additions to `teamy-llm-service`. The [public service revision `cc09503`](https://github.com/TeamDman/teamy-llm-service/tree/cc0950321a13cf6a8621c574d75cca664b150880) predates them. Use the matching locally built executable and check its help. The dated measurements below identify earlier builds separately; they do not measure every subsequent change.

The latest [paired fresh-startup measurement](#local-release-measurement-five-second-fresh-startup) reached visible text in 4.62–4.76 seconds on all three final-build runs, preserving the full generated text and token IDs for the tested workload.

## Select the model before prompting

Inspect the configured models and select the named OrcaRouter Q4_K_M model:

```powershell
teamy-llm model list
teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
  --context-tokens 8192 --no-thinking --max-new-tokens 32 --timeout-ms 600000 `
  -- "Why is the sky blue?"
```

The example requires already prepared model artifacts. Prompting does not download weights or search a Downloads folder. `--model` selects its managed model directory; `--model-dir ./models/orca` selects an explicit prepared directory instead. These flags cannot be combined. Model preparation is a separate acquisition operation.

Omitting both flags uses the explicit configured preference when present. Otherwise selection uses the first existing registered model, then the built-in `qwopus-3.5-9b-coder-q4-k-m` when no model is registered. `prompt` warns about implicit selection and points to `model list` and `model default show`. Adding OrcaRouter did not silently replace an explicitly selected default. [Persistent model preferences](interactive-inference.md#select-a-model-and-persist-a-default) provide separate `model default set` and `model default show` commands.

`prompt` streams generated text to stdout. Diagnostics use stderr; `--debug --log-file ./generation.ndjson` adds an explicitly selected NDJSON diagnostic file. Errors can leave partial generated text if streaming has already begun. Use [logging](logging.md) and [cancellation](cancellation.md) when integrating the command into another application.

## Preserve the artifact and prompt contract

The qualified weight file is [OrcaRouter's Qwen3.8-27B-Uncensored Q4_K_M GGUF at `fc437a3`](https://huggingface.co/orcarouter/Qwen3.8-27B-Uncensored-GGUF/blob/fc437a3374c9977bdc339a8ec106dcd9a0357001/Qwen3.8-27B-Uncensored-Q4_K_M.gguf). Its 16,810,714,496 bytes have SHA-256 `3445102e9cde5d562508642c100a2f5ac3368a5a3f748442811d7a95daee3bec`. The adapter verifies every byte against that frozen artifact before parsing the model and preparing its GPU session. The current local implementation uses [compiled ordered chunk commitments](#verify-every-model-byte-with-parallel-chunk-hashes) for this known identity; earlier captures used a serial whole-file hash. Q4_K_M includes mixed tensor types; it does not mean every tensor is four-bit.

Tokenization and single-turn rendering use the separate [publisher source at `8cb32d7`](https://huggingface.co/orcarouter/Qwen3.8-27B-Uncensored/blob/8cb32d72080f6a47bf34dc5adf8067daa6c63a31/tokenizer_config.json). Independent publisher-template fixtures cover Direct and Thinking modes, with and without a system message. The Rust renderer and Hugging Face tokenizer matched those fixture strings and token IDs exactly.

The pinned Hugging Face weight and source repositories require authorized access. A readable model card does not establish permission to download its files. Acquire artifacts separately with the required account access; prompting never grants access or downloads them implicitly.

For each service request, the adapter compares GGUF-vocabulary token IDs with independently encoded Hugging Face IDs before GPU prefill. Any token or ordering mismatch rejects the request. Session preparation can precede this comparison; it is not a promise that no GPU allocation has occurred. The numerical runtime uses [Makepad revision `9e5e3d2`](https://github.com/makepad/makepad/tree/9e5e3d2b03214f8c2c37adffa0e308c815662060), with a resident quantized CUDA session. See [Makepad's execution patterns](makepad-patterns.md).

This local slice is text-only and single-turn. It does not run a vision projector, accept tool calls or preserve conversation history through the prompt command. The Orca GGUF session defaults to 8,192 context tokens, counting the rendered prompt plus requested output. `--context-tokens` selects a capacity from 128 through 32,768 on `prompt`, `interactive` or `benchmark`; invalid capacities and oversized requests fail rather than truncate silently. A larger capacity reserves more runtime state and can change loading and inference costs. This override is specific to the Orca GGUF provider; the older Burn model retains its existing context contract and rejects an explicit override.

`prompt` defaults to Direct mode and 256 new tokens. Named or configured-default OrcaRouter selection with `--thinking` defaults to 512 in the newer local CLI; the legacy Thinking default remains 1,024. An explicit model directory does not receive that named-model default, so set `--max-new-tokens` yourself. Reduce the output budget for longer prompts.

The newer local service checks the exact independently encoded prompt length plus requested output against that capacity before initializing a new GPU session. File and weight validation can already have run on the CPU. This budget check is distinct from the adapter's later comparison of Hugging Face and GGUF token IDs before GPU prefill.

A historical CUDA Thinking run with the earlier 1,024-token capacity asked “What does café mean in English?” It completed successfully with 59 prompt tokens and the requested 24 generated tokens. Its output was bounded reasoning without a final answer when that token limit was reached. This establishes that tested Unicode/Thinking execution path, not completed task accuracy or a useful default output budget.

`--timeout-ms` bounds generation, excluding queue wait and model loading. `--stop-after-duration` requests cooperative cancellation from CLI startup. Cancellation checks occur between operations; they do not preempt an active load operation or GPU kernel. Generated text is an observation, not permission to execute a command.

The earlier release used for the tokenizer-reuse measurements below passed 40 model-free subprocess checks. Both `prompt --timeout-ms 0` and `benchmark --timeout-ms 0` failed with nonzero status and human diagnostics before missing-model access. The newer typed CLI separately passed [73 subprocess checks](interactive-inference.md#inspect-the-implementation-behind-the-interface). Real generation runs and model-free argument checks remain separate evidence.

## Read a prompt from stdin

Use a solitary positional `-` to read UTF-8 prompt text until the producer closes stdin:

```powershell
Get-Content -LiteralPath ./prompts/question.txt -Raw |
  teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
    --context-tokens 8192 --no-thinking --max-new-tokens 32 -
```

The input limit is 2 MiB of bytes. Invalid UTF-8 and oversized input fail before generation. This byte limit is separate from the token capacity: an accepted file can still be too long for the selected session. Missing prompt arguments do not implicitly consume stdin. A `-` among other positional words is ordinary prompt text. Cancellation can end the command even if a pipe producer leaves stdin open.

## Reuse the runtime within one process

A new `prompt` process without `--address` loads its own runtime. Reuse applies to repeated calls through one service instance, the benchmark's `--repeat` loop or [resident interactive inference](interactive-inference.md). An explicitly selected resident service, described below, keeps that runtime across separate client processes.

To measure that distinction for the same 32-token Direct workload:

```powershell
teamy-llm benchmark --model qwen3.8-27b-uncensored-q4-k-m `
  --context-tokens 8192 --no-thinking --max-new-tokens 32 --repeat 3 --timeout-ms 600000 `
  --debug --log-file ./generation-phases.ndjson `
  -- "Why is the sky blue?"
```

Select `--no-thinking` explicitly: benchmark defaults to testing both modes, unlike prompt. The newer local CLI produces a typed report, text on a terminal and JSON when redirected; `--output-format json` selects JSON explicitly. The historical measurements below used the earlier line-oriented report and 1,024-token capacity. Time to first token measures the first token event from request submission, including loading when needed. UTF-8 event buffering and callbacks affect that boundary; it is not pure decoder or kernel time.

## Keep a model resident for fresh clients

Run the service in one terminal with an explicit model and capacity. This Windows example uses the full local named-pipe address:

```powershell
teamy-llm serve --address 'local://\\.\pipe\teamy-llm-service' `
  --model qwen3.8-27b-uncensored-q4-k-m --context-tokens 8192
```

The service verifies and loads the model, then generates one warmup token before starting its IPC listener. Its `resident model ready; starting local IPC listener` diagnostic reports completed prewarming; a client must still establish its connection. Keep this process running. Omitting model selection starts a lazy service whose first request includes loading instead. `serve --context-tokens` requires an explicit prewarm selection.

From another terminal, start a fresh client for each request:

```powershell
teamy-llm prompt --address 'local://\\.\pipe\teamy-llm-service' `
  --model qwen3.8-27b-uncensored-q4-k-m --context-tokens 8192 `
  --no-thinking --max-new-tokens 64 -- "Why is the sky blue?"
```

The client requires an explicit `--model`; `--model-dir` is not accepted with `--address`. The server resolves that name against its own prepared inventory. This command uses local IPC, rejects TCP and remote Windows pipe servers, and fails if it cannot connect. It does not start another server or fall back to loading a local model. On Windows, `serve` defaults to the full pipe shown above; the short `local://teamy-llm-service` spelling is not a valid Windows pipe address.

Use the same model and capacity for prewarming and requests. A capacity change replaces the resident GGUF session and repeats loading instead of retaining two weight copies or silently using a smaller allocation. Exiting the server ends residency. Requested CUDA does not silently become CPU execution.

This workflow moves loading before client requests; it does not make the initial server load disappear. Compare [cold startup and resident latency](performance-analysis.md#separate-process-startup-from-model-residency) before treating a fast fresh-client result as a cold-start result.

## Understand the Windows mmap message

The upstream [non-Unix loader at Makepad revision `9e5e3d2`](https://github.com/makepad/makepad/blob/9e5e3d2b03214f8c2c37adffa0e308c815662060/libs/ai/loader/src/mmap.rs) returns an unavailable-platform error without attempting a Windows file mapping. That explains the message in the historical captures below; elevation, Git long-path settings and renaming the model do not enable that implementation.

Content verification is our adapter's identity policy, not a requirement imposed by Makepad. `crates/teamy_llm_makepad_gguf/src/lib.rs` verifies every GGUF byte when constructing a runtime. Successful requests reuse that verified runtime. A newly constructed runtime, including one created after a capacity change, verifies again. The verification method is recorded separately from the artifact's original whole-file SHA-256.

With the upstream Windows fallback, Makepad then allocated an owned weight arena and its [bulk reader](https://github.com/makepad/makepad/blob/9e5e3d2b03214f8c2c37adffa0e308c815662060/libs/ai/loader/src/bulk_read.rs) read tensor data with unbuffered I/O. Those reads did not use the ordinary filesystem-cache warmup from hashing. The newer local implementation vendors the loader and session hook at the same upstream revision, leaving numerical and CUDA kernel code unchanged. `vendor/makepad-ai-loader/src/mmap.rs` supplies a read-only Windows mapping through `memmap2`, avoiding that owned host-weight arena when mapping succeeds.

The mapping owns its file and releases the view before closing the file. Its handle permits ordinary read sharing but denies write and delete sharing for the opened file's lifetime. For the frozen Orca chunk-verification path, the adapter now retains the actual verified `Arc<MappedRegion>` and passes that same view into the session's weight context. The upload reads the verified view instead of discarding it and reopening the weight pathname. Metadata is still parsed through its file path, and the serial-verification path retains its separate loader mapping. This contract therefore still assumes a stable managed path and immutable backing file; it does not protect against namespace changes, pre-existing writable mappings or privileged mutation. Every-byte identity verification remains enabled. A cached length and modification time would not provide the same content check.

The [CUDA execution backend](https://github.com/makepad/makepad/blob/9e5e3d2b03214f8c2c37adffa0e308c815662060/libs/ai/llm/src/cuda_exec/real.rs) still uploads weights and prepares device buffers and execution graphs. File mapping does not make these costs zero or establish subsecond cold startup. Keep mapped and fallback captures separate and record their build identities.

The session-preparation span combines host staging, device setup and graph work. This backend uses CUDA kernels compiled during the Rust build; a runtime phase named `load llm compile` does not by itself establish repeated CUDA source compilation. The local adapter now logs elapsed times for the session's vocabulary, planning, mapping, device, cache, reserve, upload and compile phases. See [phase attribution](performance-analysis.md#attribute-model-loading-phases) before proposing a cache.

Ollama's [Windows CUDA loader at `dd1d4e9`](https://github.com/ollama/ollama/blob/dd1d4e99e7e8475d1669f566bb5c0ae30db419f1/llm/server.go) explicitly defaults to no mmap, with a performance rationale. Its interactive CLI reuses a server runner instead. This is implementation prior art, not a matched benchmark against our adapter.

Keep first-load identity verification and measure loading separately from resident use. [Terminal interfaces](terminal-interfaces.md#keep-the-loaded-runtime-between-turns) explain the runtime-lifetime pattern; the explicit service workflow extends that lifetime beyond a single client process.

## Verify every model byte with parallel chunk hashes

The current local adapter reduces serial digest work for the frozen Orca artifact by compiling an ordered table of SHA-256 commitments for 16 MiB chunks. The table contains 1,002 digests, including the exact final short chunk: 32,064 bytes of hash data. It contains no model weights.

Authoring the table reads one guarded file in one pass, computing both its ordinary whole-file SHA-256 and the ordered chunk digests from the same bytes. The result is accepted only when the size and whole-file hash match the original frozen identity. Reviewed source independently pins a canonical table root that binds its version, chunk size, file size, original whole-file hash and ordered digests. A mutable table's own claim about its root or model hash is not trusted.

At each new runtime load, bounded CPU workers freshly hash every chunk and compare the results with the compiled table before model use. The default uses reported available CPU parallelism, capped at 32 workers, with one worker when that information is unavailable. An explicit `TEAMY_LLM_GGUF_VERIFY_WORKERS` override selects a supported count within the same bound; verification cannot be disabled. This does not trust a previous successful run, file size or modification time. Independent chunk hashes cannot reconstruct the ordinary whole-file SHA-256; diagnostics identify this method as `compiled-sha256-chunks-v1`, with the original SHA retained as the artifact identity. Unknown identities retain the serial `full-sha256` verification path.

Implementation references in the matching checkout are `crates/teamy_llm_makepad_gguf/src/chunk_verification.rs`, the adapter's `orca_chunk_commitment` and the digest-only `crates/teamy_llm_makepad_gguf/assets/orca-q4-k-m.sha256-chunks.bin`. The file-sharing and stable-path limits described above still apply.

An intermediate diagnostic capture showed about 1.58 seconds between completed chunk verification and the enclosing span's exit. The implementation discarded its populated mapping in that interval, then the loader opened another view. The current session constructor accepts the verified `Arc` directly, and `load_from_mapped_region` retains it through the weight context. This removes the early view disposal and reopening from startup. The combined qualification below does not isolate this change's contribution to first-text latency; final view disposal still occurs during session teardown.

## Overlap CPU validation before device construction

On a new Orca load, the local service overlaps CPU model preflight with tokenizer validation. `inspect_model_files` first checks required files and declared metadata without parsing the tokenizer or treating the artifacts as validated. A named, scoped CPU worker then verifies the GGUF identity and parses its metadata while the calling thread loads the tokenizer through `TokenizerCache::validate`. Generation reuses that exact parsed `Arc<Tokenizer>` instead of parsing and discarding a second copy. Encoding settings and special-token handling remain unchanged.

Both CPU jobs must finish successfully before the service publishes validated artifacts or constructs the GPU session. A tokenizer failure requests cancellation of preflight and joins its worker before returning the original validation error. Preflight failure or panic also returns an error after the join. Cancellation and these errors leave no detached model reader. A matching resident runtime skips repeated weight preflight, while still validating tokenizer freshness. Replacing the model or context capacity creates a new runtime and repeats preflight.

The existing standalone `inspect_model_dir` and callback-based `inspect_model_dir_with_tokenizer_validation` remain fully validating APIs. The explicitly structural `inspect_model_files` result must not be inserted into a validated-artifact cache before tokenizer validation succeeds. Malformed replacements still fail; the tokenizer cache's freshness checks and explicit invalidation remain in effect. Grounding: `crates/teamy_llm_service/src/cpu_preflight.rs`, `inspect_generation_artifacts` and `generate_gguf` in the service, and `crates/teamy_llm_burn_qwen35/src/model.rs`.

This overlap changes scheduling of the same CPU work. It does not establish its speed contribution or make tokenizer parsing preemptible. Check overlapping spans and the external first-text clock when [qualifying startup changes](performance-analysis.md#qualify-a-cold-start-change).

## Stage Windows uploads with bounded pinned memory

The current local Windows upload uses two staging slots, each at most 64 MiB, allocated with CUDA-pinned host memory. The CPU fills the next slot while the previous slot can transfer to the device. Before overwriting either slot, the uploader waits for its recorded completion event. Transfers keep the original tensor bytes, logical offsets and CUDA stream; this change does not alter quantization, kernels or the Unix upload path.

Pinned memory matters because an asynchronous copy from pageable host memory may require driver staging and synchronization. An `Async` suffix alone does not establish overlap. These semantics are documented in [NVIDIA's API synchronization reference](https://docs.nvidia.com/cuda/cuda-runtime-api/api-sync-behavior.html).

The upload owner records each allocation immediately, drains pending transfers before freeing slots, and finishes with stream synchronization. If stream and device drains both fail, it retains the bounded allocations until CUDA teardown rather than freeing memory that DMA might still read. This bounds retained staging memory; it does not make an executing GPU transfer preemptible. Grounding: `vendor/makepad-ai-llm/src/cuda_exec/initial_upload.rs`, called only by the Windows branch of `real.rs`.

The final paired qualification below met the five-second first-visible-text target on all three fresh candidate processes. The historical resident measurements remain separate. See [cold-start qualification](performance-analysis.md#qualify-a-cold-start-change) for the comparison contract and [completed upload timing](performance-analysis.md#measure-completed-upload-work) for interpreting its diagnostics.

## Local release measurement: five-second fresh startup

On 2 October 2026, source revision `9c5011f82b3fb835a18a07172875f576619bc534` produced executable SHA-256 `84b190a4a69664c3f795f84119c6cf0a0d1b55f078cc82fbdadac69f448a3e6b`. Three fresh direct processes were paired with the preserved original executable, SHA-256 `736e48739cb57a9cae9b4af25a5598b3b0c1d92f9ff2929c9b77b86fc95290dd`, alternating which build ran first in each pair.

Both builds used the unchanged Orca Q4_K_M artifact and tokenizer on an RTX 4090, the sky prompt above, 8,192 context tokens, Direct mode and a 256-token output bound. An external monotonic clock started immediately before process launch and measured the first decoded visible model glyph on stdout. No resident model, explicit model/GPU warmup or competing GPU workload was used. Experimental verification, upload-copy and pinned-allocation overrides were unset; the final build used its bounded default of 32 verification workers on this machine.

| Pair | Original first text (s) | Final first text (s) |
| --- | ---: | ---: |
| 1 | 18.09 | 4.76 |
| 2 | 18.01 | 4.62 |
| 3 | 18.71 | 4.69 |

All six processes completed successfully and emitted the same full 256-token text, byte for byte. Separate structured reference and candidate runs confirmed an identical rendered prompt and all 256 generated token IDs. Candidate startup logs recorded fresh verification of all 16,810,714,496 model bytes against all 1,002 compiled chunk digests.

These results establish the tested fresh-process/model-session boundary, including initial loading. Windows file-cache state was uncontrolled and changed naturally through prior reads; this is not a storage-cold or post-reboot experiment. The three candidate samples met five seconds, but do not guarantee that bound for other hardware, prompts, capacities or future runs. First text also differs from full completion: the candidate commands finished in 12.18–12.57 seconds. The combined change preserves fresh content verification and moves host preparation into overlapping CPU work and bounded completed transfers; this paired comparison does not isolate each component's contribution.

## Local candidate measurement: fresh clients and mapped loading

An unpublished local candidate captured on 2 October 2026 used executable SHA-256 `4d1f204b7e11e7f6f5c5b027d22ea0577813da733bb1af5b38049b6c10fd26d4`, the unchanged Orca artifact and an RTX 4090 CUDA runtime. The short workload was “Why is the sky blue?”, 18 formatted input tokens and an eight-token Direct output bound. Each row below launched a fresh client process. External timing began immediately before `Process.Start` and detected the first decoded non-whitespace, non-control stdout character with 5 ms polling. Debug logging was enabled; no periodic GPU or process-memory queries ran during these captures.

| Request | Context capacity | External first visible text, ms | External completed process, ms |
| --- | ---: | ---: | ---: |
| Fresh local process, mapped weights | 8,192 | 18,131.43 | 20,139.42 |
| Fresh client to prewarmed service 1 | 8,192 | 499.50 | 681.75 |
| Fresh client to prewarmed service 2 | 8,192 | 352.52 | 512.11 |
| Fresh client to prewarmed service 3 | 8,192 | 319.68 | 482.31 |

The service's separate warmup logged 17,880.60 ms before starting its listener. Prewarming and readiness are excluded from the fresh-client rows. All three short resident-client samples reached visible text within one second; the fresh local model load did not. These three samples are not a latency distribution or evidence for longer prompts.

The mapped load recorded about 9.422 seconds of full-file identity verification: 2.094 seconds in read calls and 7.328 seconds in digest updates. Its upload phase took about 6.766 seconds. These are instrumented CPU-side intervals; the upload label does not isolate physical transfer time or GPU execution. Verification remains a substantial cost under our identity policy, and mapping alone did not meet the subsecond cold-start target.

An earlier executable, SHA-256 `901540128871bd35cca6bb32dc8468932d6621b052e3a69e12589bd52125e68d`, reached visible text in 23,612.21 ms using the older 1,024-token capacity and owned-arena fallback. The eight-token output matched the candidate's short-prompt text. The captures were not randomized, used different capacities and did not control the operating-system cache state. Their time difference does not isolate the effect of mapping or establish an independent numerical-quality comparison.

A separate resident stdin request preserved a synthetic 100-row directory listing: 3,232 formatted input tokens and eight generated tokens completed successfully at the default 8,192 capacity in 14,486.62 ms externally. It selected JSON output, so that duration measures completed structured output, not streamed time to first model text. An explicit 32,768-capacity session also loaded and generated the same short eight-token answer, completing in about 21,426 ms with only 18 input tokens. This checks allocation and the short-input path; it does not establish 32,768-token quality or performance.

The local evidence identifiers are `latency-mapped-cold-8192-v1`, `latency-resident-fresh-client-v1-1` through `-3`, `latency-resident-stdin-listing-v1` and `latency-context32768-direct-v1`. Their executable identity describes the capture build; a later commit, rebuild or installation has its own identity.

The later installed build, compiled from local revision `eb19d98958a5926f422b8f9f771d760caaae161e`, has executable SHA-256 `736e48739cb57a9cae9b4af25a5598b3b0c1d92f9ff2929c9b77b86fc95290dd`. Its three fresh clients reached visible text in 409.38, 357.51 and 338.39 ms using the same short workload and external clock. A single daemon also completed capacity changes from 8,192 to 32,768 and back to 8,192, the complete 3,232-token stdin fixture, and cancellation followed by a successful request. Its configured shutdown cancelled and drained work. This confirms those paths in that build; it does not establish full-window model quality or remove cold loading. The evidence identifier is `latency-final-resident-v2`.

## Local release measurement: tokenizer reuse

This historical experiment used the earlier 1,024-token capacity and upstream Windows owned-arena fallback, before the local mapping addition. Two fresh Windows x64 release processes ran the same prompt, 18 prompt tokens and three 32-token completions on an RTX 4090 with driver 610.88. No Cargo build or other inference run overlapped either capture. Debug stderr, NDJSON phase logs and external memory sampling were enabled.

| Time to first token event, milliseconds | Before tokenizer caching | With tokenizer caching |
| --- | ---: | ---: |
| First request in a cold process | 22,814.061 | 22,800.184 |
| Resident request 2 | 842.844 | 228.415 |
| Resident request 3 | 821.847 | 186.938 |

The cache reduced repeated tokenizer setup; cold startup remained about 22.8 seconds. The optimized capture's phase spans recorded about 10.1 seconds for weight hashing, 49 milliseconds for model parsing and 11.4 seconds for session preparation. Its first `llm_tokenize` span took about 402 milliseconds; later spans took 405 and 423 microseconds. These logged CPU-side phase durations are not GPU kernel timings or an additive account of every startup cost.

The optimized executable's SHA-256 is `0bd68ca0fb0fe14a302a55d35dc7c5509e9e1cf0c81f97b8c7ca18d7e73fd899`; the earlier executable was `57b31a64c6afdf54fcdcce65e68377ffa03ac0b62285e518e1d8ba246fff9e37`. The tokenizer cache retains encoding settings and reloads when file length or modification time changes. Those freshness hints are not immutable content checks; replacing a file while preserving both requires explicit invalidation.

These are two resident samples per executable, not a latency distribution. Process memory was sampled at a requested 250 ms interval; sample maxima are not true peaks. GPU readings cover the whole device and do not establish per-process allocation. The model completed real CUDA generation and passed independent prompt/token parity. Independent Python-to-Makepad numerical or logit parity was not established, nor was an output-quality ranking against the older 9B model. Follow [performance analysis](performance-analysis.md) before attributing another delay to the same cause.
