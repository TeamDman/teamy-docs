# Find where a program spends time

Start with a repeatable workload and a correct result. Use [tracing](logging.md) to divide the work into phases, then use a capture to test a specific explanation for the delay. Keep the input, build profile, features, model revision and precision settings alongside the result. A faster run that changes the task or fails [reference parity](python-ml-to-rust.md#establish-parity-before-optimizing) is a different experiment.

## Give the delay a boundary

Measure the user's wait and the internal phases separately. These boundaries are a proposed instrumentation strategy; they are not already present in every Teamy program.

| Boundary | What it separates |
| --- | --- |
| Process start to ready | Configuration, disk reads, model loading, device setup and required warmup. |
| Accepted request to work start | Queueing and scheduling. |
| Work start to encoded input | Tokenization, audio preprocessing and host staging. |
| Encoded input to completed device work | Uploads, numerical execution and synchronization. |
| Completed work to visible result | Decode/postprocessing, serialization, transport and presentation. |

Keep cold process startup, first use of a shape or kernel, and repeated resident requests separate. Record whether compilation and disk-cache warming are included. For generation, define first-token timing explicitly; for a finite-choice model, measure the completed ordered score result. A slow first output alone does not identify whether loading, compilation, queueing or inference caused it.

For the local resident CLI, use the [interactive timing fields](interactive-inference.md#run-repeatable-sessions-without-a-terminal) to distinguish the parent's submission-to-reply clock from the worker's job duration. [Mode changes and decision cancellation](interactive-inference.md#separate-model-residency-from-conversation-history) can discard residency, so separate those requests from successful repeated requests within one mode.

## Separate process startup from model residency

Choose the boundary that matches the intended experience. A fresh client can be fast while the process serving it has already spent time loading the model:

| Workload | Where loading occurs |
| --- | --- |
| Fresh `prompt` without `--address` | The client constructs its own runtime before generating text. |
| Repeated `benchmark` or interactive requests | The first request loads; later requests reuse the same service or worker within that mode. |
| Fresh `prompt --address` against a prewarmed service | `serve --model` loads and generates a warmup token before starting its listener; the client connects to that existing runtime. |

The [resident-service example](local-text-generation.md#keep-a-model-resident-for-fresh-clients) uses an explicit model and the full Windows local-pipe address. Keep its context capacity consistent across prewarm and requests: changing capacity replaces the runtime. The current Orca default is 8,192 tokens; record an explicit `--context-tokens` value when comparing builds. The historical captures below used 1,024. Julia's encoding budget is a separate setting.

Record initial server loading and client response time separately. A newly launched process is not necessarily a cold disk-cache experiment, and a ready server is not a cold model. Include process launch, CLI setup, stdin acquisition and connection establishment when measuring the user's wait from launching a fresh client. None of the earlier resident measurements establishes subsecond cold startup.

The prompt command logs both `first generated token` and `first visible model text`. The first event may contain empty or whitespace text; the visible-text event identifies usable decoded text. Both clocks start inside the command handler before prompt input and connection work, rather than at operating-system process launch. In local service revision `78e933371f9cc13a6909cf8dfaf83b132e73baba`, both timing diagnostics precede the first visible text write, so they do not split the opening word from its answer. `streaming_to_stdout` describes the intended destination; the visible event measures readiness before logging, writing and flushing, rather than confirming delivery. Writer failures still propagate. With JSON output, the structured result is written only after completion. A terminal display or external process-start measurement has a different boundary. Grounding: `PromptObserver` and its shared-terminal-sink regression in `crates/teamy_llm_cli/src/cli/prompt/llm_prompt_cli.rs`.

Compare the same model, rendered input, output bound, mode, context, logging settings and build. Retain successful token or score parity alongside the timings. Run loading and fresh-client experiments sequentially so another model or build cannot compete for the GPU during the capture.

## Estimate the remaining startup gains

Specify which cache is warm before choosing a target. These are different starting states for the unchanged Orca Q4_K_M model:

| Starting state | Evidence or limit |
| --- | --- |
| Fresh process, empty GPU, uncontrolled Windows file cache | [Qualified historical first-text measurements](local-text-generation.md#local-release-measurement-five-second-fresh-startup): 4.62–4.76 seconds. |
| Fresh process, weights absent from the OS file cache | Not measured separately. Storage must supply the model bytes; prior reads do not establish this boundary. |
| Prepared weights retained in CPU RAM, empty GPU | Not measured separately. Parsing and some host preparation may be reused; GPU upload remains. |
| Fresh client, model already on the GPU | The same five-second release's three prewarmed-service captures reached first visible text in 415.26, 345.16 and 342.57 milliseconds. Loading and server readiness happened earlier. |
| Resident model plus reusable prompt-prefix state | No qualified latency result yet. Distinguish saved KV state from caching a completed answer. |

The model contains 16,810,714,496 bytes; the captured initial upload arena contains 17,504,478,592 bytes including working state. For an RTX 4090 on PCIe Gen4 ×16, NVIDIA's rounded ideal bandwidth of 32 GB/s in one direction puts a transfer-only floor at approximately 0.53 seconds for the model and 0.55 seconds for that arena. These are optimistic calculations, excluding overhead, verification and inference. The bidirectional total is not upload bandwidth. See [RTX 4090 specifications](https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/rtx-4090/) and [NVIDIA's PCIe bandwidth explanation](https://developer.nvidia.com/blog/nvidia-hopper-architecture-in-depth/).

A 100-fold reduction from roughly 4.5 seconds would require roughly 45 milliseconds. Loading these bytes into an empty GPU over that link cannot meet it. CPU file-cache warmth can remove storage reads, but does not make weights GPU-resident. Keeping a loaded service changes the starting state and already produces a much larger improvement than a file-format change alone.

The current mapped GGUF path already retains quantized tensor bytes. A prepared format could reduce metadata, vocabulary or allocation-plan construction; it would still need content verification and transfer. The next substantial experiment is to pipeline authenticated chunks through verification, pinned staging and completed upload, or avoid the mapped-to-pinned copy where Windows/CUDA supports it. Inference must wait for successful verification of the complete artifact, and cancellation/error cleanup must drain outstanding transfers. NVIDIA documents the benefits and costs of [pinned memory and asynchronous transfers](https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/index.html#data-transfer-between-host-and-device).

Around 2–3 seconds for a fresh process with RAM-cached weights and an empty GPU is a next engineering target, not a measured bound or guarantee. Measure that cache state separately before claiming it. Existing verification and upload spans support investigating this work; they do not prove the benefit of a pipeline that has not been implemented. Storage-cold performance also needs a storage measurement. Keep the original model, verification contract and numerical parity fixed when comparing candidates.

## Add useful spans before detailed events

Create a coarse span around each meaningful phase. Record request IDs, shapes, counts and selected backend, rather than dumping prompts, tensors or filesystem contents. Use debug events for allocation, cache and lifecycle decisions; use trace events for detailed steps when they answer a question. Instrumentation itself can change allocation and timing.

The template accepts `tracing` instrumentation and installs an optional Tracy layer. Its [public logging setup at `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs) gives Tracy an independent trace filter: making terminal logs quiet does not disable that collection. See [logging filters and destinations](logging.md#terminal-logs-ndjson-and-tracy).

`#[tracing::instrument(skip_all)]` avoids automatically recording arguments, but explicit fields still need review. For async work, do not retain a `span.enter()` guard across an `.await`; instrument the future or use the instrumentation attribute. See the [`instrument` API](https://docs.rs/tracing/0.1.44/tracing/attr.instrument.html) and [async span guidance](https://docs.rs/tracing/0.1.44/tracing/span/struct.Span.html#in-asynchronous-code).

## Attribute model-loading phases

The local GGUF adapter uses coarse spans and more detailed debug events:

| Observation | Included work |
| --- | --- |
| `gguf_verify_identity` | Every-byte verification against the frozen artifact: compiled ordered chunk commitments for the known Orca identity, otherwise a serial whole-file SHA-256. |
| `gguf_load_model` | Makepad parses the model metadata. |
| `llm_tokenizer_load` | Tokenizer parsing retained by the service cache; on a fresh Orca load this can overlap CPU identity verification and metadata parsing. |
| `gguf_prepare_session` | The session constructs its vocabulary, plans buffers, obtains host weights, allocates device state, uploads weights and prepares execution. |
| `GGUF session preparation phase` debug events | Elapsed milliseconds between progress-stage transitions, including initial metadata cloning and validation. |

These boundaries come from `crates/teamy_llm_makepad_gguf/src/lib.rs` in the matching service checkout. The progress hook in `vendor/makepad-ai-llm/src/session.rs` distinguishes vocabulary, plan, mapping, device, cache, reserve, upload and compile stages. The adapter groups changing upload labels into one upload phase and changing compile labels into one compile phase. Progress fractions are approximate progress markers, not elapsed time or measured proportions of total cost. Use each event's `elapsed_ms` when attributing its CPU-side duration; the parent span already contains these intervals.

The identity check is our policy, not a Makepad prerequisite. It runs once per constructed runtime and stays enabled with the [local read-only Windows mapping](local-text-generation.md#understand-the-windows-mmap-message). The newer local [chunk verification](local-text-generation.md#verify-every-model-byte-with-parallel-chunk-hashes) authenticates a compiled table and freshly hashes every model byte in bounded parallel workers. Its table was authored in one guarded pass that also matched the original whole-file SHA-256. A table root supplied by that same mutable table would not establish trust. Unknown identities retain full serial hashing; size and modification time never replace weight-content verification.

Mapping changes the host-weight path; GPU uploads, buffer planning and first-use execution remain. Likewise, two reserve stages can prepare different token-count shapes rather than duplicate identical work. Keep the input and resulting plan equivalent when testing a planner optimization.

The local service now [overlaps CPU preflight and tokenizer validation](local-text-generation.md#overlap-cpu-validation-before-device-construction). `cpu_preflight.rs` starts a named scoped worker for GGUF verification and metadata parsing, joins it after tokenizer validation and permits GPU construction only after both succeed. Structural inspection is explicit and is not published into the validated-artifact cache prematurely. The exact formatted prompt/output budget is checked before new GPU initialization. Resident requests with matching model and capacity reuse their verified runtime.

Read these intervals as a schedule: `llm_tokenizer_load` and CPU GGUF preflight can run at the same time. Adding their durations overstates elapsed startup work. A failure still joins the CPU worker; cancellation is cooperative, and parsing is not an interruptible GPU boundary. Inspect each span's time and thread together with the enclosing wall-clock interval before assigning the benefit of overlap.

Include resource release in the phase analysis. An intermediate chunk-verification capture showed about 1.58 seconds after its completion diagnostic before `gguf_verify_identity` closed; source inspection identified disposal of the populated verification mapping in that interval. The local implementation now retains that exact verified `Arc<MappedRegion>` through session construction and the weight context, avoiding early disposal and another weight view. Metadata still has its separate path-based read. The combined startup qualification does not isolate this change's first-text contribution. Retention moves final view release to session teardown, so measure full command completion as well as first text before claiming that total work disappeared.

The [mapped-loading candidate capture](local-text-generation.md#local-candidate-measurement-fresh-clients-and-mapped-loading) demonstrates this attribution: its full-file hash recorded 2.094 seconds in read calls and 7.328 seconds in digest updates, while the upload phase recorded 6.766 seconds. These intervals support investigating digest and upload work separately; they do not establish storage bandwidth, device-copy time or GPU kernel time. The same build's short fresh-client requests reached visible text within one second only after separate service prewarming. The initial model load still exceeded that target.

Runtime phase names alone do not identify repeated source compilation. The pinned CUDA backend uses kernels compiled during the Rust build; a phase named `load llm compile` can include preparing a graph for execution. Establish the actual work in that phase before adding a compilation cache. Measure a change with full identity checks and parity intact before claiming an improvement in cold time to first visible text.

## Measure completed upload work

The local Windows [two-slot pinned uploader](local-text-generation.md#stage-windows-uploads-with-bounded-pinned-memory) now emits the debug-level tracing event `Initial CUDA arena upload completed`, with target `teamy_llm_makepad_gguf::cuda::upload`. Its host, submission, completion and wall-clock fields retain the same boundaries below. Historical captures used the raw `cuda.upload` stderr prefix; enable `--debug` or an explicit target filter for the newer [logging pipeline](logging.md#keep-runtime-diagnostics-in-the-logging-pipeline).

| Field | Boundary |
| --- | --- |
| `device_alloc_ms`, `host_alloc_ms` | Device allocation and the two bounded host staging allocations. |
| `cpu_staging_ms` | CPU copies from the original host bytes into reusable pinned slots. |
| `enqueue_ms` | CPU time recording events and submitting asynchronous transfers. |
| `completion_wait_ms` | CPU time waiting until each slot's previous transfer is complete. |
| `h2d_completed_event_ms` | Summed CUDA event intervals around host-to-device transfers, queried after completion. |
| `clear_enqueue_ms`, `final_sync_ms` | Dirty-state clearing submission and the final stream drain. |
| `upload_wall_ms` | Elapsed wall time for staging, submission, completion and upload-owner setup. |

CPU staging can overlap the prior slot's host-to-device transfer. Event intervals, host waits and wall time therefore cannot be added as independent costs. A completed event interval still reflects its CUDA stream's scheduling; it is not a standalone bus-bandwidth measurement. Progress advances after completion, rather than counting submitted bytes as completed bytes. Inspect `bytes`, `slots`, `slot_bytes` and all error diagnostics alongside the timings.

The implementation waits for an event before slot reuse and queries elapsed event time only after that wait. [NVIDIA's event reference](https://docs.nvidia.com/cuda/cuda-runtime-api/cuda_runtime_api/group__CUDART__EVENT.html) defines these completion and timing boundaries. Its [synchronization reference](https://docs.nvidia.com/cuda/cuda-runtime-api/api-sync-behavior.html) also explains why pageable-memory staging can synchronize an apparently asynchronous transfer. Preserve these boundaries when changing the uploader; changing a label or measuring submission alone is not an optimization of completed work.

## Qualify a cold-start change

The [paired fresh-startup measurement](local-text-generation.md#local-release-measurement-five-second-fresh-startup) reached visible model text in 4.62–4.76 seconds on all three final-build runs, compared with 18.01–18.71 seconds for the preserved original executable. The workload kept the Orca Q4_K_M artifact, RTX 4090, 8,192-token capacity and 256-token Direct bound fixed; all six complete text outputs matched byte for byte. Separate structured runs confirmed identical rendered input and all 256 generated token IDs. The earlier resident measurements remain evidence of a different boundary.

The combined change comprises parallel every-byte verification against the compiled 1,002-chunk Orca table, retention of the actual verified mapping, bounded pinned Windows uploads, removal of the discarded tokenizer parse and overlap of CPU model preflight with tokenizer validation. It preserves the model artifact, tensor representation and encoding contract. The matched comparison qualifies the combined result, rather than assigning each change an independent speedup.

Use at least three fresh direct processes per executable, with the frozen baseline and candidate in alternating pair order. Keep the Orca Q4_K_M artifact, tokenizer, rendered sky prompt, Direct mode, 8,192-token capacity, 256-token output bound and logging settings identical. Do not start a resident server or run an explicit model/GPU warmup. Keep all runs, including the first and any failures; qualify every candidate run against the target rather than selecting the fastest sample.

Start an external monotonic clock immediately before launching the process and stop first-text timing at its first decoded visible model glyph on stdout. Diagnostics, help, progress, ANSI decoration and JSON framing do not count as model text. Retain complete and partial stdout/stderr, exit status, timeout and executable identities. Check the full generated text against the frozen baseline and qualify token-ID parity separately. A JSON completion receipt measures completed structured output, not first streamed text.

Record the operating-system file-cache state as uncontrolled unless it was independently managed. Fresh process and GPU-session state do not establish storage-cold performance. Hashing the full weight file before a timing run itself reads and warms its pages; keep artifact-authoring and provenance work outside the experiment and describe their relationship to it. Compiler caches and prior filesystem reads must also remain visible in the experiment's limits. Run each process sequentially without competing GPU workloads or periodic GPU polling in the timed window.

Record the inherited `TEAMY_LLM_GGUF_VERIFY_WORKERS` and `MAKEPAD_LLAMA_CUDA_UPLOAD_COPY_THREADS` settings with the executable identity. They affect verification and host staging parallelism, so differently configured runs do not establish the default configuration's performance. Without a verification override, the current local default uses reported available CPU parallelism capped at 32; retain the actual `workers` field from its diagnostic. Also retain the pinned-memory allocation setting, `MAKEPAD_LLAMA_CUDA_UPLOAD_WRITE_COMBINED`, when comparing upload experiments.

## Capture and inspect with teamy-profiler

The verified public [`teamy-profiler` revision `9e2379a`](https://github.com/TeamDman/teamy-profiler/tree/9e2379a9335aa0bae8acd917d6a85b7269807deb) includes both a Cargo capture harness and a native CPU-span decoder. The template's [public `run-profiler.ps1`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/run-profiler.ps1) uses that harness with release mode and the `tracy` feature.

The harness builds the selected binary or example, starts `tracy-capture.exe`, runs the target, then records a manifest and summarizes the capture. It forwards requested features and target arguments, and adds an NDJSON log argument when the target's help advertises `--log-file`. It records build, command, capture-shutdown and postprocessing times separately. Inspect target exit, capture-shutdown and postprocessing status before using the CSV. Its `--dry-run` avoids build and target/capture launch, but still runs Cargo metadata and writes its plan manifest. Grounding: [Cargo harness](https://github.com/TeamDman/teamy-profiler/blob/9e2379a9335aa0bae8acd917d6a85b7269807deb/src/cli/run/cargo/run_cargo_cli.rs).

The NDJSON file complements the Tracy capture. The current profiler commands summarize `.tracy` CPU spans; they do not ingest NDJSON or turn its close events into a phase report. Inspect those structured events separately. Tracy's exclusive CPU summary helps find host hotspots, while named NDJSON phases help relate setup, model selection and completed requests to one another.

For an existing capture, these project-relative paths are illustrative:

```powershell
teamy-profiler tracy csv export ./tracy/review.tracy --top 25 --table
teamy-profiler tracy csv export ./tracy/review.tracy `
  --top 25 --output ./tracy/top-spans.csv
```

The second command creates or truncates its named output file. The [exporter](https://github.com/TeamDman/teamy-profiler/blob/9e2379a9335aa0bae8acd917d6a85b7269807deb/src/cli/tracy/tracy_cli.rs) ranks aggregate self/exclusive CPU time. Its percentage divides summed span time by capture wall time; parallel threads can exceed 100%. A long parent span and its child spans must not be added together as independent costs.

The [decoder](https://github.com/TeamDman/teamy-profiler/blob/9e2379a9335aa0bae8acd917d6a85b7269807deb/src/tracy_native/mod.rs) supports zstd-compressed Tracy 0.13.1–0.13.3 captures and stops after CPU timelines. It does not analyze GPU zones, allocations or call stacks. The later locally inspected profiler change inherits capture-process diagnostics; that change is not in the cited public revision.

## Diagnose Julia with NDJSON before timing

Use the same text-choice workload for diagnosis and qualification. Collect detailed phases in a separate run with an explicit fresh log filename:

```powershell
teamy-llm decide --prompt "How to add structured logging to a Rust command-line application." `
  --choice "Writing programs" --choice "Playing games" `
  --choice "Managing photos" `
  --debug --log-file ./profiles/julia-phases.ndjson

Get-Content ./profiles/julia-phases.ndjson |
  ForEach-Object { $_ | ConvertFrom-Json } |
  Where-Object { $_.fields.message -eq 'close' } |
  Select-Object timestamp, @{n='span';e={$_.span.name}}, `
    @{n='busy';e={$_.fields.'time.busy'}}, `
    @{n='idle';e={$_.fields.'time.idle'}}
```

The command requires an already recorded native Julia artifact location; [explicit overrides and the text input contract](finite-choice-models.md#score-text-choices-from-the-terminal) are unchanged. The CLI appends to `--log-file`, so use a new filename for each experiment. Prompts need not be logged to measure phase durations. Direct `decide` preserves structured span fields; an interactive worker's existing relay retains formatted line text under `worker_log` instead.

| Phase in the matching local implementation | Timing boundary |
| --- | --- |
| `julia_tokenizer_load` / `julia_tokenizer_parse` | Tokenizer file loading and parsing. These spans are nested and can overlap native CUDA/cache setup. |
| `julia_native_cuda_verify` | Native driver/NVRTC queries, exact compiler-library matching and device architecture discovery; no model inference. |
| `julia_kernel_cache_setup` | Executable/toolkit content hashing, namespace selection and persistent-cache lease/configuration. |
| `julia_checkpoint_prepare` | CPU-only configuration, immutable checkpoint mapping and full embedding validation, overlapped with CUDA/cache setup. |
| `julia_load` / `julia_native_model_load` | Initial provider preparation, including parallel tokenizer parsing, and its nested numerical preparation respectively. Their host durations alone do not prove that all queued transfers have completed. |
| `julia_embedding_validate` | Host finite-value scan of every embedding row, including unused rows. Shape/range checks precede this span. |
| `julia_embedding_gather` | Checked host row gathering and submission of the ordered input embedding tensor. `upload_bytes` describes input size, not completed transfer time. |
| `julia_encoder_layer` / `julia_decision_layer` | Host work for a model layer, including any implicit waits. These are not CUDA device-time measurements. |
| `julia_native_infer` | Numerical inference through the final logits readback and finite-logit check. |
| `julia_decide` | Encoding, completed inference and validated ordered scores for one repetition; initial model/tokenizer loading and stdout serialization are outside this span. |

Grounding: `JuliaEncoder` and `StreamingBpe` in `crates/teamy_llm_julia_common/src/{lib,streaming_bpe}.rs`, runtime setup and logits `into_data().convert::<f32>().to_vec()` readback in `crates/teamy_llm_burn_julia/src/{lib,model,cuda_runtime,kernel_cache}.rs`, and `DecisionEngine::evaluate_json` in `crates/teamy_llm_cli/src/cli/decide/llm_decide_cli.rs`. These paths refer to the matching local service checkout. Rounded `time.busy` and `time.idle` strings describe span lifetime; adding nested spans double-counts work. The readback ensures the GPU operations needed for those logits have finished, rather than merely been submitted.

Read these intervals as a schedule. The fresh native CLI parses the tokenizer on one scoped CPU worker and prepares its immutable host checkpoint on another, while the caller verifies CUDA and prepares its compiled cache. After joining the checkpoint worker, the caller creates numerical tensors; successful encoding precedes inference. `load-ms` covers this parallel preparation. Adding overlapping or nested durations overstates elapsed work. Capability and policy checks precede startup; numerical preparation may begin before encoding finishes. Every worker is joined on success or failure. Command-line startup, output and teardown still sit outside `load-ms`.

For final latency, run fresh CLI processes sequentially with the same prompt and three choices, using `--log-filter warn` and no NDJSON or Tracy capture. Start an external monotonic clock immediately before process launch; stop only after exit and both stdout and stderr reach EOF. Require exit zero, exactly three finite normalized probability lines and independent agreement with the structured result's choice order. The original independent encoding, ID, logit and probability gate remains separate from the output-shape check.

Keep the first run and every later run. For a subsecond completion goal, every qualified sample must finish below 1,000 milliseconds; first stdout, internal `inference-ms` and the fastest sample are different measurements. Do not prewarm the model or run another GPU workload during fresh-load qualification. Record the executable, configuration, logging filter and uncontrolled operating-system file and driver caches.

Identify compiled-kernel cache state separately. Julia's local CUBIN cache reuses executable/toolkit/device-specific compiled code, while every fresh CLI still constructs its model and uploads the required tensors. A new binary selects a new namespace. An empty-cache first use can include NVRTC compilation; a later fresh process can load its existing CUBIN images. Those processes are model-cold but compilation-cache warm. A resident decision reuses both model and compiler state. Retain all three cases instead of presenting their times as one cold-start result. A diagnostic run can itself populate the cache, so record whether it ran before qualification.

The combined release has passed the unchanged six-case parity gate and three installed complete-process measurements: 953.3, 894.3 and 883.6 ms with its populated default CUBIN cache. Its first default-cache invocation, captured separately with diagnostics, took 2,930.7 ms. These are model-cold processes with uncontrolled operating-system file caches, not a storage-cold guarantee. The [release measurement](finite-choice-models.md#local-release-measurement-subsecond-julia-decisions) records exact identities, resident timings, validation and limits. [Implementation decisions](finite-choice-models.md#keep-julias-cuda-precision-explicit) explain the changes.

## GPU submission is not completion

A CPU span around a GPU call may measure enqueueing, allocation, compilation or a wait. It does not establish device execution time. Use backend events or a documented synchronization boundary when measuring completed work; record which streams and operations the boundary covers. Synchronizing every operator can remove overlap and distort the workload. PyTorch's [CUDA timing guidance](https://docs.pytorch.org/docs/2.11/notes/cuda.html#asynchronous-execution) explains the same distinction for the independent reference.

Keep a minimally instrumented performance run separate from a diagnostic run that copies intermediate tensors to the CPU. Retain correctness evidence, repeat count, warmup policy and memory measurements with both. Use a capture to identify a candidate change, then repeat the same workload and parity gate after making that change.

## Find repeated setup inside a resident request

The historical unpublished [text-generation measurement](local-text-generation.md#local-release-measurement-tokenizer-reuse), using 1,024-token capacity before Windows mapping, provides a concrete example. A loaded GPU model still had roughly 822–843 ms to its first token event on two repeated requests. Phase logs showed that each request rebuilt the tokenizer. Keeping the unchanged tokenizer in the service reduced its later `llm_tokenize` spans to 405 and 423 microseconds; first-token event times became roughly 187–228 ms for the same prompt and output bound. The independent publisher prompt/token checks remained in place.

Cold first-token latency stayed near 22.8 seconds. Weight verification and session preparation still dominated the recorded loading phases. The change improved repeated host setup; it did not demonstrate faster GPU kernels or remove cold loading. The linked chapter retains executable identities, raw timing samples, instrumentation and qualification limits. Use this pattern to investigate repeated setup before assuming the model's numerical work explains the entire delay.

The newer local startup change addresses a different repetition: `inspect_model_dir` previously parsed the tokenizer and discarded it before `TokenizerCache::get` parsed it again for generation. The service now validates through that cache and retains the exact parsed `Arc<Tokenizer>` for encoding. Its fresh Orca path overlaps this validation with CPU model preflight; other validated inspection uses the callback API. The standalone inspector still performs its full validation. This removes one parse while preserving encoding options, special-token IDs, malformed-replacement rejection and explicit invalidation. The tokenizer's file-length/mtime freshness hints remain distinct from the GGUF weight verifier's every-byte identity checks. Measure the retained `llm_tokenizer_load` and the later cache hit during `llm_tokenize`; the historical tokenizer table predates these additional changes.
