# Find source before downloading

Look for an existing approved checkout before proposing a download. Match repository identity and the revision relevant to the question; a similarly named folder is insufficient.

Our prior art is [locate-git-projects-on-my-computer](https://github.com/TeamDman/locate-git-projects-on-my-computer), which combines indexed filesystem markers, Git metadata and Cargo package metadata. This topic is the entry point for “I want the source for this library.” The repository is an implementation reference, not a category the reader must know in advance.

The public baseline inspected on 1 October 2026 is [`d02d04d`](https://github.com/TeamDman/locate-git-projects-on-my-computer/tree/d02d04d738a8922e7e40f87eb4127727442e6eb8). Its [discovery implementation](https://github.com/TeamDman/locate-git-projects-on-my-computer/blob/d02d04d738a8922e7e40f87eb4127727442e6eb8/src/discovery.rs) shows indexed discovery and enrichment. The catalogue, Cargo-cache selection and clone-location proposals described below are later local changes, not features of that public snapshot.

## Inspect a Rust dependency

Read the package's locked source/version or Git revision, then check whether that implementation is already cached. Resolve `CARGO_HOME` for the current invocation; its default is `.cargo` under the runtime user's home. Do not assume a particular person's home path, and never read `credentials.toml` for source lookup.

Cargo keeps unpacked registry sources under `registry/src` and Git dependency checkouts under `git/checkouts`. Match the locked package version/revision and registry source; multiple versions or revisions may coexist. The internal layout is not stable, so an adapter must report unsupported layouts rather than guess.

Cached dependency sources are not necessarily independent Git repositories or projects authored by the local user. Include them only when implementation inspection is intended. Cargo cleaning removes target artifacts, not this download/source cache.

Sources: [Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html), [Cargo environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html), [Cargo clean](https://doc.rust-lang.org/cargo/commands/cargo-clean.html).

## Local locator prototype

The local changes are built in the locator checkout. They have not been installed into `PATH` or published as a release. An installed executable can therefore lack `--catalogue`, `--include-cargo-cache` and `source propose` even when these examples work from the checkout. Check which executable a bare command name selects:

```powershell
Get-Command locate-git-projects-on-my-computer |
    Select-Object -ExpandProperty Source
locate-git-projects-on-my-computer --version
locate-git-projects-on-my-computer --help
```

Compare the revision and build time as well as the package version. Different builds can currently share version `0.2.0`.

### What `--catalogue` means

`--catalogue` selects a richer JSON report on standard output. Without it, discovery returns the existing array of project records. With it, discovery returns a version 1 object:

| Field | Meaning |
| --- | --- |
| `version` | The catalogue schema version, currently `1`. |
| `generated_at` | When this report was assembled. |
| `provenance` | Executable version/revision/build time and source-selection settings. |
| `coverage` | Discovery backend, completeness/freshness limits and other unknowns. |
| `projects` | Project records with marker evidence, source kind, opened Git topology, repository hints and unknowns. |

The flag selects the output shape; it does not refresh the MFT index or save a persistent catalogue by itself. Index freshness and completeness remain unknown. Nested packages can share a checkout, so a project count is not a repository count. Repository hints do not establish visibility or a publication audience. Review the report before sharing it.

### Run the checkout's command surface

Replace `<locator-checkout>` with the approved absolute path to the locator repository. `cargo run --` builds and runs that checkout's source, leaving the installed PATH executable in place. Compilation can obtain dependencies if they are not cached. Start with help, which does not run discovery:

```powershell
Set-Location "<locator-checkout>"
cargo run -- --help
cargo run -- source propose --help
```

The discovery commands below read the published Teamy MFT index and enrich discovered projects. `--name` filters the results; it is not a filesystem scope boundary. Ordinary discovery excludes the resolved Cargo home. Include cached dependency source explicitly when you want to inspect its implementation:

```powershell
cargo run -- --catalogue --name facet
cargo run -- --catalogue --name facet `
    --include-cargo-cache --cargo-home "<absolute-cargo-home>"
```

Replace `<absolute-cargo-home>` with the approved cache path. `--cargo-home` overrides source selection for this invocation only; omit it to use the runtime-resolved home.

To propose a destination, replace `<absolute-clone-root>` with an approved root:

```powershell
cargo run -- source propose `
    --repo-url https://github.com/example/project.git `
    --clone-root "<absolute-clone-root>"
```

`source propose` emits versioned inert proposal data. It does not scan, contact the remote or clone the repository.

If source is missing, produce an inert proposal with repository URL, unknown or verified visibility, proposed clone root and reason. Review its source/maintainer and destination before authorizing a clone. Finding a repository hint never authorizes downloading or executing it.
