# Find a solution

This is reference material. Inclusion in an index or survey is not endorsement or adoption. [TeamyStyle](teamystyle.md) owns the engineering standards; recommendations here require a stated purpose, rationale, source evidence and decision/adoption status.


Start with the problem you want to solve. This book explains how Teamy projects have solved it before, why we chose that approach, and where to inspect the implementation. Repository names identify evidence within a topic; they are not the navigation hierarchy.

New Teamy CLI projects use Rust and [teamy-rust-cli](rust-cli-template.md). Extend those shared capabilities deliberately. For desktop and inference work, reuse the relevant prior art and connect it to our CLI, logging, cancellation and output conventions.

## Command-line tools

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Drive a Minecraft dev client, inspect its GUI or evaluate ClientDevBridge for SFM puppet/container checks | [Minecraft client automation](minecraft-client-automation.md) | Shell bridge, loader/version mapping, screenshots and JDWP hot-swap; untested candidate. |
| Compare agent harnesses, use OpenCode or retain retrievable chat memory | [Agent harnesses and memory survey](agent-harnesses.md) | OpenCode execution harness and OMP OptChat summary-tree extension; evaluation status and limits. |
| Flag malicious or mistaken Teamy Agent command requests before execution | [Agent command review reference](plan-records.md#flag-suspicious-agent-command-requests) | Unread discovery post and an unimplemented local decide idea; deterministic permissions and approval remain authoritative. |
| Stop an agent drifting, generate a burndown or choose the next migration item | [Keep a migration aligned and fast](migration-workflows.md) | SFM's local source oracle and a deterministic edit/check loop. |
| Speed up a slow agent loop, selector or repeated migration check | [Measure iteration overhead](migration-workflows.md#measure-the-complete-iteration) | SFM's observed selector delay; profiling, incremental validation, caching and measured reasoning/delegation choices. |
| Improve tooling while still completing the work it selects | [Optimize while advancing useful work](performance-analysis.md#optimize-while-advancing-useful-work) | Timed selection across real accepted changes, isolated candidates and a fixed optimization cutoff. |
| Apply TeamyStyle, structure domain types or review a software design | [TeamyStyle](teamystyle.md) | Established engineering rules, recommendations, concrete examples and review checklist. |
| Start a new Rust CLI | [Understand the template](rust-cli-template.md) | teamy-rust-cli. |
| Enforce Rust rules, reject print macros or make output testable | [Rust code standards](code-standards.md) | Local template lint policy, supplied writers and compile fixtures; CLI-package adoption in teamy-llm-service. |
| Parse arguments and expose clear help | [Command-line argument parsing](cli-parsing.md) | Facet, Figue, the template and Cloud-Terrastodon. |
| Add logs, file output or cross-process diagnostics | [Logging with tracing](logging.md) | The template, Piing and teamy-mft. |
| Convert styled logs into Ratatui without clobbering the screen | [TUI log representations](logging.md#choose-a-representation-before-styling-a-tui-log) | Cloud-Terrastodon's structured toasts and the LLM's bounded SGR-to-span conversion. |
| Capture logical spans in an eyre error or diagnose missing span traces | [Span traces and error hooks](logging.md#attach-tracing-spans-to-eyre-errors) | ErrorLayer, color-eyre and Cloud-Terrastodon's subscriber regression. |
| Preserve typed log fields and error context between processes | [IPC telemetry contracts](logging.md#preserve-field-types-before-crossing-ipc) | Current string-value limitations and a proposed Facet capture/rendering boundary. |
| Explain slow commands, find CPU hotspots or measure completed GPU work | [Optimize performance](performance-analysis.md) | Instant budgets, tracing/NDJSON, Tracy capture/viewer and teamy-profiler's Cargo harness and CPU decoder. |
| Try a faster Rust heap allocator for allocation-heavy work | [Try mimalloc](performance-analysis.md#try-mimalloc-for-allocating-hot-paths) | Microsoft's application guideline and teamy-mft's executable allocator selection. |
| Warn when a command exceeds its reasonable runtime | [Command runtime expectations](performance-analysis.md#make-slow-commands-warn-during-normal-use) | The local template's small Instant-based guard, phase checkpoints and final check. |
| Stop work safely | [Cooperative cancellation](cancellation.md) | teamy-cancellation and consuming CLIs. |
| Return results to scripts and people | [Typed results and errors](output-shape.md) | The template and the local mover prototype. |
| Add a TUI, picker or interactive prompt | [Interactive terminal interfaces](terminal-interfaces.md) | Ratatui, Cloud-Terrastodon's pick/browser, teamy-tts and Ollama; K9s navigation reference. |
| Stop tracing logs from overwriting a TUI, or open a nested picker | [Terminal ownership and logs](terminal-ownership.md) | Cloud-Terrastodon's coordinator, suspension acknowledgements and structured terminal log buffer. |
| Keep keyboard input responsive while inference or searches run | [Async terminal work](async-terminal-ui.md) | Cloud-Terrastodon's EventStream, generation-tagged candidate handlers and engine/UI separation. |

## Desktop applications

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Choose a Rust GUI, compare UI frameworks or evaluate Android companion support | [Choose a desktop UI framework](ui-frameworks.md) | GPUI Kit, Makepad, Bevy, egui/eframe, Iced and raw DirectX; dated evaluation evidence and gaps. |
| Load or extract the EXE icon without another `include_bytes!` embedding | [Executable icons and resources](executable-icons.md) | The template, tb and teamy-windows. |
| Add a system tray icon to this app | [Windows system tray icons](system-tray.md) | Piing, tb and teamy-windows. |
| Show a native message box or configuration-error dialog | [Native message boxes](native-message-boxes.md) | Piing's TaskDialogIndirect and Common-Controls v6 manifest. |
| Create a window and handle its events | [Window creation and event loops](window-creation.md) | tb and Piing's Windows helpers. |
| Take a screenshot, capture a window or inspect its controls | [Screen capture and typed observations](screen-capture.md) | WINC, Cursor-Hero's UI Automation and ShareX. |
| Embed a terminal or run an interactive child process | [Terminal integration](terminal-integration.md) | Teamy-Studio's PTY, terminal core and rendering boundary. |

## Machine learning and audio

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Generate images, evaluate Iris-3B or reuse a pixel-space prior for depth/restoration | [Image-generation survey](image-generation.md) | No-VAE architecture, fine-tunes, runtime requirements and source/weight licensing. |
| Reduce vector memory, compress a semantic-search index or find TurboQuant/MRL prior art | [Vector memory and search compression](native-multimodal-embeddings.md#vector-memory-and-search-compression-references) | Qdrant EmbeddingGemma 2 text-retrieval measurements; optional beta Edge Rust reference, no adoption. |
| Turn a timeline model announcement into a local Rust tool | [Model adoption workflow](model-adoption.md) | Evidence manifests, typed contracts, reference fixtures and resident speech runtimes. |
| Review a plan, retain a decision or check whether examples compile | [Plan records and decisions](plan-records.md) | Versioned data, quote anchors, validation coverage and approved public projections. |
| Choose the best way to run inference on a GPU | [GPU-enabled inference](gpu-inference.md) | teamy-tts and teamy-transcriber's published backends. |
| Run Jev-style decisions locally or score text and image choices | [Finite-choice models](finite-choice-models.md) | Julia-1, shared Qwen weights and Clef's trained multi-question head. |
| Reuse one input for several questions or compare Clef with our deciders | [Compare decision providers](finite-choice-models.md#compare-clef-with-our-decision-providers) | The announcement, model cards, reference head and compatible runtimes; external prior art. |
| Speed up choice scoring by skipping unused vocabulary outputs | [Remove unused decision work](performance-analysis.md#remove-unused-decision-scoring-work) | Qwen output projection and readback; proposed changes with numerical and timing gates. |
| Run a local text model, choose its prompt format or reuse its loaded runtime | [Generate text locally](local-text-generation.md) | Local teamy-llm prompt, pinned OrcaRouter GGUF and Makepad CUDA session. |
| Connect prompt, decision or interactive CLI clients to one running server | [Resident inference connections](resident-inference.md) | Typed Vox IPC, explicit daemon selection and request-scoped tracing from teamy-mft prior art. |
| Describe an image, compare screenshots or place images inside a prompt | [Prompt with local images](image-prompts.md) | Direct-path Liquid tags, typed prompt parts and the local Makepad vision adapter. |
| Prompt repeatedly, score choices interactively or set a default model | [Resident interactive inference](interactive-inference.md) | Local typed teamy-llm commands, Ratatui, captured worker diagnostics and explicit model defaults. |
| Convert a Python ML implementation to Rust | [Port Python ML to Rust](python-ml-to-rust.md) | Model loading, preprocessing, parity and profiling in our speech tools. |
| Produce independent Python model fixtures with limited host access | [Containerized Python references](python-reference-containers.md) | Observed isolated Podman/uv/Torch CPU and CUDA reference, explicit WSL2 GPU setup and frozen inputs. |
| Learn from Makepad and keep control of the runtime | [Reuse Makepad's inference patterns](makepad-patterns.md) | Makepad study references and our speech-tool implementations. |

## Email inspection, composition and protocols

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Build teamy-email.exe, inspect or compose .eml messages, or own an email stack | [Plan an owned email CLI](email-planning.md) | Proposed Rust/Facet/Figue stages; Makepad MIME and source/index prior art, no implementation or adoption. |
| Find Makepad email, MIME, SMTP or IMAP support | [What Makepad provides](email-planning.md#what-makepad-already-provides) | App decoder and demo send behavior; separate networking crate with backend limits. |
| Find the Rust SMTP repository or plan secure submission and mailbox sync | [SMTP versus IMAP reference](email-planning.md#correct-the-smtp-versus-imap-reference) and [proposed stages](email-planning.md#start-with-offline-inspectable-behavior) | rust-imap is an IMAP client; lettre supplies SMTP integration-test fixtures. Mature TLS remains a separate boundary. |

## Video creation and animation

| What you want to do | Read this | External references |
| --- | --- | --- |
| Make videos with code, reuse React video layouts or animate data | [Make videos with code](making-videos.md) | Remotion; special licensing terms, not an adopted dependency. |
| Create explanatory vector motion graphics or synchronize animation with narration | [Compare animation references](making-videos.md) | Motion Canvas TypeScript animation. |
| Create Python mathematical animations, evaluate Rust GPU rendering or study 3Blue1Brown's workflow | [Compare Manim editions and ManimGX](making-videos.md) | Manim Community, original Manim/ManimGL and ManimGX; separate APIs, behavior and licensing limits. |

## Source and file organization

| What you want to do | Read this | Implementation references |
| --- | --- | --- |
| Put docs.teamdman.ca on GitHub Pages using Namecheap IaC and 1Password | [Custom-domain IaC plan](publication.md#plan-the-custom-domain-through-existing-iac) | Existing SFM secret-loading/delegation reference; current domain/state ownership and a bounded record change still need verification. |
| Find the source for a library or project | [Find source before downloading](source-lookup.md) | The locator, teamy-mft and Cargo's source cache. |
| Read a takeout or GDPR export ZIP/TAR without full extraction | [Bounded archive access](takeout-archives.md#read-bounded-members-directly) | thrumzip's offset readers; bounded members and versioned provenance. |
| Validate an explicit Instagram ZIP and report optional export coverage | [First archive validation slice](takeout-archives.md#start-with-an-inspectable-vertical-slice) | Proposed teamy-instagram archive validate command, typed known-file models and synthetic tests. |
| Archive saved/bookmarked Instagram reels, sent or received shares, likes and watch evidence | [Normalize Instagram evidence](takeout-archives.md#normalize-observed-instagram-evidence) | Reviewed export shapes; direct reel likes and separate message reactions. |
| Preview resumable media downloads and retain video/JSON/transcripts as ordinary files | [Plan and materialize a local archive](takeout-archives.md#start-with-an-inspectable-vertical-slice) | teamy-youtube receipts, teamy-transcriber APIs and the proposed teamy instagram namespace. |
| Plan where existing files should go | [Plan file movement](moving-files.md) | The local planning-only teamy-mover prototype. |
| Avoid path-length failures | [Windows path length](windows-paths.md) | Git for Windows and the template's resource manifest. |
| Decide what can be cleaned or retained | [Repository hygiene and scratch](repository-hygiene.md) | Git/Cargo previews and explicit material lifecycles. |

## Keep the prior art useful

Use book search with the operation you want: `EXE icon`, `message box`, `tray icon`, `screenshot`, `UI Automation`, `GPU inference`, `Python ML`, `PTY`, `TUI`, `Ratatui`, `interactive prompt`, `terminal ownership`, `argument parsing`, `find source` or `plan moves`. Each topic should connect an intent to code and then to related decisions. If a capability is missing from the template, say so rather than implying every CLI already has it.

[Curate prior art](curating-prior-art.md) explains how we turn existing projects and GitHub stars into recommendations. A source snapshot, a tested outcome and a research candidate have different evidence. Keep those differences visible.

[Find an authored Stack Overflow answer](stackoverflow-prior-art.md) connects earlier answers to their purposes, including EXE icon extraction, terminal colours, UI Automation, Rust and formal languages. Inspect current source before adopting historical examples.

For a constrained grammar, parser, semantic checker or verified state machine, read [formal methods and language implementation](formal-methods.md). It connects Prolog, NuSMV, Alloy and the paused Bend rewrite to their distinct purposes.
