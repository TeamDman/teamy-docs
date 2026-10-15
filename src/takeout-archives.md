# Turn service exports into a local archive

Start with a proposed `teamy-instagram archive validate <zip-path>` command: read one explicitly supplied local ZIP, parse supported metadata into typed models, and report optional-section coverage and schema gaps. Later takeout/GDPR tooling can register archive roots, plan media downloads and produce ordinary JSON, video and transcript files.

This is an architecture proposal, grounded in source inspection and a read-only Instagram export review on 4 October 2026. The proposed `teamy instagram` commands and takeout domain crates do not exist yet. Source inspection, executable help and end-to-end execution establish different things; keep their status visible. The export review is an exploratory survey of selected metadata, not an implemented reusable inspector or an exhaustive activity audit. Watch/view/play evidence may occur outside watch-named files; completeness remains unestablished.

## Reuse the prior art with its limits

| Reference | Verified capability | Status and adaptation |
| --- | --- | --- |
| [teamy-rust-cli command schema](https://github.com/TeamDman/teamy-rust-cli/blob/99460d927c745a43285b0c5206fc2d42cec33ebc/src/cli/mod.rs) | Typed Facet/Figue commands, nested invocation, shared startup and typed output. | Current public source snapshot; the [template chapter](rust-cli-template.md) records its earlier baseline and newer local work. Archive configuration and takeout ingestion still need implementation. |
| [thrumzip reader](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/src/read_entries_from_zips.rs#L36) and [ZipEntry](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/src/zip_entry.rs#L36) | Typed archive/member paths, central-directory enumeration and on-demand member readers. | Historical schema-agnostic archive code. Reuse the access pattern; bound concurrency and replace whole-member buffering. |
| [Nanuak YouTube ingest](https://github.com/TeamDman/Nanuak/blob/f437e26f65d4f5c6b6eb82449fae8780474fb927/nanuak-youtube-takeout-ingest/src/entry.rs#L14) | Converts exported time, title, title URL and subtitles into search, watch and post-view events. | Historical extracted-JSON-to-Postgres binary, with no ZIP reader or saved/shared/reaction model. Reuse event normalization, not its storage assumptions. |
| [teamy-youtube channel sync](https://github.com/TeamDman/teamy-youtube/blob/1f936f30b2f1dfaa531c368bbe1650e2a2c66904/src/channel_sync/mod.rs#L121) | Separate plan and execution functions, yt-dlp integration, and request/completion/failure records. | Reuse the adapter and receipts. Its existing-media lookup depends on teamy-mft; the takeout slice needs an independent filesystem/catalog lookup. |
| [teamy-discord sync records](https://github.com/TeamDman/teamy-discord/blob/81101e3c8721ae35c6923bf4a2e74b903c4f860e/src/archive/sync_engine.rs#L33) and [output-root resolution](https://github.com/TeamDman/teamy-discord/blob/81101e3c8721ae35c6923bf4a2e74b903c4f860e/src/paths/output_dir.rs#L84) | Open message/attachment records, blob digests, resumable checkpoints reconstructed from output, and CLI/environment/saved-root precedence. | Existing archive storage prior art. Its CLI output composition is less uniform; adapt it to the shared typed output contract. |
| [teamy-transcriber workflow](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/workflow.rs#L148) and [storage](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/storage.rs#L19) | Recording preparation, transcription, cancellation variants, manifests/events and text export. | Existing library APIs. A reel adapter must map its UUID recording store and exported chunks into the open archive layout. |
| [teamy-llm finite choices](https://github.com/TeamDman/teamy-llm-service/blob/6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a/crates/teamy_llm_core/src/finite_choice.rs) and [Decide CLI](https://github.com/TeamDman/teamy-llm-service/blob/6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a/crates/teamy_llm_cli/src/cli/decide/llm_decide_cli.rs) | Structured finite-choice input and ordered scores, with text/JSON output. | Optional annotation prior art. Current installed help also exposes daemon routing; the served Decide RPC is newer local working-tree code, without an end-to-end inference check here. |

### Historical attempts: Meta and YouTube exports

[Thrumzip](https://github.com/TeamDman/thrumzip/tree/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407) explored comparison and deduplication of repeated Meta exports. Its README and historical Meta-takeout plans motivate that problem. At the inspected revision, the implemented reader is generic archive handling: it has no saved, liked or shared Instagram schema parser. Exploratory examples and planned behavior must remain distinct from completed commands.

[Nanuak's YouTube takeout ingest](https://github.com/TeamDman/Nanuak/tree/f437e26f65d4f5c6b6eb82449fae8780474fb927/nanuak-youtube-takeout-ingest) is a historical extracted-JSON-to-Postgres attempt. Its English-prefix event normalization is useful prior art, but it materializes whole JSON files and assumes a database. Its timestamp-only uniqueness model also needs replacement, as discussed below. Preserve the lesson without making full extraction or Postgres the new architecture's default.

## Future: register explicit source roots

Store a versioned list of canonical archive roots under the application's platform configuration directory, with an explicit override. Each registration states its provider and source scope, with an optional explicit exporter display-name/alias mapping. Removing a registration changes configuration; it does not delete an archive. Reuse the template's `AppHome`/cache boundary and pass resolved paths into domain code. A configuration read must not create directories or silently fall back to the current working directory.

Registration should expand only the documented variable/home syntax, resolve relative input against an explicit base, validate that the source exists, and canonicalize it before persistence. Report unresolved variables, missing roots and aliases of an already registered root. Preserve the user-entered spelling for diagnostics alongside the canonical path. Discover a redirected Documents folder through the platform's folder configuration when needed; constructing a path from a home directory is insufficient.

The [YouTube directory setter](https://github.com/TeamDman/teamy-youtube/blob/1f936f30b2f1dfaa531c368bbe1650e2a2c66904/src/cli/sync/dir/set/sync_dir_set_cli.rs#L27) demonstrates expansion, canonicalization and persistence for an output directory. For input archive roots, omit its directory-creation effect. Thrumzip's profile registration only trims quotes and does not establish this stronger contract.

Persist scan scope and symlink/reparse-point policy explicitly. Enumerate registered roots with ordinary directory APIs; this design has no MFT or USN dependency. Do not search unrelated drives to compensate for an unavailable source.

Record filename export dates separately from filesystem modification times. A copied archive's modification time can differ from its export date, and a newer export can omit older records. Keep distinct exports and multipart members; neither chronological order nor a matching member name establishes replacement.

## Read bounded members directly

A 40 GB ZIP needs a seekable file and member readers, not a 40 GB memory buffer or full extraction. Read the central directory, retain member metadata, and decompress only selected entries. Metadata memory still scales with entry count; bound archive/member concurrency and limit metadata size.

Thrumzip opens a shared random-access file and constructs an async reader at a member offset. Its patched rc-zip implementation supports offset-based ZIP64 reading. However, [`ZipEntry::bytes` and `write_to_file`](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/src/zip_entry.rs#L43) buffer a whole decompressed member. Use a streaming copy for media. For metadata, use an incremental parser where qualified, or a declared bounded buffer that rejects oversized entries before allocating them.

Expose an archive adapter with entry enumeration and `open_member` operations. ZIP can use offsets. TAR and compressed TAR need bounded sequential iteration; a generated index is an explicit cache optimization, not an assumed central directory. Preserve the original archive as the source of truth.

Distinguish directory CRC/size metadata from payload verification. Consuming a member to EOF can validate its reader's expected size/CRC. Thrumzip's [CLI `validate` audit body](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/src/command/validate_command.rs#L124) is unfinished, despite exploratory validation examples and README statistics. Use CRC/size for cheap comparisons and a streamed strong digest when exact content identity is required.

The [sync implementation](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/src/command/sync_command.rs#L68) skips existing member paths and collapses duplicates by CRC alone. Its perceptual image policy chooses the smallest uncompressed member and hardcodes a threshold instead of using the profile preference. Preserve changed versions and originals. A smaller JPEG or a similar perceptual hash does not establish equivalent quality; defer perceptual deduplication beyond the first slice.

Thrumzip also has [absolute local rc-zip dependency patches](https://github.com/TeamDman/thrumzip/blob/bcb2644f3f5d3a9ac11ee813513bbf40a60f8407/Cargo.toml#L33). Pin a reviewed portable dependency or fork before adopting this code.

## Normalize observed Instagram evidence

The reviewed JSON export has saved and liked records with direct reel URLs, message-share/reaction records, story-view records referencing reel URLs, and sparse ad-related watch records. These observations do not establish complete history or one universal Meta export format. Inspect structures and nested links beyond watch-named files; a filename alone cannot establish an activity's meaning.

Saved and liked entries use a root array of records with `timestamp`, `media`, `label_values` and string `fbid`. The URL is a label-value item with `label: "URL"`, `value` and `href`. Optional Caption/Title values and nested `title`/`dict` groups also occur. Do not assume the older `string_list_data` shape.

This is a synthetic structural example, with an invented timestamp, identifier and reel shortcode:

```json
[
  {
    "timestamp": 1700000000,
    "media": [],
    "label_values": [
      {
        "label": "URL",
        "value": "https://www.instagram.com/reel/EXAMPLE01/",
        "href": "https://www.instagram.com/reel/EXAMPLE01/"
      },
      {"label": "Caption", "value": "[synthetic caption]"}
    ],
    "fbid": "100000000000001"
  }
]
```

Treat `fbid` as an opaque exported field until its identity semantics are established. Extract the reel shortcode from a validated permalink; keep both the original and canonical URL. Preserve the shortcode's case and remove tracking queries from canonical public links. `/reel/` links were observed; a `/reels/` alias can be supported by a separate synthetic fixture without claiming it occurred in this export. A `/p/` or `/tv/` reference has unknown media type until its metadata establishes it.

| Evidence | What it supports | Limits |
| --- | --- | --- |
| Saved/bookmarked record | The export associates a reference with saved activity and its source timestamp. | Preserve non-reel references and unresolved rows; saving is distinct from liking. |
| Liked-post record | Evidence of liking the referenced post or reel. | It says nothing about a reaction to a shared message. |
| `your_instagram_activity/story_interactions/stories_viewed.json` | Observed root-array timestamp/media/label-values/fbid records contain direct URL value/href reel references, plus Caption/Title and Name/Username metadata. | Retain the source-defined `StoriesViewed` kind and unresolved source-timestamp semantics. No duration, completion or explicit watched-at field was observed; this does not prove a completed play or complete watch history. |
| `personal_information/personal_information/note_and_repost_interactions.json` | Observed nested URL/Name/Username profile-like metadata. | No timestamp, link `href`, or watch/play/type field was found in the inspected shape; it does not establish reel watching. |
| `ads_information/ads_and_topics/videos_watched.json` | The observed records contain ad-related reel URLs in the same label-value record family. | Sparse advertising evidence is not a complete reel watch history. |
| `your_instagram_activity/subscriptions/reels.json` | A subscription object with URL/Name/Username metadata. | It is not a watched-reels inventory. |
| Shared message | A message-linked media reference, timestamp and sender name. | Determine outgoing/incoming only with an explicit, validated exporter identity mapping; ambiguous names stay unknown. |
| Message reaction | A reaction attached to the message and attributed to an actor. | It is message interaction, including when the message shares a reel; it does not establish a reel like. |

Message roots contain `participants`, `messages`, `title`, `is_still_participant`, `thread_path` and `magic_words`. Observed messages have `sender_name` and `timestamp_ms` in Unix milliseconds. Reactions have `reaction`, `actor` and an optional `timestamp` in Unix seconds. Preserve the raw value and validated source unit before converting either to a common time representation. Participant names alone do not identify the exporter.

A synthetic message fragment illustrates the separate share and reaction objects:

```json
{
  "sender_name": "<SELF_OR_OTHER_DISPLAY_NAME>",
  "timestamp_ms": 1700000000000,
  "content": "[synthetic message]",
  "share": {
    "link": "https://www.instagram.com/reel/EXAMPLE01/",
    "share_text": "[synthetic shared caption]",
    "original_content_owner": "<CREATOR_HANDLE>"
  },
  "reactions": [
    {
      "reaction": "<EMOJI>",
      "actor": "<DISPLAY_NAME>",
      "timestamp": 1700000001
    }
  ]
}
```

Some shares lack a link. `original_content_owner` is a creator-handle hint, not an immutable verified creator identity. Optional share media/profile fields and export-specific flags should retain their actual semantics; do not reinterpret `is_unsent_image_by_messenger_kid_parent` as a generic unsent-message flag.

Preserve unknown fields and report unsupported shapes with archive/member/record locators. Add adapters for older JSON or HTML formats only with separate fixtures and schema evidence. Do not quietly interpret a missing activity file as proof that no activity occurred.

### Model known files and optional coverage

The reusable inspector is proposed Rust tooling, replacing the exploratory in-memory inspection with tested Facet models. Use a typed known-file manifest with optional member descriptors: exports can omit activities because of provider selections, date ranges or format changes. A missing optional file, an unreadable file and an unsupported schema are different coverage findings. Stream or bound parsing through those descriptors rather than putting all metadata into a single vector.

Start with `your_instagram_activity/saved/saved_posts.json`, `your_instagram_activity/likes/liked_posts.json`, the story/ad/subscription/interaction files above, and message members. Inventory unknown members locally, report schema gaps and retain unknown labels/fields for future adapters. The template's current Facet/Figue pin is `23ad4dcd034d123f0c38943305ffb077519040ce` (Facet `0.50.0-rc.5`, Figue `5.0.0-rc.5`). The [container attribute](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/facet/src/lib.rs#L200) and [facet-json rejection test](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/facet-json/tests/integration/issue_1904.rs#L103) support `facet(deny_unknown_fields)`. A proposed debug form is `#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]`, using the [demonstrated cfg_attr pattern](https://github.com/TeamDman/facet/blob/23ad4dcd034d123f0c38943305ffb077519040ce/facet/tests/integration/derive.rs#L366); this exact new-inspector combination has not been compiled here. Test it against the adopted pin. Unknown fields are otherwise ignored by default, so production adapters still need raw-field retention or explicit coverage diagnostics.

## Keep identifiers and provenance separate

Use a stable provider key for a reel, and retain every evidence occurrence separately. An evidence record needs the source archive identity, member path, source record locator, parser/schema version, original URL, normalized URL, source event time and evidence type. Retain sender/actor/direction only when resolved, with the resolution basis. No stable message/account IDs were found in the reviewed message shape. Where provider IDs are absent, combine provenance and a semantic fingerprint, preserving distinct same-time messages and reporting ambiguous matches.

Archive size and modification time are cheap cache invalidation inputs, not integrity proofs. When a source changes, invalidate its parsed index or retain a new source revision. Deduplicate equivalent observations across exports while retaining their source occurrences. A timestamp alone is not an event key: Nanuak's [timestamp-only history primary keys](https://github.com/TeamDman/Nanuak/blob/f437e26f65d4f5c6b6eb82449fae8780474fb927/nanuak-schema/migrations/2024-12-08-162117_create_youtube_schema/up.sql#L5) can collapse distinct same-time events.

Keep `Saved`, `ReelLike`, source-defined `StoriesViewed`, ad-related `Watch`, `SharedOutgoing`, `SharedIncoming`, `SharedUnknown` and `MessageReaction` as distinct domain evidence. Missing links or unresolved identities should survive in the report; never manufacture a reel match from an author name and a nearby timestamp.

## Compose domains into one CLI

Use a takeout/archive core for registered sources, bounded access, source revisions and provenance. Put Instagram parsing and normalization in an Instagram domain crate, and typed commands in its CLI crate. Additional services can implement their own adapters over the same archive boundary. The LLM core/service/CLI workspace is a package-separation precedent; YouTube and the transcriber currently expose library modules from single packages. The package split and top-level `teamy` composition are proposed work, not capabilities supplied automatically by the current single-package template.

Follow [Facet/Figue parsing](cli-parsing.md), subcommand directory modules with descriptively named `*_cli.rs` files, nested async invocation, [typed results](output-shape.md), tracing and [cooperative cancellation](cancellation.md). Parse once and install logging, cancellation and the Tokio runtime once at the top level. Pass configuration, cancellation, adapters and output sinks into domain code instead of invoking another CLI's startup recursively.

Compact reports can use the existing `CliOutput` renderer. A complete archive index needs paged results or a separately specified streaming format: the current renderer materializes the whole rendered string. State framing, schema version and incomplete-output behavior. Align the actual Facet/Figue versions and fork revisions before embedding existing command types in a shared schema.

## Start with an inspectable vertical slice

The next implementation belongs in the proposed `teamy-instagram` repository. Build one explicit-path archive validation command with typed known-file models and synthetic fixture tests. Persistent source registration, a likes-list command and synchronization are deferred work.

The following is a proposed surface, not a runnable example:

```text
teamy-instagram archive validate "<zip-path>"
```

Open that local ZIP directly, enumerate its members, and validate the supported metadata shapes through bounded readers. Return a compact typed coverage report: recognized/parsed sections, absent optional sections, unknown members, unsupported shapes and parse/integrity issues. Report locators and field diagnostics without dumping message bodies or identity-bearing records.

A missing optional export section is not a failure: provider checkboxes, date ranges and export versions can omit it. An unreadable ZIP input, malformed included metadata and an unsupported included schema have different outcomes and must remain visible. Do not describe selective metadata validation as a complete payload audit of every media member.

Tests should establish direct ZIP access, the reviewed saved/liked/message/story shapes, absent optional sections, unknown-field policy, parser limits and diagnostic behavior. New source variants need independent fixtures. This first slice performs no downloads, configuration writes, model inference, perceptual deduplication, database access, MFT or USN work.

### Later: build a deterministic sync plan

Source registration and the top-level `teamy instagram` namespace are later integration work. When synchronization is implemented, its dry run should return the plan without downloads, source changes, inferred configuration writes or transcript/model work. Disclose any explicitly requested report/cache effects.

Order that plan by saved, then sent, then received references. Deduplicate canonical reel keys; aggregate their independent likes, view/watch and message-reaction evidence. Make tie-breaking explicit and stable. Unknown direction and unresolved references appear as findings, not guessed priorities. The same sources/configuration/catalog should yield the same plan, independent of directory enumeration order.

Each plan item should identify its reel key, canonical URL, priority reason, evidence IDs, intended output paths and current media/transcript state. Separate blocked/unavailable/private media findings from execution errors.

## Materialize ordinary media and transcript files

After the plan contract is tested, introduce a replaceable yt-dlp adapter and fake downloader. Reuse teamy-youtube's plan/execution and receipt patterns while supplying an MFT-independent catalog of completed local media. Capture the tool version, arguments, exit status and reported final path. Evaluate Instagram-specific extraction, authentication and unavailable-media behavior separately; a working YouTube invocation does not validate reel support.

Use an open layout with stable keys, for example:

```text
creators/<creator-key>/reels/<reel-key>/
  metadata.json
  evidence.ndjson
  video.<ext>
  video.info.json
  transcript.txt
  transcript.json
  events.ndjson
```

Treat these names as a proposed output contract. Preserve the downloaded media format unless an explicit derivative is requested. Keep yt-dlp's tool-produced info JSON unchanged as a sidecar alongside typed normalized metadata, recording its tool/schema provenance. A normalized projection must not replace the original capture. Keep creator display names/handles and URL aliases in metadata; attribution changes must not silently change a reel's identity or erase provenance. An unresolved creator remains explicit until attribution is established.

Wrap teamy-transcriber's preparation and transcription APIs with a recording-store adapter, then export text and typed chunk metadata into the reel folder. Its [text export](https://github.com/TeamDman/teamy-transcriber/blob/d87d5020d0a2c3847c5fa461d9bd9de39901b52e/src/workflow.rs#L882) supports an explicit output path. Source-range/clip timestamps do not provide word alignment or an SRT contract. Record the model, backend, source-media digest and transcript version.

Persist request/completion/failure events and verify a completion receipt against the file before skipping work. Use bounded queues, cancellation checks, temporary incomplete outputs and atomic finalization where supported. A restart should retry interrupted/failed work without replacing a valid completed artifact. Re-running with the same inputs should be idempotent; metadata/enrichment changes need explicit revisions.

## Add finite-choice annotations later

The local `decide` command is real: installed help verified `--daemon[=<STRING>]` for an optional local daemon endpoint and `--no-daemon` for the default in-process route. Current service code registers typed Vox IPC Decide; it is not an HTTP `/decide` endpoint. Help/source inspection here does not verify server startup or inference.

Use it optionally after deterministic parsing, for example to annotate a supplied transcript with named categories. Retain the question, ordered choices, model/provider revision and returned scores. Julia is text-only, and its softmax probabilities are not calibrated confidence. Do not use a score to invent saved/shared/liked evidence, resolve an ambiguous participant identity, or authorize downloading. See [finite-choice models](finite-choice-models.md) and [resident connections](resident-inference.md).

## Query documentation by section later

Teamy Docs includes the mdBook, build/link scripts, a reusable Rust section-discovery library, and the standalone `teamy-docs sections list` CLI (also available as `teamy-docs section list`). The command lists SUMMARY-linked pages and direct heading bodies as typed JSON, retaining preambles, empty sections, page/heading hierarchy, source-relative IDs and anchors, and byte/line provenance. The pure `examples/New-SectionDecideRequests.ps1` mapper prepares one current version-1 Decide request per section with `relevant` and `irrelevant` choices, retaining section provenance outside the strict request; it does not execute inference. Content hashes, candidate selection, explicit chunking, relevance evaluation, and query caching remain later work.

A later query pipeline should first select cheap deterministic candidates, then send bounded individual sections and the user's question to resident LLM Decide with explicit choices such as relevant, irrelevant or uncertain. Reuse one selected daemon/runtime and return the section ID, source hash, score and supporting excerpt. Do not concatenate the entire book into one request or treat scores from changing prompts as calibrated relevance.

Cache only with query/prompt version, section content hash and model/provider revision in the key. Bound per-section bytes/tokens and concurrency, measure the complete query runtime, and qualify relevance against synthetic queries. Preserve the user's question and explicit daemon choice; a classifier result does not authorize documentation edits or publication.

## Test contracts without private exports

Commit minimal synthetic ZIP/TAR fixtures with invented identities, URLs, captions and timestamps. Include duplicate exports, changed same-name members, distinct same-time events, multipart grouping, unknown fields, non-reel links, missing links, direction ambiguity, direct likes and separate message reactions. Verify deterministic plan ordering and complete provenance.

Exercise malformed JSON/HTML, unsupported schema versions, oversized or highly compressed metadata, traversal-style entry names, cancellation, failed downloads, partial outputs and restart behavior. A sparse large-file fixture and instrumented readers can verify seek/streaming behavior without a large committed archive. Use temporary application homes, fake downloader/transcriber/model adapters, and no credentials or network in the core checks.

Keep raw archives, message bodies, private identities, machine paths and real inventories out of the public repository, generated search index and copied assets. Derived indexes and transcripts remain private user data unless explicitly approved for their audience. Follow [publication and audiences](publication.md). Review both source and generated book output before publishing.

Remaining evidence work includes other export versions/languages and timestamp formats, complete watch-history availability, stable exporter/creator identities, multipart conventions, and real Instagram download/transcription qualification. Promote each capability only after its own fixture or end-to-end check establishes it.
