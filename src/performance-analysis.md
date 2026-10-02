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

## Add useful spans before detailed events

Create a coarse span around each meaningful phase. Record request IDs, shapes, counts and selected backend, rather than dumping prompts, tensors or filesystem contents. Use debug events for allocation, cache and lifecycle decisions; use trace events for detailed steps when they answer a question. Instrumentation itself can change allocation and timing.

The template accepts `tracing` instrumentation and installs an optional Tracy layer. Its [public logging setup at `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/src/logging_init.rs) gives Tracy an independent trace filter: making terminal logs quiet does not disable that collection. See [logging filters and destinations](logging.md#terminal-logs-ndjson-and-tracy).

`#[tracing::instrument(skip_all)]` avoids automatically recording arguments, but explicit fields still need review. For async work, do not retain a `span.enter()` guard across an `.await`; instrument the future or use the instrumentation attribute. See the [`instrument` API](https://docs.rs/tracing/0.1.44/tracing/attr.instrument.html) and [async span guidance](https://docs.rs/tracing/0.1.44/tracing/span/struct.Span.html#in-asynchronous-code).

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

The local unpublished [text-generation measurement](local-text-generation.md#local-release-measurement-tokenizer-reuse) provides a concrete example. A loaded GPU model still had roughly 822–843 ms to its first token event on two repeated requests. Phase logs showed that each request rebuilt the tokenizer. Keeping the unchanged tokenizer in the service reduced its later `llm_tokenize` spans to 405 and 423 microseconds; first-token event times became roughly 187–228 ms for the same prompt and output bound. The independent publisher prompt/token checks remained in place.

Cold first-token latency stayed near 22.8 seconds. Weight verification and session preparation still dominated the recorded loading phases. The change improved repeated host setup; it did not demonstrate faster GPU kernels or remove cold loading. The linked chapter retains executable identities, raw timing samples, instrumentation and qualification limits. Use this pattern to investigate repeated setup before assuming the model's numerical work explains the entire delay.
