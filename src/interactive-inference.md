# Prompt and choose repeatedly with a resident model

Use `teamy-llm interactive` when you want several requests against one loaded runtime. It combines a Ratatui interface with the same generation and finite-choice services used by batch commands. Generating text and scoring supplied choices remain distinct operations.

This chapter describes the installed and qualified local release on 2 October 2026. The service source changes have not yet been published. Use that release or the matching development build; another installation may not expose these commands. Model-free validation and numerical performance evidence remain separate.

## Select a model and persist a default

An explicit selection remains useful when reproducing a result:

```powershell
teamy-llm model list
teamy-llm interactive --model qwen3.8-27b-uncensored-q4-k-m `
  --no-thinking --max-new-tokens 64
```

The model must already be prepared. Starting the session does not acquire weights. `--model-dir ./models/orca` selects an existing prepared directory instead; it cannot be combined with `--model`.

Use the separate default commands to persist your usual named model:

```powershell
teamy-llm model default set qwen3.8-27b-uncensored-q4-k-m
teamy-llm model default show
teamy-llm model list --output-format text
teamy-llm model list --output-format json
```

`set` accepts a known, prepared managed model. It checks inventory metadata and required artifacts before replacing the versioned preference record. It does not load a numerical runtime. An invalid selection leaves the previous record intact. Preparation order does not replace an explicitly chosen default.

`show` reports the model, directory and selection origin. The text list marks the selected row with `(default)`; JSON carries `is-default` alongside the same inventory fields. Without an explicit preference, selection uses the first existing registered model; if none is registered, it falls back to the built-in Qwopus model. Omitting a model from `prompt` produces a warning and points to `model list`; the interactive header shows the effective selection.

An interactive session resolves an omitted selection once at startup and sends that explicit directory to its worker. Changing the preference from another process does not change the open session's model or displayed label. Start another session to adopt the new preference. Named or implicit-default Orca thinking mode defaults to 512 new tokens; an explicit directory needs an explicit output bound.

Configuration and artifact storage have independent `TEAMY_LLM_SERVICE_HOME_DIR` and `TEAMY_LLM_SERVICE_CACHE_DIR` overrides. See [start a Rust project](rust-cli-template.md) for our configuration and CLI conventions.

<details>
<summary>Run the development build with actual CUDA</summary>

Run this from the `teamy-llm-service` checkout. Its wrapper requires an installed CUDA toolkit and C++ build tools, and requires the actual Makepad CUDA backend:

```powershell
.\scripts\cargo-cuda.ps1 -CargoArguments @(
  'run', '--release', '--locked', '-p', 'teamy_llm_cli', '--',
  'interactive', '--model', 'qwen3.8-27b-uncensored-q4-k-m',
  '--no-thinking', '--max-new-tokens', '64'
)
```

This builds and launches the CLI. A bare Cargo invocation can use a stub backend if its build environment does not require CUDA. The wrapper scopes its environment changes to that invocation.

</details>

## Edit a request while the worker runs

The screen uses stderr and requires terminal stdin and stderr. Rendering, keyboard input and worker events run separately from blocking inference. The CLI rejects another submission while a request is active.

| Control | Behavior |
| --- | --- |
| Enter | Submit the current request. |
| Shift+Enter | Insert a newline when the terminal distinguishes this key combination. |
| Tab or Shift+Tab | Switch between prompt and configured decision modes while idle. |
| Arrows, Home and End | Edit the request; Ctrl+Home and Ctrl+End move to its beginning and end. |
| Ctrl+C or Esc | Request cancellation while busy; quit while idle. |
| Ctrl+Q | Quit and stop the owned worker. |

Cancellation remains cooperative. An executing GPU kernel cannot be preempted. The session waits for the active work to wind down before starting another request. See [cooperative cancellation](cancellation.md) and [async terminal work](async-terminal-ui.md).

The final Windows ConPTY check displayed Julia's selected `heal` choice using ONNX CUDA with explicitly permitted mixed placement. A Unicode prompt followed by Ctrl+C cancelled the request while keeping the interface open; Ctrl+Q then exited with status zero. The input console mode matched its original value, stdout contained zero bytes and the independent NDJSON capture retained 18 log records. This qualifies the observed input, output and controlled cleanup path in that terminal.

## Configure finite choices explicitly

Enable Julia decision mode with an explicit provider and artifact directory:

```powershell
teamy-llm interactive --model qwen3.8-27b-uncensored-q4-k-m `
  --max-new-tokens 64 `
  --decision-provider native --decision-model-dir ./models/julia `
  --device cuda --max-tokens 1024 --head-tokens 256
```

For ONNX, select `--decision-provider onnx` and supply `--runtime-library ./runtime/onnxruntime.dll`. Requested CUDA never silently falls back to CPU. `--allow-cpu-fallback` explicitly permits mixed ONNX graph placement; it is not a native-provider option.

Decision mode accepts the same version 1 request JSON as [score supplied choices](finite-choice-models.md). It returns ordered option scores and a selected ID. It does not execute the selected choice. Julia accepts 2 to 20 text options; images are rejected before tokenizer or numerical artifact reads.

## Separate model residency from conversation history

Repeated requests within one mode retain the worker and its numerical runtime. Prompt turns are independent: the visible transcript does not become the next model input. The OrcaRouter adapter still has a 1,024-token context for the formatted prompt plus requested output. Lower the output limit for longer inputs.

Cold loading verifies the frozen GGUF identity and prepares its GPU session. A resident request reuses that session and the unchanged tokenizer instead of repeating those cold steps. Reuse is scoped to the owned process; exiting the CLI ends it. See [local text generation](local-text-generation.md) for artifact identity checks and measured loading costs.

This does not fix Windows memory mapping. The pinned Makepad loader's non-Unix implementation reports mapping as unavailable and uses an owned weight arena. Retaining an already loaded session avoids another load within that mode; it does not remove first-load hashing, disk staging or device preparation. See [the Windows mmap explanation](local-text-generation.md#understand-the-windows-mmap-message).

Switching between prompt and decision modes replaces the owned worker process. Dropping a provider alone does not guarantee that Burn or CubeCL releases allocator-held GPU memory. The process boundary releases that mode's residency before loading the other model. Returning to a mode therefore includes another cold load. Work remains sequential.

Cancelling a Julia decision discards its numerical provider and cached cancellation token. A later decision constructs a fresh provider within the same worker. Successful decisions retain the provider; a cancelled decision therefore changes the next request's loading cost even when the mode stays the same.

## Keep diagnostics away from the screen

The parent owns Ratatui and the optional `--log-file ./captures/session.ndjson` destination. The child sends bounded, versioned protocol replies on stdout. Its stderr carries both tracing and native diagnostics, including Makepad messages that do not use our subscriber.

Decision paths cross that private JSON boundary as explicit UTF-8 strings, preserving their lexical spelling and rejecting NUL or non-UTF-8 values; the child reconstructs and validates its typed CLI options. Keep boundary types explicit and test their serialized shape and round trip: a reflected `PathBuf` inside CLI configuration is not automatically a usable JSON transport contract.

The parent drains that stderr pipe and records worker diagnostics through its own tracing subscriber. While the terminal lease is active, the human log layer buffers records for the log pane. The independent NDJSON layer continues writing. Human history is bounded; it is not a complete persistent capture.

Terminal restoration happens before releasing the output lease. Failed restoration poisons the human-output route and fails the command instead of resuming writes into a damaged screen. These boundaries follow our [terminal ownership and logs](terminal-ownership.md) prior art. See [logging with tracing](logging.md) for filters, file output and the `--log-level` alias.

## Run repeatable sessions without a terminal

`--script ./sessions/repeat.json` uses the same engine for a non-terminal session. Its input is a strict version 1 document with 1 to 1,000 actions, bounded to 2 MiB. Prompt and decision actions are separate typed variants. A decision action contains the same request JSON accepted by `decide`.

Save this input as `./sessions/repeat.json` to request a prompt, cancel another prompt and then score supplied choices:

```json
{
  "version": 1,
  "actions": [
    {"prompt": {"text": "Why is the sky blue?"}},
    {"prompt": {"text": "Tell me a long story.", "cancel-after-ms": 5}},
    {"decide": {
      "request-json": "{\"version\":1,\"input\":{\"text\":\"Choose a useful next step.\",\"visuals\":[]},\"options\":[{\"id\":\"measure\",\"content\":{\"text\":\"Measure the existing implementation.\",\"visuals\":[]}},{\"id\":\"guess\",\"content\":{\"text\":\"Guess where time is spent.\",\"visuals\":[]}}]}"
    }}
  ]
}
```

The outer action uses `prompt` or `decide` as its tag. `request-json` is a JSON string containing the finite-choice document, rather than another untyped object. Run it with both numerical modes explicitly configured:

```powershell
teamy-llm interactive --model qwen3.8-27b-uncensored-q4-k-m `
  --no-thinking --max-new-tokens 64 `
  --decision-provider native --decision-model-dir ./models/julia --device cuda `
  --script ./sessions/repeat.json
```

`cancel-after-ms` requests cooperative cancellation after submission. The model may complete before the cancellation is handled. This example deliberately changes modes and includes a cancellation request; use repeated prompt-only actions when measuring resident generation.

Each action produces one JSON record on stdout with its request ID, mode, timing, typed result or error, and cancellation status. The result distinguishes generated text from decision scores. Per-action failure is a shaped result so the session can continue; unrecoverable input, transport or shutdown errors propagate through `eyre` and exit nonzero. Global cancellation ends the command. Logs remain on stderr independently of structured stdout.

This illustrative prompt record shows the confirmed wire shape. Its text, IDs and timings are synthetic; they are not a captured model response or performance measurement:

```json
{
  "version": 1,
  "id": 1,
  "mode": "prompt",
  "elapsed-ms": 0.0,
  "worker-elapsed-ms": 0.0,
  "first-token-ms": 0.0,
  "result": {
    "prompt": {
      "result": {
        "rendered-prompt": "Synthetic formatted prompt",
        "generated-token-ids": [123],
        "generated-text": "Synthetic answer"
      }
    }
  },
  "error": null,
  "cancelled": false
}
```

Decision success uses `result.decide.result` with the existing typed `DecideOutput`. A cancelled action has `result: null`, a diagnostic in `error` and `cancelled: true`; it does not pretend to contain a complete answer. Switch on the result tag or cancellation/error fields instead of inventing a distinct process exit code for each action outcome.

Session timing uses these explicit boundaries:

| Field | Clock and included work |
| --- | --- |
| `elapsed-ms` | Parent duration from before submission to receiving the terminal reply, including any mode-change worker restart. |
| `first-token-ms` | Parent duration to receiving the first token event, including loading and transport; this event can contain empty text. |
| `worker-elapsed-ms` | Successful child-job duration, excluding the parent's mode-change restart and reply transport. Failures leave this field unknown. |

These are separate measurements, not durations to add together. The first action's parent clock starts after the initial worker handshake. A process-start measurement must also include that handshake and CLI setup. The TUI currently starts its live busy timer after submission and reports worker duration on completion; script timings are the clearer comparison interface.

For repeated generation measurements without a screen, `benchmark` also uses one service instance. Its typed reports default to text on a terminal and JSON when redirected; `--output-format json` selects JSON explicitly. Read [performance analysis](performance-analysis.md) before comparing cold loading, first-token time and completed inference.

## Observe cold loading and resident generation

Source revision `8cc6e279682ac615512060a43cc058391c05373f`, executable SHA-256 `45c5edefebd00f47461090437395fc1870212c334a3b6d3bb38a824aae71996f`, completed a 13-action mixed session on an RTX 4090. It began with three identical “Why is the sky blue?” requests and a 32-token output bound. One owned worker handled those first three prompts. Debug stderr capture and parent NDJSON logging were enabled; no Cargo build or other inference run overlapped the capture.

| Request | Parent time to first token event, ms | Parent time to completed reply, ms |
| --- | ---: | ---: |
| First request, cold model | 22,333.08 | 22,996.00 |
| Resident request 2 | 222.61 | 877.37 |
| Resident request 3 | 193.52 | 847.90 |
| Return to prompt after decision mode, cold model | 21,993.66 | 22,650.24 |

The generated token-ID arrays matched exactly across the initial three prompts and the final return to prompt mode. Weight identity verification, tokenizer loading and GPU session preparation each occurred once per prompt worker. Residency avoided repeating those loading steps within a mode; it did not remove the first load or add Windows memory mapping.

The decision portion used ONNX CUDA with explicitly permitted mixed graph placement. All six Julia reference cases preserved option order, selected IDs, question and encoding policy, and met the original score tolerances. The first decision loaded its provider; the following successful decisions reported `load-ms: 0`. Exact encoding was independently checked by the separate 24 batch cases across native CPU/CUDA and ONNX CPU/mixed CUDA. The resident session checked scores and fixed policy; these short fixtures do not establish 1,024-token performance.

A malformed decision returned a shaped error in 0.21 ms. A later decision requested cancellation after 1 ms and returned `cancelled: true` after 69.32 ms. The next valid decision reloaded its provider, completed in 2,959.38 ms and again passed the reference score check. The two mode changes replaced the owned worker twice; returning to prompting incurred the cold load shown in the table. This observed recovery does not make executing GPU kernels preemptible.

The initial sequence contains two resident timing samples, not a latency distribution. Timings use the parent's script clock after the initial worker handshake and include event transport. The first event can contain empty text; these are not pure decoder or first-visible-text timings. Matching prompt token IDs establish repeatability for this sample, rather than independent generation numerical parity or general answer quality. The final display-only release is identified below; it is distinct from this capture.

In an earlier prompt-only capture, executable SHA-256 `bd536fe08b2d00f25002dfeab41f7bfb74f1f485937195f04891871f8d03d485` returned a shaped cancelled prompt after 18.87 ms when cancellation was requested after 5 ms. Its following Unicode “café” prompt completed 32 generated tokens, with a first-token event at 250.64 ms and completed reply at 902.38 ms. That earlier recovery capture and the [tokenizer-cache experiment](local-text-generation.md#local-release-measurement-tokenizer-reuse) used separate executables and workloads.

## Inspect the implementation behind the interface

The final local source revision `14ce5f54b991711f03ec6239e410e5418dcb33f5` passed 62 CLI unit tests, 25 service unit tests, strict Clippy and 73 model-free subprocess checks. Its Windows executable's SHA-256 is `901540128871bd35cca6bb32dc8468932d6621b052e3a69e12589bd52125e68d`. The final changes affect three display files: interactive human logs omit ANSI styling and the chosen option remains visible in a two-line results pane. Normal CLI colour output is retained; the numerical engine is unchanged from the mixed-session capture.

The installed executable matches that qualified build's hash. Installing it retained the previously registered legacy model as the effective default; selecting Orca remains explicit unless you set a new preference.

The process checks cover command parsing, generated help, invalid-input failures, isolated model preferences and typed output. They do not establish GPU speed or answer quality. Hardware and transport changes require their own input, timing and runtime evidence.

In the matching `teamy-llm-service` checkout, start with `crates/teamy_llm_cli/src/cli/interactive/llm_interactive_cli.rs`, then inspect `src/interactive/engine.rs`, `worker.rs`, `tui.rs`, `script.rs` and `terminal_output.rs` beneath the CLI crate. Model defaults live in `crates/teamy_llm_service/src/model_defaults.rs`; generation residency lives in the service and GGUF adapter.

The command hierarchy uses Facet and Figue, following [argument parsing](cli-parsing.md) and [the Rust CLI template](rust-cli-template.md). [Interactive terminal interfaces](terminal-interfaces.md) connects this implementation to Cloud-Terrastodon's pickers, teamy-tts, Ollama and K9s. Reuse those ideas by purpose and keep their different runtime and terminal contracts visible.
