# Track the Facet fork ecosystem

Use this page to plan a compatible dependency update, not to replace the behavioral guides for [argument parsing](cli-parsing.md), [cancellation](cancellation.md) or [logging](logging.md).

**Snapshot: 7 November 2026.** Registry availability, a repository's declared pins, its lockfile, and a tested consumer are different facts. The table records the evidence actually checked; it is not an automatic claim that every project is current. Older chapters retain their revision-specific implementation evidence.

## Understand the dependency layers

- **Reflection and formats:** the [Teamy Facet fork](https://github.com/TeamDman/facet) supplies `teamy-facet` and companion reflection crates. The separate [format fork](https://github.com/TeamDman/facet-format) supplies JSON, CSV and other formats. Cargo aliases commonly keep the Rust names `facet` and `facet-json` while selecting `teamy-*` packages.
- **Argument parsing:** [Teamy Figue](https://github.com/TeamDman/figue) consumes reflection and format crates. It is a separate repository and package, not the historical Figue directory in the combined Facet Git revision.
- **Shared facilities:** cancellation optionally consumes Facet/Figue; tracing consumes Facet and JSON/format. Features matter: cancellation's async-only use does not activate its parser dependencies.
- **Transport and templates:** Vox has its own reflection/schema dependencies. The Rust CLI template combines Figue, formats, cancellation and tracing. Updating the template does not update existing applications.
- **Applications:** MFT combines the parser/cancellation path with Vox transport. Cloud Terrastodon and other CLIs have their own pins and validation obligations. This is a dependency graph, not a single linear release train.

## Branch direction

Use `teamy-main` for current Facet ecosystem fork work. The inspected Facet and Figue release revisions listed below are on their respective `teamy-main` branches.

**Long-term plan:** move the maintained fork work from `teamy-main` to `main`. This is the maintainer's intended direction, not a completed branch migration or a claim that the fork changes have been accepted upstream. Until that transition is explicitly carried out and verified, keep using `teamy-main`; do not assume an existing `main` branch contains the same work. Coordinate release automation, branch references and consumer documentation when making the transition.

## Published fork versions

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

## Consumer adoption

| Consumer | Observed pins or state | What remains |
| --- | --- | --- |
| Rust CLI template, [public revision `58d9418`](https://github.com/TeamDman/teamy-rust-cli/blob/58d94181a3fc11d99c8691254dbffe8258dcc311/Cargo.toml) | Facet rc.8; JSON/CSV rc.10; pretty rc.7; Figue rc.6; cancellation 0.8.1; tracing 0.2.1. | Public manifest is updated. This audit did not run the template's full qualification suite; existing copies still need adoption. |
| [Vox](https://github.com/TeamDman/vox) | Published Vox rc.9 remains on Facet rc.7. Its observed workspace also uses JSON/value rc.9 and Figue rc.5. | Update and qualify the affected Vox packages, then publish a mutually compatible set before retrying MFT. Migration is pending, not completed. |
| [MFT](https://github.com/TeamDman/teamy-mft) | Working migration retains Facet rc.7, JSON rc.9, Figue rc.5 and Vox/types/stream rc.9. Cancellation alone advances to 0.8.1 with `async`, without `figue`. | This cancellation change is working-tree evidence, not a verified published release. The newer Facet/Figue trial failed dependency resolution against Vox's exact Facet rc.7 pin; those trial pins were restored. |
| Cloud Terrastodon, [public revision `fd989bd`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/fd989bd357cbfc5d60ad06aff2a7677a3a46491f/Cargo.toml) | Facet/reflect/pretty rc.7; JSON/value rc.9; Figue rc.5; cancellation 0.7.0 with Facet enabled. | Older compatible set remains declared. No upgrade or build qualification was performed in this audit. |
| [teamy-codex](https://github.com/TeamDman/teamy-codex) | Inspected manifest retains legacy combined Facet/Figue Git revision `23ad4dcd0`, cancellation Git revision `7c77848`, optional tracing Git revision `207658ceacef0a756bec704a05d6cc1db3536f20`, and optional owned JSON rc.9. | Mixed legacy and published-fork adoption. Do not interpret optional telemetry adoption as a complete parser migration. |

The MFT cancellation-only validation passed all-targets checking, strict all-feature Clippy, an all-feature build, 244 library tests plus 18 binary/integration tests, and seven Tracey combinations. Four tests were ignored. Existing cancellation-example doctest failures were observed before this upgrade and remain unresolved; this is not an all-tests-pass claim. These results do not validate the rejected newer Facet/Figue combination.

This is a curated adoption map, not a complete repository inventory. Projects not listed have **unknown** migration status until their own manifests, resolved dependencies and tests are checked.

## Update in dependency order

1. Record registry packages, exact source revisions, enabled features and resolved lockfile versions. Distinguish upstream packages from renamed fork packages.
2. Qualify reflection and formats together, then Figue. Preserve exact companion requirements rather than guessing a uniform release suffix.
3. Qualify cancellation with the features each consumer uses and tracing with its output/telemetry features.
4. Update Vox's affected schema, macro, core and transport packages as a compatible set. Check generated-code and protocol behavior, not only the top-level manifest. Publish only after the required checks pass.
5. Retry MFT using the released compatible set; run its full checks and distinguish pre-existing failures from regressions.
6. Adopt the template changes in each consumer deliberately. Reconcile independent application work rather than overwriting it with template files.
7. Update this page with a new as-of date, revision links, validation results and remaining unknowns. A successful dependency resolution is not proof of runtime compatibility or publication.

Use [migration workflows](migration-workflows.md) for the implementation discipline and [publication boundaries](publication.md) before sharing results. Keep machine paths, private inventories and diagnostic captures out of this public record.
