# Section discovery CLI

This Rust vertical slice lists public mdBook sections as a reusable library and standalone command. It follows the current Teamy Rust CLI's Facet/Figue types, subcommand directory modules, typed command results, caller-owned output writer, shared cancellation, and one startup/runtime/logging owner. `rust-src/` preserves the book's existing `src/` directory. A future top-level Teamy CLI can compose the library/dispatch with its own startup and runtime.

## Commands and contract

~~~powershell
cargo run --bin teamy-docs -- sections list --output-format json
teamy-docs sections list --book-root . --output-format json > sections.json
~~~

Book selection uses `--book-root`, then `TEAMY_DOCS_BOOK_ROOT`, then the saved per-user `book-root.txt`, then exactly the current directory. Relative command/environment roots resolve from the current directory. An invalid selected root or malformed/unreadable saved preference fails without trying another book; an explicit `--book-root` bypasses lower-priority environment/config values. Discovery never searches ancestors or unrelated directories and does not create config files.

The installer saves this repository as the default after successful installation. To select another existing book, run `teamy-docs book-root set --path <book-directory>`; this validates `book.toml`, canonicalizes the root and replaces the saved preference. If the repository moves, rerun the installer or setter. The library uses `directories-next` with `ProjectDirs::from("", "TeamDman", "teamy-docs").config_dir()` for platform config storage. `TEAMY_DOCS_CONFIG_DIR` can override that directory with an absolute path, useful for isolated sessions/tests. `TEAMY_DOCS_BOOK_ROOT` still overrides a saved preference. Saved files are bounded to 64 KiB and read as strict UTF-8.

`book.toml` supplies `[book].src` (default `src`). Distinct local Markdown pages linked from `SUMMARY.md` are read in first appearance order. Paths use forward slashes relative to the source root. Parent traversal, encoded/query/backslash paths, and local children under external/draft SUMMARY parents fail explicitly. Canonical containment rejects symlinks resolving outside book/source roots.

Default bounds are 2 MiB per input file, 32 MiB total input, 1,024 pages, and 10,000 sections. Named arguments adjust each. Errors produce no successful partial report. The finite version-1 JSON object has `schema_version`, `content_basis`, `pages`, and `sections`. Pages retain SUMMARY hierarchy and parent paths.

Every page has a level-0 preamble, even when empty. Each heading has a record, retained when empty. `text` and `markdown` are the direct body until the next heading of any level; descendants are not duplicated. Heading ancestors, nearest shallower `parent_id`, byte offsets, and source line retain context. `source_start` includes the heading; body fields exclude it. The parser handles ATX/Setext headings and ignores apparent headings in fenced/indented code. The plain-text view retains code but omits raw HTML markup; body Markdown retains original markup apart from outer whitespace.

`content_basis: "markdown-source"` is deliberate: no examples/preprocessors are executed and include directives are not expanded. IDs are page path plus source-derived anchors with numeric duplicate suffixes and explicit `{#id}` support. Duplicate explicit IDs and empty generated slugs fail; supply an explicit ID for the latter. The synthetic fixture verifies twelve anchors against pinned mdBook 0.5.4, including repeated/colliding headings, inline code, Setext, and explicit IDs. Preprocessors and raw-HTML headings remain outside this source contract. File moves, heading renaming, or inserting an earlier duplicate can change IDs; explicit IDs improve stability.

Pipes default to JSON and terminals to text; `--output-format json|text` selects explicitly. Diagnostics go to stderr. CSV is absent because the template serializer cannot serialize nested sequences. Write/flush/serialization errors reach callers. Cancellation is checked between reads/parser events/output phases; a currently blocked filesystem read or stdout write cannot be interrupted mid-call.

## Resident relevance contract

The report must be mapped to one strict LLM request per section:

~~~powershell
$work = @(./examples/New-SectionDecideRequests.ps1 -ReportPath ./sections.json -Query '--help')
$work[0].request | ConvertTo-Json -Depth 8 -Compress
~~~

The pure mapper creates provenance envelopes and puts the strict Decide payload under `request`. Payload fields are `version: 1`, `model`, `question`, `input` (`text`, `visuals: []`), and two options with IDs `relevant`/`irrelevant` and `content.text`/`content.visuals: []`. Section ID, page path, anchor, and ordinal stay outside the request. Decide denies unknown fields and reads one complete UTF-8 JSON document per invocation, capped at 2 MiB. Multiple NDJSON requests cannot share one stdin invocation.

The pipeline example requires PowerShell 7. After explicitly choosing inference, run plain `teamy-llm serve` separately. First compatible Decide lazily loads Julia; later requests reuse it. Serve's generation prewarm flags are not verified decision prewarm. Use `--daemon`, not `--address`:

~~~powershell
$results = $work | ForEach-Object -Parallel {
    $OutputEncoding = [Text.UTF8Encoding]::new($false)
    $envelope = $_
    $json = $envelope.request | ConvertTo-Json -Depth 8 -Compress
    $resultJson = $json | teamy-llm decide --daemon --request -
    if ($LASTEXITCODE -ne 0) { throw ('Decide failed for ' + $envelope.section_id) }
    $result = $resultJson | ConvertFrom-Json
    [pscustomobject]@{
        index = $envelope.index
        section_id = $envelope.section_id
        best_option_id = $result.'best-option-id'
        scores = $result.result.scores
    }
} -ThrottleLimit 2
if (@($results).Count -ne $work.Count) { throw 'One or more section requests failed; result set is incomplete.' }
$results | Sort-Object index
~~~

This example has not run inference. Throttling bounds client/IPC work, not concurrent model computation: the service has one sequential decision actor, queue capacity eight, and a GPU mutex shared with generation. Compatible provider/device/model/encoding configuration preserves residency; changes replace the engine and failures/cancellation may discard it.

Wire size is separate from context fit. Julia defaults to 1,024 lossless tokens and 256 head tokens, with a supported maximum of 8,192. Over-budget input fails instead of silently truncating. The mapper's character and wire-byte caps do not prove token fit. Paragraph-aware chunking/aggregation is later work. Output uses kebab-case keys such as `best-option-id` and score `option-id`. Softmax probabilities are not calibrated confidence.

The mapper/contract checks never start a service, load a model, or download anything. All fixtures are synthetic.

## Validation and integration

~~~powershell
cargo fmt --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
./examples/Test-SectionDecideRequests.ps1 -ReportPath ./synthetic-sections.json
~~~

Integration adds Cargo manifest/lockfile, `rust-src/`, synthetic tests, `examples/`, and this guide, preserving existing mdBook content, license, scripts, and dirty work. Append Rust ignores to existing `.gitignore`; do not overwrite it. Book/nav changes require prescribed `scripts/build.ps1` and `scripts/check-links.ps1`. The implementation does not install the executable automatically.

## Install or update the executable

Run the helper to install into Cargo's existing configured bin directory:

~~~powershell
pwsh -NoProfile -File ./update.ps1 -Offline
# Equivalent installer entry point:
pwsh -NoProfile -File ./install.ps1 -Offline
teamy-docs section list --output-format json
~~~

Both entry points run `cargo install --path <this repository> --locked`, preserving `Cargo.lock` and resolving the repository from `$PSScriptRoot` even when invoked elsewhere. After installation succeeds, a locked release-mode `cargo run` of that same manifest invokes `book-root set --path <this repository>`, so the installed command can find its book from any directory. Rust owns the platform config path. Both calls honor `-Offline`; omit it when dependency downloads are intended. The helpers guard a missing manifest/lockfile/book and report installation or registration failures separately. Registration failure leaves the successfully installed binary available but requires repairing config or selecting `--book-root`. They do not pull Git, update dependencies, change PATH, or change execution policy. `sections list` remains canonical; `section list` is a supported alias with the same output.


## Independent diagnostic sinks

The CLI now adopts the local teamy-rust-cli logging contract. --log-filter (alias --log-level) selects stderr; --log-format selects text or ndjson. Stderr resolves explicit filter, then --debug, then RUST_LOG, then INFO. --log-file requires an explicit file path and appends; --log-file-filter defaults to DEBUG independently and --log-file-format defaults to ndjson. --output-format continues to select only section-result stdout. Full EnvFilter directives are accepted with regex matching disabled. Startup failures and parse errors have safe generic stderr presentation; arbitrary application fields still require producer review. Emit direct tracing events at meaningful operation call sites with safe operation-specific counts or timings. Routine dispatch and completion are DEBUG. No generic progress lifecycle or schema is required. Every successful command returns an explicit completion warning budget, including output; the threshold is execution metadata and does not change result serialization. Invocation errors have no returned budget. The warning is not a watchdog and does not change exit status.

Successful command receipts carry warning-only completion budgets, excluded from result JSON. Book-root set returns one second. Sections list allows one second plus 100 ms per permitted MiB, 5 ms per permitted page and 1 ms per permitted section (19.32 seconds at default limits). The check includes output rendering and preserves rendering failures. No public book content or hosting configuration is changed by this CLI contract.
