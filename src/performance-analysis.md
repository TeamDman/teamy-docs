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
| `gguf_verify_identity` | Our adapter reads the whole GGUF and computes SHA-256, comparing the named artifact with its frozen identity. |
| `gguf_load_model` | Makepad parses the model metadata. |
| `gguf_prepare_session` | The session constructs its vocabulary, plans buffers, obtains host weights, allocates device state, uploads weights and prepares execution. |
| `GGUF session preparation phase` debug events | Elapsed milliseconds between progress-stage transitions, including initial metadata cloning and validation. |

These boundaries come from `crates/teamy_llm_makepad_gguf/src/lib.rs` in the matching service checkout. The progress hook in `vendor/makepad-ai-llm/src/session.rs` distinguishes vocabulary, plan, mapping, device, cache, reserve, upload and compile stages. The adapter groups changing upload labels into one upload phase and changing compile labels into one compile phase. Progress fractions are approximate progress markers, not elapsed time or measured proportions of total cost. Use each event's `elapsed_ms` when attributing its CPU-side duration; the parent span already contains these intervals.

The whole-file hash is our identity check, not a Makepad prerequisite. It runs once per constructed runtime and stays enabled with the [local read-only Windows mapping](local-text-generation.md#understand-the-windows-mmap-message). Mapping changes the host-weight path; GPU uploads, buffer planning and first-use execution remain. Likewise, two reserve stages can prepare different token-count shapes rather than duplicate identical work. Keep the input and resulting plan equivalent when testing a planner optimization.

The [mapped-loading candidate capture](local-text-generation.md#local-candidate-measurement-fresh-clients-and-mapped-loading) demonstrates this attribution: its full-file hash recorded 2.094 seconds in read calls and 7.328 seconds in digest updates, while the upload phase recorded 6.766 seconds. These intervals support investigating digest and upload work separately; they do not establish storage bandwidth, device-copy time or GPU kernel time. The same build's short fresh-client requests reached visible text within one second only after separate service prewarming. The initial model load still exceeded that target.

Runtime phase names alone do not identify repeated source compilation. The pinned CUDA backend uses kernels compiled during the Rust build; a phase named `load llm compile` can include preparing a graph for execution. Establish the actual work in that phase before adding a compilation cache. Measure a change with full identity checks and parity intact before claiming an improvement in cold time to first visible text.

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
