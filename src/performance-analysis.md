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

The prompt command logs both `first generated token` and `first visible model text`. The first event may contain empty or whitespace text; the visible-text event identifies usable decoded text. Both clocks start inside the command handler before prompt input and connection work, rather than at operating-system process launch. With text output, the visible event follows its stdout write and flush. With JSON output, it records text readiness; the structured result is written only after completion. A terminal display or external process-start measurement has a different boundary.

Compare the same model, rendered input, output bound, mode, context, logging settings and build. Retain successful token or score parity alongside the timings. Run loading and fresh-client experiments sequentially so another model or build cannot compete for the GPU during the capture.

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

For an existing capture, these project-relative paths are illustrative:

```powershell
teamy-profiler tracy csv export ./tracy/review.tracy --top 25 --table
teamy-profiler tracy csv export ./tracy/review.tracy `
  --top 25 --output ./tracy/top-spans.csv
```

The second command creates or truncates its named output file. The [exporter](https://github.com/TeamDman/teamy-profiler/blob/9e2379a9335aa0bae8acd917d6a85b7269807deb/src/cli/tracy/tracy_cli.rs) ranks aggregate self/exclusive CPU time. Its percentage divides summed span time by capture wall time; parallel threads can exceed 100%. A long parent span and its child spans must not be added together as independent costs.

The [decoder](https://github.com/TeamDman/teamy-profiler/blob/9e2379a9335aa0bae8acd917d6a85b7269807deb/src/tracy_native/mod.rs) supports zstd-compressed Tracy 0.13.1–0.13.3 captures and stops after CPU timelines. It does not analyze GPU zones, allocations or call stacks. The later locally inspected profiler change inherits capture-process diagnostics; that change is not in the cited public revision.

## GPU submission is not completion

A CPU span around a GPU call may measure enqueueing, allocation, compilation or a wait. It does not establish device execution time. Use backend events or a documented synchronization boundary when measuring completed work; record which streams and operations the boundary covers. Synchronizing every operator can remove overlap and distort the workload. PyTorch's [CUDA timing guidance](https://docs.pytorch.org/docs/2.11/notes/cuda.html#asynchronous-execution) explains the same distinction for the independent reference.

Keep a minimally instrumented performance run separate from a diagnostic run that copies intermediate tensors to the CPU. Retain correctness evidence, repeat count, warmup policy and memory measurements with both. Use a capture to identify a candidate change, then repeat the same workload and parity gate after making that change.

## Find repeated setup inside a resident request

The historical unpublished [text-generation measurement](local-text-generation.md#local-release-measurement-tokenizer-reuse), using 1,024-token capacity before Windows mapping, provides a concrete example. A loaded GPU model still had roughly 822–843 ms to its first token event on two repeated requests. Phase logs showed that each request rebuilt the tokenizer. Keeping the unchanged tokenizer in the service reduced its later `llm_tokenize` spans to 405 and 423 microseconds; first-token event times became roughly 187–228 ms for the same prompt and output bound. The independent publisher prompt/token checks remained in place.

Cold first-token latency stayed near 22.8 seconds. Weight verification and session preparation still dominated the recorded loading phases. The change improved repeated host setup; it did not demonstrate faster GPU kernels or remove cold loading. The linked chapter retains executable identities, raw timing samples, instrumentation and qualification limits. Use this pattern to investigate repeated setup before assuming the model's numerical work explains the entire delay.

The newer local startup change addresses a different repetition: `inspect_model_dir` previously parsed the tokenizer and discarded it before `TokenizerCache::get` parsed it again for generation. The service now validates through that cache and retains the exact parsed `Arc<Tokenizer>` for encoding. Its fresh Orca path overlaps this validation with CPU model preflight; other validated inspection uses the callback API. The standalone inspector still performs its full validation. This removes one parse while preserving encoding options, special-token IDs, malformed-replacement rejection and explicit invalidation. The tokenizer's file-length/mtime freshness hints remain distinct from the GGUF weight verifier's every-byte identity checks. Measure the retained `llm_tokenizer_load` and the later cache hit during `llm_tokenize`; the historical tokenizer table predates these additional changes.
