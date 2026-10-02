# Find a solution

Start with the problem you want to solve. This book explains how Teamy projects have solved it before, why we chose that approach, and where to inspect the implementation. Repository names identify evidence within a topic; they are not the navigation hierarchy.

New Teamy CLI projects use Rust and [teamy-rust-cli](rust-cli-template.md). Extend those shared capabilities deliberately. For desktop and inference work, reuse the relevant prior art and connect it to our CLI, logging, cancellation and output conventions.

## Command-line tools

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Start a new Rust CLI | [Understand the template](rust-cli-template.md) | teamy-rust-cli. |
| Parse arguments and expose clear help | [Command-line argument parsing](cli-parsing.md) | Facet, Figue, the template and Cloud-Terrastodon. |
| Add logs, file output or cross-process diagnostics | [Logging with tracing](logging.md) | The template, Piing and teamy-mft. |
| Explain startup delay, find CPU hotspots or measure completed GPU work | [Understand performance](performance-analysis.md) | Template spans, teamy-profiler's Cargo harness and CPU capture decoder. |
| Stop work safely | [Cooperative cancellation](cancellation.md) | teamy-cancellation and consuming CLIs. |
| Return results to scripts and people | [Typed results and errors](output-shape.md) | The template and the local mover prototype. |

## Desktop applications

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Load or extract the EXE icon without another `include_bytes!` embedding | [Executable icons and resources](executable-icons.md) | The template, tb and teamy-windows. |
| Add a system tray icon to this app | [Windows system tray icons](system-tray.md) | Piing, tb and teamy-windows. |
| Show a native message box or configuration-error dialog | [Native message boxes](native-message-boxes.md) | Piing's TaskDialogIndirect and Common-Controls v6 manifest. |
| Create a window and handle its events | [Window creation and event loops](window-creation.md) | tb and Piing's Windows helpers. |
| Take a screenshot, capture a window or inspect its controls | [Screen capture and typed observations](screen-capture.md) | WINC, Cursor-Hero's UI Automation and ShareX. |
| Embed a terminal or run an interactive child process | [Terminal integration](terminal-integration.md) | Teamy-Studio's PTY, terminal core and rendering boundary. |

## Machine learning and audio

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Turn a timeline model announcement into a local Rust tool | [Model adoption workflow](model-adoption.md) | Evidence manifests, typed contracts, reference fixtures and resident speech runtimes. |
| Review a plan, retain a decision or check whether examples compile | [Plan records and decisions](plan-records.md) | Versioned data, quote anchors, validation coverage and approved public projections. |
| Choose the best way to run inference on a GPU | [GPU-enabled inference](gpu-inference.md) | teamy-tts and teamy-transcriber's published backends. |
| Run Jev-style decisions locally or score text and image choices | [Finite-choice models](finite-choice-models.md) | Julia-1, explicit vision limits and native Rust versus ONNX providers. |
| Run a local text model, choose its prompt format or reuse its loaded runtime | [Generate text locally](local-text-generation.md) | Local teamy-llm prompt, pinned OrcaRouter GGUF and Makepad CUDA session. |
| Convert a Python ML implementation to Rust | [Port Python ML to Rust](python-ml-to-rust.md) | Model loading, preprocessing, parity and profiling in our speech tools. |
| Produce independent Python model fixtures with limited host access | [Containerized Python references](python-reference-containers.md) | Observed isolated Podman/uv/Torch CPU and CUDA reference, explicit WSL2 GPU setup and frozen inputs. |
| Learn from Makepad and keep control of the runtime | [Reuse Makepad's inference patterns](makepad-patterns.md) | Makepad study references and our speech-tool implementations. |

## Source and file organization

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Find the source for a library or project | [Find source before downloading](source-lookup.md) | The locator, teamy-mft and Cargo's source cache. |
| Plan where existing files should go | [Plan file movement](moving-files.md) | The local planning-only teamy-mover prototype. |
| Avoid path-length failures | [Windows path length](windows-paths.md) | Git for Windows and the template's resource manifest. |
| Decide what can be cleaned or retained | [Repository hygiene and scratch](repository-hygiene.md) | Git/Cargo previews and explicit material lifecycles. |

## Keep the prior art useful

Use book search with the operation you want: `EXE icon`, `message box`, `tray icon`, `screenshot`, `UI Automation`, `GPU inference`, `Python ML`, `PTY`, `argument parsing`, `find source` or `plan moves`. Each topic should connect an intent to code and then to related decisions. If a capability is missing from the template, say so rather than implying every CLI already has it.

[Curate prior art](curating-prior-art.md) explains how we turn existing projects and GitHub stars into recommendations. A source snapshot, a tested outcome and a research candidate have different evidence. Keep those differences visible.

[Find an authored Stack Overflow answer](stackoverflow-prior-art.md) connects earlier answers to their purposes, including EXE icon extraction, terminal colours, UI Automation, Rust and formal languages. Inspect current source before adopting historical examples.

For a constrained grammar, parser, semantic checker or verified state machine, read [formal methods and language implementation](formal-methods.md). It connects Prolog, NuSMV, Alloy and the paused Bend rewrite to their distinct purposes.
