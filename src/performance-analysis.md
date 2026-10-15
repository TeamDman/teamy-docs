# Optimize performance with measured expectations

Start with a defined workload and a correct result. Use [tracing](logging.md) to divide the work into phases, then use a capture to test a specific explanation for the delay. Keep the input, build profile, features, model revision and precision settings alongside the result. Controlled comparisons need repeatable inputs; an ongoing work loop also needs measurements after real changes. A faster run that skips required work or fails [reference parity](python-ml-to-rust.md#establish-parity-before-optimizing) is not an improvement.

This is the starting point for fixing a slow Teamy program, including a CLI, desktop app or agent's repeated tooling loop. New CLIs inherit the [Rust template](rust-cli-template.md); retain its logging and profiling support so a slow command can supply evidence without adding a second diagnostics system.

## Follow one optimization loop

1. Define the workload, correct result and reasonable elapsed time. Measure the whole user's wait with a monotonic clock; separate build time, process startup and repeated requests.
2. Record a baseline with the same executable, inputs and cache state. Keep several runs and failures rather than selecting the fastest sample.
3. Use `--log-file` and coarse phase spans first. When phase timings cannot explain the delay, capture Tracy and inspect the CPU timeline or hottest spans.
4. Change the expensive work: remove repeated discovery, improve the algorithm, reuse preparation or cache with explicit invalidation. Recheck correctness against the full reference.
5. Repeat the original measurement and encode the accepted runtime expectation in the command. A warning makes a later regression visible during ordinary use.

For the [SFM migration loop](migration-workflows.md#measure-the-complete-iteration), the reported next-item command took 80 seconds. That observation motivates measurement and a runtime warning; it does not establish which operation was expensive. Measure selection, transformation, validation and coordination separately, and optimize accepted work per elapsed minute.

## Optimize while advancing useful work

For a selector that tells an agent what to do next, keep performance improvement and task progress in one continuing loop:

1. Measure the selector on the current state and record the work it identifies.
2. Complete that work and validate it against the task's acceptance criteria.
3. Measure the selector again on the changed state; verify that its result reflects the current unresolved work.
4. Use those timings to improve the selector while continuing the next useful item. Adopt validated tool changes between iterations.

```text
main work:  time next(S0) -> complete and validate A -> S1
            time next(S1) -> complete and validate B -> S2 -> ...
optimizer:  inspect timings -> improve candidate -> validate -> adopt between calls
```

`S0`, `S1` and `S2` represent successive work states. Optimization can run alongside the main work in an isolated candidate checkout, or be interleaved at bounded checkpoints by one agent. Keep the authoritative source and cache protected from concurrent candidate mutation. Revalidate a selection if its source state changes before execution. Parallel work should help accepted progress; measure its resource and coordination overhead too.

Record the selector revision, source state, workload counts, selected item, elapsed time and accepted work for each iteration. The live sequence shows whether the tool stays fast and useful as the task advances. Controlled measurements on the same saved state can diagnose a bottleneck or attribute a speedup. They complement this sequence; repeated unchanged queries alone do not establish progress or fresh selection after edits.

A cached or constant answer can be very fast and still be wrong. Qualify runtime across real accepted changes, with current results checked against the full reference. Completed work must stop being selected when it is resolved; new work and earlier regressions must remain discoverable. An item can legitimately remain selected if validation failed. Require a correct answer for the current state rather than merely a different answer on every call.

Use scheduled full-comparison checkpoints to qualify candidates. Keep normal iteration checks focused on affected work, so correctness validation does not silently reintroduce the expensive whole-matrix loop.

## Bound optimization while advancing dependent work

Dispatch joint optimization with two concrete endpoints: a measured performance target and an absolute cutoff time. Name the dependent task and keep advancing it with the best validated implementation throughout the allowance. Reaching either endpoint ends additional optimization effort; the useful-work loop continues toward its own completion criteria.

Record these terms together in the [task or decision record](plan-records.md):

| Term | Concrete example |
| --- | --- |
| Performance target | Each of 3 successive selector calls, interleaved with real accepted source changes, completes within one second. Measure fresh process launch through correct output and successful exit using the same validated tool revision, with current results matching the full reference. Record input, edit, workload counts and cache state. |
| Optimization allowance | Spend at most 30 minutes investigating and validating improvements. This is an illustrative scheduling choice, not a default budget for every task. |
| Canonical cutoff | Convert the allowance once, before dispatch, into an ISO 8601 UTC end DateTime. Retain that exact value across handoffs and resumes. |
| Dependent task and fallback | Continue source conversion during optimization and afterward using the fastest validated selector, even if its speed target was missed. Keep the correct baseline available until a candidate passes validation. |

Use a persisted UTC timestamp so any agent can compare the cutoff with `Get-Date`. This setup block writes a local scheduling record once; `New-Item` without `-Force` rejects an existing record instead of extending the allowance:

```powershell
# Once, before dispatch: canonicalize the 30-minute allowance.
$optimizeUntilUtc = (Get-Date).ToUniversalTime().AddMinutes(30)
$recordedCutoff = $optimizeUntilUtc.ToString(
  'o', [Globalization.CultureInfo]::InvariantCulture
)
New-Item -ItemType File -Path ./optimization-cutoff.utc.txt `
  -Value $recordedCutoff -ErrorAction Stop | Out-Null
```

Check the saved value before each experiment and when resuming. Do not recompute it from the new start time:

```powershell
$optimizeUntilUtc = [DateTimeOffset]::Parse(
  (Get-Content -LiteralPath ./optimization-cutoff.utc.txt -Raw -ErrorAction Stop).Trim(),
  [Globalization.CultureInfo]::InvariantCulture
)
$optimizationAllowanceExpired = (Get-Date).ToUniversalTime() -ge $optimizeUntilUtc.UtcDateTime
$optimizationAllowanceExpired
```

`True` means stop exploring performance changes after safely winding down the current experiment; continue the dependent task that has been progressing alongside it. Meeting the speed target earlier also ends optimization effort. A missing or invalid record does not grant a fresh allowance: report it and keep doing useful work with the validated baseline while resolving the scheduling record.

Keep experiment durations within the remaining allowance. Use a supported timeout or cooperative cancellation, and avoid starting work that cannot safely finish within it. A blocked operation may need a bounded wind-down; record any actual delay rather than moving the cutoff. Wall-clock UTC makes the shared deadline inspectable; retain monotonic `Instant` measurements for command performance.

At the cutoff or achieved target, retain the measurements, selected implementation, achieved speed and accepted task progress. Record whether the speed target was met separately from the dependent task's completion. If correctness or another hard prerequisite prevents useful work, record that concrete blocker. Further optimization needs an explicit scheduling decision and a newly recorded cutoff; a missed speed target must not silently turn a supporting improvement into an indefinite project.

## Understand the tool chain

| Piece | Responsibility | What to inspect |
| --- | --- | --- |
| `std::time::Instant` | Measures elapsed wall time within one process with a monotonic clock. | User-facing boundaries, phase completion and expected runtime checks. |
| `tracing` | Emits structured events and named spans around the work. | Stable phase names, counts, cache hits and selected backend; omit large or sensitive payloads. |
| `tracing-subscriber` | Collects, filters and formats that instrumentation through subscriber layers. | The template's stderr, NDJSON and optional Tracy configuration. |
| `--log-file <path>` | Selects the template's NDJSON file layer. The local update enables span-close events with busy/idle durations. | Individual JSON lines, explicit elapsed fields and correlated phase names. No Tracy build is required. |
| `tracing-tracy` and its Tracy client | Bridge Rust spans into the external profiler when the `tracy` feature is enabled. | CPU zones and their ordering; collection has its own filter. |
| `tracy-capture.exe` | Connects to a running instrumented program and saves a `.tracy` capture. | Capture connection and bounded shutdown status, rather than assuming a file means capture succeeded. |
| `tracy-profiler.exe` | Provides Tracy's interactive viewer for captures and detailed timelines. | Nesting, overlap, threads and periods spent waiting. |
| `teamy-profiler` | Builds/runs a Cargo target under capture, records an experiment manifest and summarizes CPU spans. | `run cargo` for repeatability; `tracy csv export` for an existing capture. It does not currently parse NDJSON. |

The path is `tracing → tracing-subscriber → stderr / NDJSON / tracing-tracy → Tracy client → tracy-capture → .tracy → tracy-profiler or teamy-profiler`. The first two destinations also work without Tracy. The capture harness coordinates the build, target and capture process; the summary command reads a capture that already exists.

Grounding: the template's [published subscriber setup](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs), [tracing-tracy](https://docs.rs/tracing-tracy/latest/tracing_tracy/) and [Tracy's upstream tools](https://github.com/wolfpld/tracy). The span-close file update and runtime warning helper described here are local template additions reviewed on 4 October 2026, rather than features of that published snapshot.

## Collect timing logs before a full capture

From a template checkout containing the local timing update, build first and then run the executable. This measures command execution separately from Cargo's build work:

```powershell
cargo build --release
./target/release/teamy-rust-cli.exe --log-filter info `
  --log-file ./profiles/home-show.ndjson --output-format json home show

Get-Content ./profiles/home-show.ndjson |
  ForEach-Object { $_ | ConvertFrom-Json } |
  Where-Object { $_.fields.message -eq 'close' } |
  Select-Object timestamp, @{n='span';e={$_.span.name}}, `
    @{n='busy';e={$_.fields.'time.busy'}}, `
    @{n='idle';e={$_.fields.'time.idle'}}
```

An installed CLI can be invoked directly with the same flags. Use its crate's target name in a narrower filter, for example `warn,teamy_rust_cli=debug`. `--debug` and an explicit filter conflict in the template. A `warn` filter excludes ordinary info/debug spans, so a file can contain warnings without a complete phase breakdown.

The template creates or truncates the named file; select a new filename for each retained experiment. Other programs can have different contracts: teamy-llm currently appends. NDJSON diagnostics remain separate from JSON results on stdout. See [logging destinations](logging.md#terminal-logs-ndjson-and-tracy) for filtering and file behavior.

Span-close timing needs both explicit `FmtSpan::CLOSE` configuration and timing enabled for that formatter. [The subscriber reference](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html#method.with_span_events) defines busy time as time entered and idle time as time alive outside an entry. These are elapsed intervals, not hardware CPU-cycle measurements. Nested or concurrent intervals overlap. With async work, instrument futures; keeping an entered guard across `await` can assign unrelated work to the span. Use explicit `Instant` fields when the desired completion boundary differs from the span's lifetime.

## Make slow commands warn during normal use

Declare a reasonable duration where the command's behavior is defined. Use `Instant::now()` at the boundary, compare `elapsed()` at meaningful checkpoints and check again before returning. Keep this warning independent of a full profiling build. Its purpose is to reveal suspicious delay while retaining the normal result, rather than turning a performance expectation into a cancellation deadline.

The local template supplies `CommandTimer` in `src/command_timing.rs`. It stores one start time, an expected `Duration` and whether it has warned. `checkpoint(phase)` emits one `warn!` after the expectation is exceeded; `Drop` checks at the final boundary, including error returns and cooperative cancellation. It creates no timer thread and adds no dependency. Adopt it in a new command with its own workload-specific expectation:

```rust,ignore
use std::time::Duration;
use teamy_rust_cli::command_timing::CommandTimer;

let mut runtime = CommandTimer::new("source oracle next", Duration::from_secs(1));
// Perform discovery using the existing command's inputs and cancellation token.
runtime.checkpoint("discovery complete");
// Select and render the typed result; the guard checks again when its scope ends.
```

This sketch shows the integration pattern; it does not assert that SFM already has the helper. In a generated project, use its own crate name. The warning carries `command`, `phase`, `elapsed_ms` and `expected_ms`. For example, an 80-second selector with a one-second expectation should report the overrun when control reaches a checkpoint or returns. Repeated checkpoints suppress duplicate warnings for that invocation.

The template sets one-second expectations for `home show` and `cache show`, and three-second expectations for `home open` and `cache open`, at the dispatch/output boundary. These are initial design expectations, not benchmark claims. The clock begins after argument parsing and logging setup and includes runtime construction, dispatch, output writes and the final cancellation check. It does not include OS launch, parsing or logging initialization. Size-dependent scaffolding and the cleanup placeholder have no assigned budget. New commands must choose their own boundary and expectation; a server or interactive session needs startup or per-request checks rather than a budget for its entire lifetime.

Checkpoints cannot report while a synchronous operation is stuck, and `Drop` cannot run after process termination or abort. Add a checkpoint after each meaningful chunk of a long loop; do not add polling threads solely for this policy. Warnings follow the chosen tracing filter, so `--log-filter error` can suppress them. A warning is evidence to investigate, not proof of failure or permission to skip correctness checks. Review the expectation when workload size legitimately changes rather than repeatedly raising it to hide regressions.

Tests cover the elapsed-time threshold, duplicate suppression and final checks without sleeping. The CLI's real log-file regression also checks that command span durations reach NDJSON while the typed stdout result remains separate. Run the template's [quality gate](code-standards.md#run-the-policy-checks-explicitly) after adopting or modifying this pattern.

## Try mimalloc for allocating hot paths

[Microsoft's application guideline M-MIMALLOC-APPS](https://microsoft.github.io/rust-guidelines/guidelines/apps/index.html#M-MIMALLOC-APPS) recommends setting mimalloc as an application's global allocator. It reports benchmark improvements of up to 25% along allocating hot paths. That is Microsoft's observed result, not a measured improvement for SFM or a guarantee for every command.

`teamy-mft` already follows this pattern. In the public revision [`42e6e21`](https://github.com/TeamDman/teamy-mft/tree/42e6e21bd0a579955aef57282730394338f6bf9f), its [Cargo manifest](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/Cargo.toml) declares `mimalloc = "0.1.48"`; its [lockfile](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/Cargo.lock) resolves `0.1.52`. The manifest is a compatible-version range, not an exact pin. Record the candidate's actual resolved version and features when comparing results.

Its [executable entry point](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/main.rs) installs the allocator before delegating to `teamy_mft::main()`. The declaration has no platform or feature guard:

```rust,ignore
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

Place this selection once in the application's executable, preserving its existing entry point. Check for an existing global allocator before adding another. Reusable libraries leave that choice to their application. The [Rust wrapper](https://docs.rs/mimalloc/0.1.52/mimalloc/) supplies `MiMalloc`; adding the dependency alone does not select it. This changes Rust's global heap allocation, including collections and strings. It does not replace CUDA/device allocation or a native library's independent allocator.

Use this as a small, measured experiment when parsing, rendering, cloning or collection construction spends time allocating:

1. Preserve a correct release-build baseline and record its profile, features and allocator. Add the dependency and executable declaration to an isolated candidate.
2. Continue the [measure selector → do useful work → measure changed state loop](#optimize-while-advancing-useful-work). Record accepted work and latency together. Use paired runs on the same saved state to isolate the allocator's effect.
3. Compare command wall time, allocation-heavy phase timings and memory use, keeping inputs and build settings matched. Check output parity and the existing correctness gates before adopting the candidate.
4. Retain the improvement only if the evidence supports it. Restore the baseline if it regresses. Fit the trial within the [existing optimization cutoff](#bound-optimization-while-advancing-dependent-work) while continuing the main task.

An allocator change can reduce allocation overhead; it does not eliminate repeated full scans, redundant rendering or stale-cache errors. Keep algorithmic improvements and allocation reuse in the same investigation. MFT establishes prior adoption, rather than a measured allocator speedup for its workload or SFM's selector.

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

For the local resident CLI, use the [interactive timing fields](interactive-inference.md#run-repeatable-sessions-without-a-terminal) to distinguish the parent's submission-to-reply clock from the worker's job duration. [Provider changes and Julia decision cancellation](interactive-inference.md#separate-model-residency-from-conversation-history) can discard residency. Explicit managed Qwen prompt and decision mode changes retain their shared runtime. Record which path ran before interpreting loading costs.

## Separate process startup from model residency

Choose the boundary that matches the intended experience. A fresh client can be fast while the process serving it has already spent time loading the model:

| Workload | Where loading occurs |
| --- | --- |
| Fresh `prompt` without `--daemon` | The client constructs its own runtime before generating text. |
| Repeated `benchmark` or interactive requests | The first request loads; later matching requests reuse the runtime, including managed Qwen prompt and decision mode changes. |
| Fresh `prompt --daemon` against a prewarmed service | `serve --model` loads and generates a warmup token before starting its listener; the client connects to that existing runtime. |

The [resident-service example](resident-inference.md) uses an explicit prewarm model and local IPC. The newer daemon interface also accepts `decide` and `interactive` clients. Direct clients and the newer typed interactive daemon relay retain request diagnostics as `daemon_*` fields. Local and native child stderr still use formatted `worker_log` text. See [log forwarding](logging.md#forward-one-requests-diagnostics) before choosing a capture format. Keep generation context settings consistent when comparing measurements. The current Qwen scheduler retains the same model weights when mutable cache capacity changes; graph rebuilding and retained-history replay can still add time. The current Orca initial capacity is 8,192 tokens; record an explicit `--context-tokens` value when comparing builds. The historical captures below used 1,024. Julia's encoding budget is a separate setting.

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

For a slow burndown or next-item command, measure the [complete migration iteration](migration-workflows.md#measure-the-complete-iteration). SFM's reported 80-second selector call illustrates how repeated validation and coordination can dominate routine transformations. Use dependency-aware caching with a full-comparison reference, and measure accepted work after repairs.

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

The local service now [overlaps CPU preflight and tokenizer validation](local-text-generation.md#overlap-cpu-validation-before-device-construction). `cpu_preflight.rs` starts a named scoped worker for GGUF verification and metadata parsing, joins it after tokenizer validation and permits GPU construction only after both succeed. Structural inspection is explicit and is not published into the validated-artifact cache prematurely. The Qwen owner checks complete formatted input and output budgets before generation; a cold runtime may already have been constructed. Requests for the same resident model reuse its verified weights while mutable cache capacity can change separately.

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

From the reviewed template checkout, plan a capture first and then run the same command without `--dry-run`:

```powershell
teamy-profiler run cargo --project . --bin teamy-rust-cli `
  --profile release --feature tracy --artifact-dir ./profiles `
  --dry-run -- --log-filter info home show

teamy-profiler run cargo --project . --bin teamy-rust-cli `
  --profile release --feature tracy --artifact-dir ./profiles `
  -- --log-filter info home show
```

The real run requires `tracy-capture.exe` on PATH and a target supporting the selected feature. The harness discovers `--log-file` through target help. Keep the `.profile-run.json` manifest, `.tracy` capture, target logs and top-span CSV together; the manifest distinguishes build and orchestration time from target execution. For interactive or long-running targets, choose a bounded representative session. Open the resulting capture in `tracy-profiler` for timeline inspection, or use the CPU summary below. Confirm improvements with the normal release build too, because instrumentation can change runtime.

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

The command requires an already recorded native Julia artifact location; [explicit overrides and the text input contract](finite-choice-models.md#score-text-choices-from-the-terminal) are unchanged. The CLI appends to `--log-file`, so use a new filename for each experiment. Prompts need not be logged to measure phase durations. Direct `decide` preserves structured span fields. The earlier interactive relay retained all child diagnostics as formatted `worker_log` text; the newer daemon relay preserves typed source fields, while local child diagnostics still use that text path. Check the matching executable before interpreting a capture.

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

## Remove unused decision scoring work

Start with the result the caller actually needs. The [Qwen finite-choice adapter](finite-choice-models.md#reuse-qwen-generation-weights-for-decisions) consumes 2 to 20 label logits. The inspected local implementation computes the complete 248,320-entry vocabulary projection for the last token of each prefill chunk. It reads that row back and selects the labels on the host; earlier chunk results are discarded.

Inspect `vendor/makepad-ai-llm/src/runtime.rs` for the output-weight projection, `vendor/makepad-ai-llm/src/session.rs` for logits readback and `crates/teamy_llm_makepad_gguf/src/scheduling.rs` for label selection. These are paths in the matching local `teamy-llm-service` checkout described by the [Qwen implementation record](finite-choice-models.md#reuse-qwen-generation-weights-for-decisions), not features reproduced by its older published base commit.

Two proposed changes target work that the decision result discards:

- keep the required hidden-state and cache updates for intermediate prefill chunks, but omit their output projection and logits readback
- project the final normalized hidden state against only the requested label rows, instead of computing scores for the whole vocabulary

Row selection preserves the mathematical scoring operation without training a new head. Restricted-label softmax does not need the omitted vocabulary scores. Quantized packing, tensor shapes and kernel selection can still change numerical results. Keep generation's full-vocabulary path intact and validate the restricted decision path against it, including choice order, logits, probabilities and near ties. Recheck image choices, mixed prompt/decision requests and cancellation. Skipping readback alone does not establish that the projection was skipped.

Profile the projection and readback first. Compare complete decision latency, memory and correctness using the same inputs and cache state. Do not infer a speedup from the reduction in output rows alone; backbone work can still dominate. Follow the [bounded optimization loop](#bound-optimization-while-advancing-dependent-work).

Reuse is another way to avoid work. [Clef's multiple-question design](finite-choice-models.md#reuse-one-input-across-several-questions) processes one input for several decisions in one backbone pass. Treat that as a separate model and contract change; do not attribute its quality or performance to label-row selection in our existing adapter.

## GPU submission is not completion

A CPU span around a GPU call may measure enqueueing, allocation, compilation or a wait. It does not establish device execution time. Use backend events or a documented synchronization boundary when measuring completed work; record which streams and operations the boundary covers. Synchronizing every operator can remove overlap and distort the workload. PyTorch's [CUDA timing guidance](https://docs.pytorch.org/docs/2.11/notes/cuda.html#asynchronous-execution) explains the same distinction for the independent reference.

Keep a minimally instrumented performance run separate from a diagnostic run that copies intermediate tensors to the CPU. Retain correctness evidence, repeat count, warmup policy and memory measurements with both. Use a capture to identify a candidate change, then repeat the same workload and parity gate after making that change.

## Find repeated setup inside a resident request

The historical unpublished [text-generation measurement](local-text-generation.md#local-release-measurement-tokenizer-reuse), using 1,024-token capacity before Windows mapping, provides a concrete example. A loaded GPU model still had roughly 822–843 ms to its first token event on two repeated requests. Phase logs showed that each request rebuilt the tokenizer. Keeping the unchanged tokenizer in the service reduced its later `llm_tokenize` spans to 405 and 423 microseconds; first-token event times became roughly 187–228 ms for the same prompt and output bound. The independent publisher prompt/token checks remained in place.

Cold first-token latency stayed near 22.8 seconds. Weight verification and session preparation still dominated the recorded loading phases. The change improved repeated host setup; it did not demonstrate faster GPU kernels or remove cold loading. The linked chapter retains executable identities, raw timing samples, instrumentation and qualification limits. Use this pattern to investigate repeated setup before assuming the model's numerical work explains the entire delay.

The newer local startup change addresses a different repetition: `inspect_model_dir` previously parsed the tokenizer and discarded it before `TokenizerCache::get` parsed it again for generation. The service now validates through that cache and retains the exact parsed `Arc<Tokenizer>` for encoding. Its fresh Orca path overlaps this validation with CPU model preflight; other validated inspection uses the callback API. The standalone inspector still performs its full validation. This removes one parse while preserving encoding options, special-token IDs, malformed-replacement rejection and explicit invalidation. The tokenizer's file-length/mtime freshness hints remain distinct from the GGUF weight verifier's every-byte identity checks. Measure the retained `llm_tokenizer_load` and the later cache hit during `llm_tokenize`; the historical tokenizer table predates these additional changes.

## Measure continuous batching and suspension

The shared-weight Qwen extension has local mixed-request and memory-pressure qualification on 5 October 2026. Follow [its residency and memory contract](resident-inference.md#share-weights-and-schedule-continuous-work) before interpreting a speedup. The historical single-request captures above retain their own builds and cannot qualify batched decoding, replay or memory-pressure recovery.

Start with a serial daemon baseline using `serve --max-concurrent 1 --batch-wait-ms 0`. Repeat the same workload with 2 active slots, then compare an optional 250 ms idle accumulation window. Keep model bytes, executable, context settings, input text, choice order, image shapes and explicit output limits identical. Record startup and vision preparation separately from resident request timing.

Omitted `--max-new-tokens` now means unlimited output. Set a finite length in controlled throughput comparisons. Otherwise a change in natural response length can dominate the result. A `benchmark --repeat` loop submits serial requests through one service instance; it does not establish concurrent daemon batching.

Measure a mixed workload with 2 prompt clients and 2 decision clients arriving together. Retain each request's arrival, first-token or decision-completion time, final completion, exit status and output. Also record total completed requests and generated tokens over the shared wall-clock interval. Do not add the requests' overlapping durations and call that elapsed server time.

Inspect actual decode batch width and steps with more than one ready lane. Connected clients alone do not prove shared numerical execution. A decision may finish during prefill while prompts continue decoding. Report the distinction between serial chunked prefill and joint decode, rather than calling every operation batched inference.

Correctness checks must match the path being claimed:

- compare complete token IDs and text on fixtures that reliably exercise independent state and replay
- compare Qwen label logits, ordered probabilities and chosen IDs separately, retaining near-tie and kernel-variation evidence
- distinguish independent labels and recurrent state by interleaving different requests, rather than only repeating one input
- verify that suspension and replay neither duplicate streamed output nor lose the pending decode token
- cancel queued, active and suspended requests, then require a following request to succeed

Keep Julia's independent numerical reference gate separate. It cannot validate Qwen's restricted-label scoring, image embeddings or recurrent batching. For image decisions, retain input and option ordering and require image-position accounting to include every occurrence.

Record memory snapshots before loading, after weights, after vision preparation and at the workload's highest observed cache use. Include our exact allocation accounting and device-wide total and free readings. Polling can change timing, so run detailed memory diagnostics separately from minimally instrumented throughput comparisons. A sampled peak is a lower bound on the actual peak.

Test cache growth and forced concurrency reduction with small initial capacities before using large workloads. Retain low-memory, suspension, resume and context-discard warnings. Compare `--strict` failure with best-effort continuation and report discarded positions. Output parity after context discard is not an expected invariant: removing old input deliberately changes what the model sees.

Only begin the optimization pass after the serial and concurrent correctness checks pass. Profile a named bottleneck, change it, repeat that gate and then rerun the same timing experiment. Report every sample, the first-use state and any failures. A wider batch or fewer allocations is a mechanism to investigate, not evidence of improved end-to-end latency.

## Check state and numerics separately

Batch width can change the arithmetic kernel without changing model weights. The local Makepad CUDA path selects `FlashVec` for one query column and `FlashMma` for multiple columns. Tensor-core input conversion and reduction ordering differ. A fixed seed and greedy sampling therefore do not promise identical text when logits are nearly tied.

The local probe used executable SHA-256 `8528bd8356f99795584918a6bf6318507ce1460c6f5f5dac6c86186b22ae6f7a` and a 1,024-position cache with no rebuilds. A prose answer first diverged at generated-token index 7, before suspension or replay:

| Execution | Logit for `recognized` | Logit for `officially` | Selected token and margin |
| --- | ---: | ---: | --- |
| Two-lane decode | 24.328907 | 24.278217 | `recognized`, 0.050690 |
| Single-lane decode | 24.285820 | 24.293238 | `officially`, 0.007418 |

Both executions shared the same prefix through index 6. Source inspection found no cross-lane mask or stride alias in the inspected attention path. The observed rank reversal is consistent with the differing arithmetic kernels; it does not independently prove all their numerical operations correct. Keep this failed exact-text comparison alongside successful state checks.

Two different 512-token counting fixtures matched all token IDs and text after streamed suspension and replay. They test state isolation and output delivery for those fixtures. They do not prove that arbitrary prose must match, or replace an independent tensor reference. Set numerical tolerances before a comparison and investigate deviations; do not loosen them afterward to accept a candidate.

For targeted diagnosis, the local adapter exposes early top-two logits through tracing:

```powershell
teamy-llm prompt "List facts about the Solar System." --daemon `
  --model qwen3.8-27b-uncensored-q4-k-m --max-new-tokens 32 `
  --log-filter 'warn,teamy_llm_makepad_gguf::logits=trace' `
  --log-file ./logits.ndjson
```

The trace records candidate position, selected token, top-two token IDs, logits and margin for the first 32 candidates. Through IPC, source fields appear in `daemon_fields`, which the NDJSON encodes as a JSON string. Parse that string before filtering an exact position; a text search for `7` can accidentally match decimal values from other positions. The tracing run is diagnostic evidence, not a minimally instrumented latency sample.

Validate the receipt parser itself. Prompt JSON stores text and token IDs beneath `result` as `generated-text` and `generated-token-ids`. Comparing two nonexistent `generated_text` fields can turn missing values into an apparent pass. Require nonempty expected fields and explicit token counts before checking parity.

## Local resident batching baseline

The 5 October baseline used executable SHA-256 `13a15cb6391c601b459f3b2ee8c075e5f0ab031c4153a3e662b18d89a4f820be`, the pinned Orca Q4_K_M artifact and an RTX 4090. Its source is an uncommitted extension over `6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a`. Each server started with 2,048 context positions, zero idle batch delay and the established bounded prefill implementation. Startup and prewarming were outside resident-client timing.

| Measurement | One active slot | Two active slots |
| --- | --- | --- |
| Fresh-client warm decision completion, 3 samples | 533.97, 522.36, 515.35 ms | 538.35, 517.18, 520.49 ms |
| Client first-token event, 3 eight-token prompts | 252.26, 218.66, 190.11 ms | 238.20, 206.37, 187.51 ms |
| Two 256-token counting prompts and two decisions, shared wall time | 12,486.47 ms | 8,258.18 ms |

The mixed workload completed about 1.51 times faster with 2 active slots. That is one workload sample per setting, not a p95 result or a general throughput guarantee. Both counting outputs matched their serial controls. Decisions can wait behind the longer requests when all slots are occupied; their idle half-second timings do not predict that queued latency.

Decision completion uses an external fresh-process clock and includes CLI startup, IPC and output. The first-token event uses the client's internal submission clock, excludes CLI startup and does not measure first stdout: these prompts requested JSON, written only after completion. Detailed diagnostic captures were separate from minimally instrumented mixed timing. Device memory was sampled before and after; those samples do not establish a peak.

The bounded optimization pass keeps the qualified 32-position prefill default. A 128-position prototype, executable SHA-256 `e9afc7bf847907ed661186263eca43aff657fcff2cd0e09c1e162e96687fc181`, reduced one-slot decision completion to 302.56, 284.49 and 332.62 ms, and two-slot completion to 315.20, 280.07 and 285.84 ms. Mixed wall times were 12,160.87 ms and 7,780.81 ms. Counting token IDs and selected choices stayed unchanged for these fixtures, but scores were not bit-identical: the writing-programs label logit moved from 24.44015694 to 24.37754440, and probability from 0.989553292 to 0.988886248.

The final executable exposes this tuning as `serve --prefill-tokens 128`. It separately passed [mixed generation, image scoring, cache replay and cancellation checks](resident-inference.md#qualified-final-local-build). One later healthy decision completed in 323.97 ms. That sample belongs to final-build functional qualification, not a repeated matched benchmark. These records support an explicit tuning option for the tested workloads; they do not establish equivalent general accuracy, a p95 latency or a reason to silently replace the 32-position default.

Bound optimization with a concrete target and one persisted UTC cutoff. The local pass allowed 30 minutes after the functional baseline and stopped after the tuning target and functional checks were achieved. It retained the established default and exposed the measured alternative explicitly. Do not reset that cutoff for each experiment or keep repeating a completed experiment after its endpoint. [Joint optimization while completing work](migration-workflows.md) explains how to keep progress and performance measurement in the same loop.
