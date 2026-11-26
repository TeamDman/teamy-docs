# Track the Facet fork ecosystem

Use this page to plan a compatible dependency update. The behavioral guides cover [argument parsing](cli-parsing.md), [cancellation](cancellation.md) and [logging](logging.md).

**Release snapshot, 26 November 2026.** Registry publication, manifest pins, resolved lockfiles and passing consumer tests each establish a different stage of adoption. The 7 November observations below retain their original scope.

## Understand the dependency layers

- **Reflection and formats:** the [Teamy Facet fork](https://github.com/TeamDman/facet) supplies `teamy-facet` and companion reflection crates. The separate [format fork](https://github.com/TeamDman/facet-format) supplies JSON, CSV and other formats. Cargo aliases commonly keep the Rust names `facet` and `facet-json` while selecting `teamy-*` packages.
- **Argument parsing:** [Teamy Figue](https://github.com/TeamDman/figue) consumes reflection and format crates. It is a separate repository and package, not the historical Figue directory in the combined Facet Git revision.
- **Shared facilities:** cancellation optionally consumes Facet/Figue; tracing consumes Facet and JSON/format. Features matter: cancellation's async-only use does not activate its parser dependencies.
- **Transport and templates:** Vox has its own reflection/schema dependencies. The Rust CLI template combines Figue, formats, cancellation and tracing. Updating the template does not update existing applications.
- **Applications:** MFT combines the parser/cancellation path with Vox transport. Cloud Terrastodon and other CLIs have their own pins and validation obligations. This is a dependency graph, not a single linear release train.

## Branch direction

Use `teamy-main` for current Facet, Facet Format and Figue fork work.

**Long-term plan:** move the maintained fork work from `teamy-main` to `main`. This is the maintainer's intended direction, not a completed branch migration or a claim that the fork changes have been accepted upstream. Until that transition is explicitly carried out and verified, keep using `teamy-main`; do not assume an existing `main` branch contains the same work. Coordinate release automation, branch references and consumer documentation when making the transition.

## Published fork versions

All 18 package versions in the table are available on crates.io and passed Cargo's publish verification.

| Packages | Version |
| --- | --- |
| `teamy-facet-core`, `teamy-facet-path`, `teamy-facet-reflect`, `teamy-facet-pretty`, `teamy-facet-solver`, `teamy-facet-dessert`, `teamy-facet-dom` | `0.50.0-rc.9` |
| `teamy-facet`, `teamy-facet-default`, `teamy-facet-error` | `0.50.0-rc.11` |
| `teamy-facet-format`, `teamy-facet-value` | `0.50.0-rc.12` |
| `teamy-facet-json`, `teamy-facet-csv` | `0.50.0-rc.13` |
| `teamy-facet-xml`, `teamy-facet-xml-node` | `0.50.0-rc.10` |
| `teamy-figue` | `6.0.0-rc.9` |
| `teamy-figue-attrs` | `6.0.0-rc.7` |

The downloaded archives record clean working trees at the published release commits. [Facet `f570b6e`](https://github.com/TeamDman/facet/tree/f570b6e2eefe888b870242836ea40c13e08d5a5f), [Facet Format `1f4ea72`](https://github.com/TeamDman/facet-format/tree/1f4ea72eb05e178a8e80b3232022cf9f13f693a2), [Figue `080d354`](https://github.com/TeamDman/figue/tree/080d35448b8a5e0af3e2d0052f802b9f289760e5).

Enable the `jiff` feature on `teamy-facet` for Jiff 1.x reflection of `Timestamp`, `ZonedDateTime`, `civil::DateTime`, `civil::Date` and `civil::Time`. It replaces the Jiff 0.2 feature. The release tests used Jiff `1.0.3` and verified that the JSON decoder reads RFC 3339 timestamps with UTC and numeric offsets and preserves nanosecond precision. [Jiff type support](https://github.com/TeamDman/facet/blob/f570b6e2eefe888b870242836ea40c13e08d5a5f/docs/content/guide/type-support.md) · [JSON tests](https://github.com/TeamDman/facet-format/blob/1f4ea72eb05e178a8e80b3232022cf9f13f693a2/facet-json/tests/jiff_roundtrip.rs).

```toml
[dependencies]
facet = { package = "teamy-facet", version = "=0.50.0-rc.11", features = ["jiff"] }
facet-json = { package = "teamy-facet-json", version = "=0.50.0-rc.13" }
jiff = "1.0.3"
```

Resolve the updated companion crates together and check that the application uses one `teamy-facet-core` version. The published set supports registry dependencies without a local Facet patch.

## Consumer adoption

`teamy-cancellation 0.8.4` moves its optional reflection and CLI integration to Facet `0.50.0-rc.11` and Figue `6.0.0-rc.9`. The release passed strict Clippy and 90 all-feature tests on its pinned Rust 1.96 toolchain. [Crates.io](https://crates.io/crates/teamy-cancellation/0.8.4) · [Release source](https://github.com/TeamDman/teamy-cancellation/tree/0a71a7ba79929740197803c276e802efa340b74b).

`teamy-tracing 0.3.1` uses Facet `0.50.0-rc.11`, Format `0.50.0-rc.12` and JSON `0.50.0-rc.13`. Its Windows maintainer checks passed, and the registry checksum matched the verified archive. [Crates.io](https://crates.io/crates/teamy-tracing/0.3.1) · [Release source](https://github.com/TeamDman/teamy-tracing/tree/0e8b3b11d4fee4f26fec77154a458cc368dee2b5).

The Rust CLI template and the local Cloud Terrastodon cutover resolve the updated packages from crates.io. Both locked Windows all-feature graphs contain one version of each Teamy Facet and Figue package, use Jiff `1.0.3`, and contain no Chrono.

| Consumer | Local verification |
| --- | --- |
| Cloud Terrastodon | Nightly formatting, the locked offline workspace check with all features and targets, 22 focused tests, and `account login --help` from a fresh build passed. The workspace check emitted existing unused dependency warnings. |
| Rust CLI template | The locked check with all features and targets and all 11 library tests passed on Rust 1.96. A `--version` smoke test displayed the local timestamp and UTC offset. [Cutover commit](https://github.com/TeamDman/teamy-rust-cli/commit/3ffcb31011a15ab36ae6c1c215ba6bd0e2e53168). |

The earlier Vox, MFT and teamy-codex observations remain dated to the inspection below.

## 7 November 2026 snapshot

### Published fork versions on 7 November

These registry versions were observed on the snapshot date. Package versions are deliberately independent: **do not replace every `rc.7` with `rc.8`.**

| Package | Observed published version | Compatibility evidence |
| --- | --- | --- |
| `teamy-facet` | `0.50.0-rc.8` | [Fork release source](https://github.com/TeamDman/facet/tree/2d6d06e1d9a0ab47cd68547f5a0a4dbb60d36afa). |
| `teamy-facet-json`, `teamy-facet-format` | `0.50.0-rc.10` | Exact requirements of published Figue rc.6. |
| `teamy-figue` | `6.0.0-rc.6` | [Fork release source](https://github.com/TeamDman/figue/tree/85b3886592767bfe66cdf4588ee12d015efaf569); published manifest inspected. |
| `teamy-figue-attrs` | `6.0.0-rc.4` | Exact companion requirement of Figue rc.6. |
| `teamy-cancellation` | `0.8.1` | [Source](https://github.com/TeamDman/teamy-cancellation/tree/e4d962738dce985632bdff37a196d2c3e14500ac); published manifest optionally pins Facet rc.8 and Figue rc.6. |
| `teamy-tracing` | `0.2.1` | [Manifest](https://github.com/TeamDman/teamy-tracing/blob/47bc18c5738d62cdb471afee66e682e6c1ea851a/Cargo.toml) pins Facet rc.8 and JSON/format rc.10; development value crate also rc.10. |
| `teamy-vox` | `0.10.0-rc.9` | Published manifest still pins Facet/pretty/reflect rc.7. A compatible new release has not been verified. |

Figue rc.6's published manifest specifically requires Facet/error **rc.8**, core/pretty/reflect **rc.7**, format/JSON **rc.10**, and Figue attrs **rc.4**. This mixed set is intentional in the inspected package, not evidence of an incomplete update.

The upstream package `figue` was `5.0.0-rc.6` in the registry lookup; `teamy-figue` is a distinct fork package, not an upstream release with a higher compatible version. The upstream Facet latest release and complete fork-versus-upstream commit delta were **not** established by this audit. Consult the [upstream Facet](https://github.com/facet-rs/facet) and [upstream Figue](https://github.com/bearcove/figue) sources separately before claiming upstream parity.

### Consumer adoption on 7 November

| Consumer | Observed pins or state | What remains |
| --- | --- | --- |
| Rust CLI template, [public revision `58d9418`](https://github.com/TeamDman/teamy-rust-cli/blob/58d94181a3fc11d99c8691254dbffe8258dcc311/Cargo.toml) | Facet rc.8; JSON/CSV rc.10; pretty rc.7; Figue rc.6; cancellation 0.8.1; tracing 0.2.1. | Local merge `2c90b54` integrates this dependency update with the terminal/Windows foundation at `d3e304a`. Its full check script passes: nightly formatting, strict all-target/all-feature Clippy, 14 lint-policy fixtures, all-feature build and 45 tests. Root and leaf help checks pass. The local merge has not been pushed; existing copies still need adoption. |
| [Vox](https://github.com/TeamDman/vox) | Published Vox rc.9 remains on Facet rc.7. Its observed workspace also uses JSON/value rc.9 and Figue rc.5. | Update and qualify the affected Vox packages, then publish a mutually compatible set before retrying MFT. Migration is pending, not completed. |
| [MFT](https://github.com/TeamDman/teamy-mft) | Committed revision `54bd17a` retains Facet rc.7, JSON rc.9, Figue rc.5 and Vox/types/stream rc.9, with cancellation 0.8.1 using `async` and `io`, without `figue`. | The newer Facet/Figue trial failed dependency resolution against Vox's exact Facet rc.7 pin; those trial pins were restored. A compatible Vox release remains a prerequisite. This source revision is not a new published MFT release. |
| Cloud Terrastodon, [public revision `fd989bd`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/Cargo.toml) | Facet/reflect/pretty rc.7; JSON/value rc.9; Figue rc.5; cancellation 0.7.0 with Facet enabled. | Older compatible set remains declared. No upgrade or build qualification was performed in this audit. |
| [teamy-codex](https://github.com/TeamDman/teamy-codex) | Inspected manifest retains legacy combined Facet/Figue Git revision `23ad4dcd0`, cancellation Git revision `7c77848`, optional tracing Git revision `207658ceacef0a756bec704a05d6cc1db3536f20`, and optional owned JSON rc.9. | Mixed legacy and published-fork adoption. Do not interpret optional telemetry adoption as a complete parser migration. |

The MFT working tree examined on 7 November passed its full check script: nightly formatting, strict all-target/all-feature Clippy, the all-feature build and 312 tests, with three existing tests ignored. Default-feature tests also passed. These results qualify the retained Facet rc.7, JSON rc.9, Figue rc.5 and Vox rc.9 graph; they do not validate the rejected newer Facet/Figue combination or constitute a published MFT release. The working tree also repairs index-log correlation across the Rayon dispatch boundary; issue #14's restart-sensitive behavior remains unproven.

This is a curated adoption map, not a complete repository inventory. Projects not listed have **unknown** migration status until their own manifests, resolved dependencies and tests are checked.

### Format release history on 7 November

The desktop format pull was reconciled in local merge `03941a0`. Its pre-release tree was identical to the incoming release commit's parent; rewritten commit metadata had changed ancestry and produced conflicts without independent source changes. The resolved tree matches [release preparation `55dd746`](https://github.com/TeamDman/facet-format/commit/55dd746345382b81a8e9eb1652cddcd4b7cb2fda), retaining the Cow fix and the previously merged XML work.

The affected Format, JSON, CSV, Value, DOM, XML and XML-node library, integration and documentation suites pass with the locked graph. The registry already contains Format/JSON/CSV/Value rc.10, DOM rc.7 and XML/XML-node rc.8, all observed unyanked on 7 November. The published archives' 420 source files match the reconciled checkout after line-ending normalization. No additional format release is needed for these changes. Source qualification and archive comparison do not constitute a new publication; neither this local merge nor the template integration was pushed by this work.

## Update in dependency order

1. Record registry packages, exact source revisions, enabled features and resolved lockfile versions. Distinguish upstream packages from renamed fork packages.
2. Qualify reflection and formats together, then Figue. Preserve exact companion requirements rather than guessing a uniform release suffix.
3. Qualify cancellation with the features each consumer uses and tracing with its output/telemetry features.
4. Update Vox's affected schema, macro, core and transport packages as a compatible set. Check generated-code and protocol behavior, not only the top-level manifest. Publish only after the required checks pass.
5. Retry MFT using the released compatible set; run its full checks and distinguish pre-existing failures from regressions.
6. Adopt the template changes in each consumer deliberately. Reconcile independent application work rather than overwriting it with template files.
7. Update this page with a new as-of date, revision links, validation results and remaining unknowns. A successful dependency resolution is not proof of runtime compatibility or publication.

Use [migration workflows](migration-workflows.md) for the implementation discipline and [publication boundaries](publication.md) before sharing results. Keep machine paths, private inventories and diagnostic captures out of this public record.
