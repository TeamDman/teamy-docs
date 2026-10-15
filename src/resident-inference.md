# Connect CLI clients to a resident service

Use `--daemon` to send a request to an already running `teamy-llm serve` process. It keeps prepared inference runtimes across separate CLI clients. Without that flag, inference stays local to the invoking CLI or its owned interactive worker.

The Windows local-IPC interface was qualified on an RTX 4090 on 4 October 2026, using release executable SHA-256 `9d271e5dbf55c80e08339eb7154ed6ab75c32cd642b9a20c4628efabf66e0fac`. The scope and limits appear below. Check the matching executable's help; earlier latency captures in [local text generation](local-text-generation.md) and [finite-choice models](finite-choice-models.md) retain their original build identities.

The local extension on 5 October 2026 adds shared-weight Qwen decision scoring and continuous scheduling. RTX 4090 checks cover mixed prompting and decisions, image choices, cache growth, streamed suspension and replay, strict rejection and queued and suspended cancellation. The bounded optimization pass is complete, with the established defaults retained and explicit prefill tuning available. The scope appears in [the qualification record](#local-cuda-qualification-on-5-october-2026). The earlier daemon record below describes the previous serialized implementation and separate Julia decisions.

## Start the server once

Start a local server in one terminal:

```powershell
teamy-llm serve --model qwen3.8-27b-uncensored-q4-k-m `
  --context-tokens 8192 --max-concurrent 2 --batch-wait-ms 0 `
  --prefill-tokens 32 --vram-reserve-mib 1024 --log-filter warn
```

The explicit generation model receives a one-token prewarm before the listener starts. Without that selection, the server starts lazily and the first request loads its model. All artifacts must already be prepared. Prompting and choosing do not download them.

Keep the server process running. On Windows its default endpoint is `local://\\.\pipe\teamy-llm-service`. A separate endpoint keeps a second service distinct:

```powershell
teamy-llm serve --address 'local://\\.\pipe\teamy-llm-review' `
  --model qwen3.8-27b-uncensored-q4-k-m --context-tokens 8192
```

`serve --address` names the listening endpoint. Client commands use `--daemon` to select one. The transport accepts local IPC, rejects TCP and remote Windows pipe servers, and does not start a server automatically. A failed connection fails the command instead of constructing a local inference runtime.

## Prompt, choose and interact from fresh clients

Run these from another terminal:

```powershell
teamy-llm prompt "Why is the sky blue?" --daemon `
  --model qwen3.8-27b-uncensored-q4-k-m --max-new-tokens 64

teamy-llm decide --prompt "A note about writing a Rust program" `
  --choice "Writing programs" --choice "Playing games" --daemon

teamy-llm interactive --daemon `
  --model qwen3.8-27b-uncensored-q4-k-m --max-new-tokens 64
```

The same [text-choice interface](finite-choice-models.md#score-text-choices-from-the-terminal), [stdin prompt](local-text-generation.md#read-a-prompt-from-stdin) and [Liquid image templates](image-prompts.md) retain their result contracts. Images are acquired by the client and sent as bounded encoded bytes. The server does not receive an image path to open.

Omitting a generation model asks the server to resolve its own configured default. An explicit `--model` selects the server's named inventory. `--model-dir` for prompt generation remains local-only. Server configuration and storage do not have to match the client's homes.

Julia remains a separate decision model with a separate native artifact-location record. The server resolves its own record when the client omits `--model-dir`. Explicit decision model and ONNX library paths are resolved to absolute paths by the client, then opened on the server. This is local IPC on the same machine; those paths do not transfer artifacts.

To enable decision mode in an interactive daemon session, select the provider and use the server's recorded native Julia location:

```powershell
teamy-llm interactive --daemon `
  --model qwen3.8-27b-uncensored-q4-k-m --max-new-tokens 64 `
  --decision-provider native --device cuda
```

The interface owns a model-free relay child that connects to the server. It keeps that relay across prompt and decision mode changes. Exiting closes the client connection and its owned relay, while the independently started server keeps running. Visible transcript history still does not become the next prompt's model input.

An explicit `--decision-model-dir ./models/julia` overrides that server-side location. ONNX still requires its explicit model directory and runtime library.

To use Qwen for both operations in the working implementation, select its decision provider:

```powershell
teamy-llm decide --provider qwen --model qwen3.8-27b-uncensored-q4-k-m `
  --prompt "A note about writing a Rust program" `
  --choice "Writing programs" --choice "Playing games" --daemon

teamy-llm interactive --daemon --model qwen3.8-27b-uncensored-q4-k-m `
  --decision-provider qwen
```

The server keeps one Qwen language-weight allocation and one lazily loaded vision tower. Prompt generation and [Qwen label scoring](finite-choice-models.md#reuse-qwen-generation-weights-for-decisions) enter that same GPU owner. They retain independent request state. Julia and ONNX remain separate providers.

## Share weights and schedule continuous work

`serve --max-concurrent` sets the maximum active Qwen requests, with 2 as the working default. Additional requests wait for memory and a free slot. The limit controls active request state, rather than duplicate weight copies. Reducing it to one provides a serial baseline.

The scheduling loop processes one bounded prefill chunk per round and rotates between unfinished inputs. It alternates that work with decoding so long inputs do not monopolize every round. Image embedding prefill also follows the request's independent positions. Prefill is not a fused batch across requests in this implementation.

`serve --prefill-tokens` sets that chunk size, from 1 to 256 positions, with a default of 32. This bounds one scheduling step; it does not cap the model context. Larger chunks can reduce overhead for short decisions, but delay other requests and change floating-point rounding. The measured 128-position setting is an explicit option. Compare [its timing and score variation](performance-analysis.md#local-resident-batching-baseline) before changing a workload's configuration.

A decode step feeds one pending token from each ready request into a shared matrix operation. Attention still reads that request's own KV cache. Recurrent state, attention masks, position counters and sampling state remain separate. New requests can join as others finish. This is continuous batching, rather than waiting for every request in a fixed batch to conclude.

A decision finishes after its final label logits are available and releases its slot. Two prompts and two decisions can therefore share weights while only the prompts continue decoding. Actual overlap depends on arrival times, prefill work and memory admission. A slow output consumer pauses its own lane; it should not hold up other ready lanes.

`--batch-wait-ms 250` adds an optional idle accumulation window for new work. The default is zero. It can increase the initial batch width at the cost of up to 250 ms of added initial latency. It is not a delay before each generated token. Continuous admission normally removes the need for client-side batch IDs.

The numerical operations are joint within one GPU step, rather than separate kernels racing over shared mutable state. A mutex around entire requests would retain residency while still serializing inference. Qualify the actual batch width and request-state isolation, not just the number of connected clients. Different batch widths can select different arithmetic kernels; [the numerical comparison](performance-analysis.md#check-state-and-numerics-separately) explains why arbitrary greedy text need not match bit for bit.

## Inspect memory before admitting more work

The working memory interface exposes a live server snapshot:

```powershell
teamy-llm memory --daemon --output-format json
```

The version 1 report separates weights, attention cache, recurrent state, mutable arenas, CUDA scratch and vision allocations. `resident_allocated_bytes` counts known runtime allocations, including vision. Do not add the arena's constituent caches or activation reserve to that total again. `cuda_total_bytes`, `cuda_free_bytes` and `cuda_used_bytes` describe the whole device, including other applications and driver overhead. Our allocation total and device-wide used bytes measure different things.

The newer local report adds a `_human` string beside each of its 13 `_bytes` fields. For example, `weights_bytes` remains an exact integer, while `weights_human` presents that value in binary units through `humansize::BINARY`. The same convention covers allocation components, reserves and device-wide readings. Human strings round values for display; use the byte fields for arithmetic, comparisons and allocation decisions. These presentation additions do not change memory accounting or qualify a new performance result.

A synthetic fragment shows the naming and units:

```json
{
  "cuda_total_bytes": 25769803776,
  "cuda_total_human": "24 GiB"
}
```

In the matching local checkout, `GgufMemoryReport` and `format_memory_bytes` live in `crates/teamy_llm_makepad_gguf/src/lib.rs`. Its owner's snapshot in `src/scheduling.rs` derives each presentation string from the same value it reports as bytes.

For each request, the report exposes its phase, input positions, generated tokens, retained positions, occupied cache positions, remaining allocated continuation capacity and replay count. It also reports active, queued and suspended counts, cache reconfigurations, actual decode batch widths and `prefill_batch_size`. Remaining allocated capacity is not a promise that all later growth will fit. A fresh local `memory` command does not load a model; use `--daemon` to inspect an existing owner.

`serve --vram-reserve-mib` requests headroom for other applications and allocation uncertainty. Its default is 1,024 MiB. The effective admission guard is at least 1 GiB; the report exposes both configured and effective reserve bytes. This is an allocation target, not a hard reservation of device memory. Temporary workspaces, graph rebuilding, fragmentation and changing device use can reduce observed free bytes below the target. Check allocation outcomes rather than consuming every reported free byte.

The pinned GGUF declares 262,144 context positions, 16 full-attention layers, 4 KV heads and 256-value key and value vectors. The current F16 KV layout requires:

```text
16 layers × 4 heads × (256 key + 256 value) × 2 bytes
= 65,536 bytes per retained position per cache slot
```

The same model uses 156,893,184 bytes, or 149.625 MiB, of F32 recurrent state per slot. Its shared read-only weight arena is 16,810,714,496 bytes, about 15.66 GiB. Add activation arenas, compiled-graph resources, shared scratch space, vision allocations and driver use before assessing fit.

| Retained capacity per slot | KV for one slot | KV for 2 slots |
| --- | ---: | ---: |
| 8,192 positions | 512 MiB | 1 GiB |
| 32,768 positions | 2 GiB | 4 GiB |
| 262,144 positions | 16 GiB | 32 GiB |

The full declared context cannot fit alongside these weights on a 24 GiB GPU with this layout. This does not establish the largest usable smaller context: vision, scratch arenas, graph rebuilding and other device use change the available budget. Use exact allocation receipts and live driver readings for the actual workload.

Caches reserve storage for allocated positions, including unfilled continuation space. Prefill uses formatted input positions; generated tokens then occupy further positions. Two concurrent requests need 2 caches and recurrent states. They do not divide one logical context window, but their combined allocations compete for the same VRAM. Serial execution can reuse one slot's memory.

## Suspend work when growing caches exhausts memory

The working owner grows mutable cache and activation storage while keeping weights resident. It drops affected compiled graphs and rebuilds request state by replaying retained history. For image requests, it retains host embeddings for replay rather than encoding the image again. It preserves the pending decode token and sampling state, so replay must not redeliver previously streamed text.

If growth cannot fit all active slots, the owner reduces concurrency and suspends newer requests. The oldest active request proceeds first. A suspended request resumes when another request releases enough capacity. This releases GPU state rather than merely stopping compute while leaving the same memory occupied. Replay can increase latency and produces suspension and resume diagnostics.

When a single remaining request cannot grow further, best-effort mode warns before discarding oldest context. It retains roughly 75% of the existing window, removes complete image spans and replays the remaining history. The output can continue but its answer may change because older input is no longer visible to the model. `--strict` fails instead of discarding context or exceeding its declared budget.

Retain warnings about low VRAM, reconfiguration, suspension, resume and discarded positions with the request identity. They explain pauses between tokens that would otherwise look like a stalled client. Cancellation must still remove queued or suspended jobs. GPU kernels remain cooperatively cancellable between operations, rather than preemptible in flight.

The local pressure check suspended a counting request after 221 generated tokens and resumed it after the older request finished. Both complete 512-token counting outputs matched their serial controls. A separate rollover check warned about discarded context and still answered its retained question. These checks qualify the exercised replay and recovery paths, not all prompts or memory layouts. See [the batching measurement method](performance-analysis.md#measure-continuous-batching-and-suspension).

## Local CUDA qualification on 5 October 2026

These Windows checks used the pinned Orca Q4_K_M artifact and an RTX 4090. Source was an uncommitted extension over `6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a`; the base commit alone cannot reproduce it. Preserve each executable's identity rather than treating successive local builds as one release:

- mixed generation, image choices and trained-context rejection: SHA-256 `6c253fdcac3f577597d96de5cbb7cf184f4b2aec1b0e24b4fc2ec4465e3869ed`
- the numerical kernel probe: SHA-256 `8528bd8356f99795584918a6bf6318507ce1460c6f5f5dac6c86186b22ae6f7a`
- streamed pressure, physical rejection, rollover, recovery and the performance baseline: SHA-256 `13a15cb6391c601b459f3b2ee8c075e5f0ab031c4153a3e662b18d89a4f820be`

| Exercised path | Observed result |
| --- | --- |
| Two 512-token prompts and two Qwen decisions | 509 joint decode steps reached width 2. Cache capacity grew from 256 to 512 to 1,024 positions. The 16,810,714,496-byte weight allocation stayed resident. |
| Image input and image choices | A red input matched the red choice over blue, with probability 0.99745975. Three 64 × 64 fixtures occupied 27 image positions in total. A separate blue-image prompt answered `Blue`. |
| Pressure after output had started | The second counting request suspended after 221 generated tokens, then resumed. Both 512-token outputs and token IDs matched serial controls without duplicate delivery. |
| Strict and best-effort budgets | Explicit 100,000-token and 300,000-token output budgets exceeded physical and trained-context limits respectively. Both strict requests failed before stdout. Best-effort rollover emitted a discard warning and continued. |
| Recovery and output termination | Queued cancellation recorded the queued phase, followed by a healthy surviving request. An omitted output bound reached the model's end token. |

Counting fixtures test independent state and replay delivery. They do not establish independent model mathematics or universal answer accuracy. A prose fixture diverged at a near-tied logit before any cache rebuild; retain that evidence and compare arithmetic separately. Small colour fixtures do not qualify general image understanding or every image resolution. Slow-consumer bounds and arbitrary cancellation timings need their own workload checks.

The later [performance baseline](performance-analysis.md#local-resident-batching-baseline) measures residency and mixed throughput separately. Each newer executable needs matching correctness evidence; these results do not automatically qualify subsequent optimization changes.

### Qualified final local build

The final local executable has SHA-256 `1aaf09648184674d20d1d69717258ce4a0dda828f96cf267791ba5db109f1c2b`. Its 286 focused tests passed, one host-dependent test remained ignored, and owned-package Clippy passed with warnings denied. Those checks are separate from the actual CUDA runs.

With explicit `--prefill-tokens 128`, two 512-token counting prompts and two decisions produced 510 joint decode steps at width 2. The cache grew from 256 to 512 to 1,024 positions while the same language weights stayed resident. Both counting outputs matched their serial token IDs and complete text. A three-image decision selected red over blue with probability 0.99708226, and the blue-image prompt answered `Blue`. Strict trained-context rejection, active cancellation and a following healthy decision passed.

A separate final-build check cancelled a request after it had streamed and entered the suspended phase. The older request finished its 512 tokens; a subsequent decision succeeded. The final snapshot contained no active, queued or suspended requests. A local [interactive prompt–decision–prompt script](interactive-inference.md#reuse-qwen-across-local-mode-changes) also retained one Qwen load.

After the image check, known resident allocations were 20,434,332,736 bytes. Device-wide CUDA used memory was 22,713,135,104 bytes, with 3,043,430,400 bytes free. This is one final snapshot, not a measured peak or a reservation guarantee. The [bounded optimization record](performance-analysis.md#local-resident-batching-baseline) separates prototype timing from this final-build qualification.

## Ground scheduling in reference implementations

[vLLM's scheduler at `e8a53a5`](https://github.com/vllm-project/vllm/blob/e8a53a55e6ffe92645d8183066aa72e228ba7181/vllm/v1/core/sched/scheduler.py#L592) schedules token budgets across running and waiting requests. Its [allocation-pressure path](https://github.com/vllm-project/vllm/blob/e8a53a55e6ffe92645d8183066aa72e228ba7181/vllm/v1/core/sched/scheduler.py#L752) preempts work when KV blocks cannot be allocated. [Preemption](https://github.com/vllm-project/vllm/blob/e8a53a55e6ffe92645d8183066aa72e228ba7181/vllm/v1/core/sched/scheduler.py#L1542) frees state, resets computed positions and returns the request to a waiting queue. This supplies inspectable prior art for replay and admission; our contiguous slot storage is not vLLM's paged cache.

[Ollama's scheduler at `42e911b`](https://github.com/ollama/ollama/blob/42e911bc3d05798cad729cb474bf62f378cb2e26/server/sched.go#L798) accounts for parallel slots by multiplying context capacity by parallelism. Its [architecture guard](https://github.com/ollama/ollama/blob/42e911bc3d05798cad729cb474bf62f378cb2e26/server/sched.go#L502) forces one parallel request for `qwen35`, `qwen35moe`, `qwen3next` and other listed families. This is a useful warning to validate hybrid recurrent state independently. It is not evidence that this model's batched execution already works in every implementation. Ollama also [clamps context to training metadata](https://github.com/ollama/ollama/blob/42e911bc3d05798cad729cb474bf62f378cb2e26/server/sched.go#L166); our warning and strict policy deliberately differs.

The new Teamy paths are local working changes over source commit `6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a`. Inspect `crates/teamy_llm_makepad_gguf/src/lib.rs`, `vendor/makepad-ai-llm/src/{session,lanes}.rs`, and the shared service routes in the matching checkout. The public service source linked below predates these additions.

## Make the optional endpoint unambiguous

The option has 3 states:

| Invocation | Inference location |
| --- | --- |
| No daemon option, or `--no-daemon` | Local CLI or its owned interactive worker |
| `--daemon` without a value | The default local endpoint |
| `--daemon 'local://\\.\pipe\teamy-llm-review'` | That explicit local endpoint |

Do not combine `--daemon` and `--no-daemon`. `--daemon` can consume the next non-option token as its endpoint. Put positional prompt text before it, or use `--` to end options:

```powershell
teamy-llm prompt "Why is the sky blue?" --daemon
teamy-llm prompt --daemon -- "Why is the sky blue?"
teamy-llm prompt "Why is the sky blue?" `
  --daemon 'local://\\.\pipe\teamy-llm-review'
```

This follows [Figue's typed command structure](cli-parsing.md). It avoids introducing a top-level positional shorthand that could reinterpret a mistyped subcommand.

## Send request logs back to the caller

Select diagnostics on the invoking client:

```powershell
teamy-llm prompt "Why is the sky blue?" --daemon `
  --log-filter 'warn,teamy_llm_service=debug' --log-file ./request.ndjson
```

The server captures only that request's tracing events using the client's source filter. Its own console and file layers keep their independent filters. A server started with `--log-filter warn` can therefore send this client's requested debug events without printing them on its own terminal.

Direct `prompt` and `decide` clients replay typed events at their original level through their tracing subscriber. Original timestamp, target, file, line, request identity, fields and span stack remain explicit `daemon_*` fields. Span-close events carry source busy and idle nanoseconds. Their log files record these fields as NDJSON.

The newer local human formatter presents those source records as `[daemon]` logs, with their original UTC timestamp, target, fields, location and span stack. Client records are labelled `[client]`. Span-close durations use readable units. It does not print the nested transport envelope on human stderr. The NDJSON capture keeps the same structured `daemon_*` fields for analysis; read [source-aware log presentation](logging.md#present-source-records-at-the-client) for the writer and filtering boundaries.

Interactive mode forwards the owned daemon event through a bounded typed worker reply. The parent validates the active worker action ID separately from the inner daemon request ID, then replays the original event through its subscriber. Parent NDJSON retains the same structured `daemon_*` metadata as direct clients. The relay child's local and native stderr still use formatted `worker_log` text. Human delivery follows the parent's [terminal ownership buffer](terminal-ownership.md); daemon records are not also printed by the child.

Capture has a 128-event queue and a 16 KiB aggregate text bound per event. It bounds fields and span depth, truncates oversized values and reports dropped diagnostics. Producers never wait for diagnostic transport credit. Result streaming and cancellation have separate channels, so a slow log consumer cannot hold up inference. Forwarding is bounded best-effort delivery, not a durable audit log or a global server-log subscription.

Ctrl+C and `--stop-after-duration` request cancellation of the current server call. They do not stop the server or preempt a running GPU kernel. `--stop-after` applies to client instrumentation, including the relay child's instrumentation in interactive mode. Forwarded source spans are metadata rather than entered client spans; it is not a server-side span selector.

Read [request-scoped log forwarding](logging.md#forward-one-requests-diagnostics) for the implementation and the differences from MFT's history-and-broadcast hub.

## Measure the work that residency removes

A prepared server reuses verified weights and tokenizers. The first Julia decision still initializes its separate provider; the first image request still loads the vision tower. Qwen decisions reuse the language weights. The scheduling change separates mutable context growth from those weights, while changing the model can still replace its runtime. GPU work remains subject to the server's queue and available memory.

Record initial server startup, first use of each provider or image shape, and later fresh-client calls separately. Include connection, transport and result-writing time when measuring the client's experience. Moving work into an earlier server startup changes the measurement boundary; it does not make that initial work disappear.

## Inspect the implementation

On 4 October 2026, 220 affected tests and the owned-package Clippy checks passed. Model-free capture tests covered request isolation, source debug with a warning-only console, dynamic span filters, nested worker dispatch, and bounded loss and truncation. Replay and worker-relay tests covered original-level preservation, invalid-wire rejection and source-specific filtering through the interactive parent.

Live Windows named-pipe checks with the release executable above covered:

- Separate prompt clients reused the Qwen CUDA runtime, including selection through the server's configured default.
- Separate native Julia CUDA clients reused their provider. A prompt–decision–prompt sequence retained the Qwen runtime.
- A mixed scripted interactive session used the daemon route. In-flight cancellation was followed by a successful request.
- A Liquid image prompt reached the vision runtime with its request tracing context, including GPU-owner diagnostics.
- A missing endpoint failed with a nonzero exit rather than starting local inference. The server drained on its intentional cancellation deadline and exited nonzero according to the cancellation convention.

These checks qualify the exercised routing, reuse and recovery paths. They do not establish a general latency benchmark: the warm prompt samples used differing texts, and first-load samples ran under sandbox cache-persistence restrictions. They do not requalify an earlier cold-start target or establish equivalent Unix behavior.

[MFT's daemon log layer](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/machine/daemon_log.rs) provides prior art for typed diagnostic transport and client replay. Its [owned wire events](https://github.com/TeamDman/teamy-mft/blob/42e6e21bd0a579955aef57282730394338f6bf9f/src/daemon/daemon_log_event.rs) preserve source metadata. Our capture instead creates a private bounded queue for one request and exposes no global history.

The [public service source](https://github.com/TeamDman/teamy-llm-service/tree/cc0950321a13cf6a8621c574d75cca664b150880) predates these local additions. In the matching checkout, inspect `crates/teamy_llm_service/src/vox_api.rs` for typed IPC and independent cancellation, `request_logs.rs` for span routing, and `crates/teamy_llm_core/src/remote_log.rs` for the owned wire contract. CLI selection and draining live in `crates/teamy_llm_cli/src/cli/{daemon_args,daemon_client,daemon_drain}.rs`; replay and output filtering live in `crates/teamy_llm_cli/src/{remote_logging,logging}.rs`.
