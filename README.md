# teamy-docs

A public mdBook for practical strategies: writing programs, playing games, CLI design, and repository hygiene. New Teamy CLIs use Rust and `teamy-rust-cli`; the book explains the template's decisions and capabilities through implementation references, including [logging](src/logging.md) and [template orientation](src/rust-cli-template.md).

Repository: [TeamDman/teamy-docs](https://github.com/TeamDman/teamy-docs). Website: [Teamy Docs](https://teamdman.github.io/teamy-docs/).

This repository contains unclassified documentation suitable for a public audience. Source, generated search data, print views and copied assets share that boundary. Keep private reviews, inventories and machine-specific records outside this repository.

## Build locally

The book pins mdBook 0.5.4, using the official Windows x86-64 release archive. PowerShell 7 and Git are the local prerequisites. Run these commands from this repository:

```powershell
pwsh -NoProfile -File ./scripts/bootstrap-mdbook.ps1
pwsh -NoProfile -File ./scripts/build.ps1
pwsh -NoProfile -File ./scripts/check-links.ps1
```

Bootstrap downloads into the ignored project-local `.tools/` directory. It checks the archive SHA-256 against the pinned value obtained from the [official release metadata](https://api.github.com/repos/rust-lang/mdBook/releases/tags/v0.5.4), extracts only `mdbook.exe`, and records a local binary receipt. It does not install a global tool or execute an installer. Review `tool-lock.json` and the script before use. The checksum confirms agreement with that release metadata; it is not an independent signature.

The build checks the binary receipt and version, then writes `book/`. Link validation checks generated local HTML/asset targets and fragment identifiers; it does not claim that external websites are available. Open `book/index.html` after a successful build.

On another platform, obtain the same pinned mdBook version from its [official release](https://github.com/rust-lang/mdBook/releases/tag/v0.5.4), verify the appropriate asset digest, and run `mdbook build`. The included bootstrap/build wrapper currently supports Windows x86-64 only.

## Edit the book

1. Add a Markdown chapter under `src/`.
2. Link it from `src/SUMMARY.md`.
3. Build and check links.

`create-missing = false` makes a missing summary target fail instead of creating an empty chapter. Keep examples synthetic or project-relative. Do not add raw inventories, private review comments, credentials, or machine-specific home/backup paths.

## GitHub Pages

The Pages workflow builds and checks the book on pushes to `main` and on manual dispatch. It uses the same pinned mdBook archive as the local build, then uploads only `book/` and deploys that artifact through the `github-pages` environment. Official GitHub Actions are pinned to immutable commits. Pages uses GitHub Actions as its publishing source.

Review the outgoing source and commit metadata before pushing: a push to `main` publishes the resulting book. See [publication and audiences](src/publication.md).

The current site uses `https://teamdman.github.io/teamy-docs/`. Connecting `docs.teamdman.ca` is a separate DNS follow-up. This repository has no custom-domain configuration or authenticated private documentation.

## License and tool provenance

Original repository material is licensed under MPL-2.0; see `LICENSE`. The locally downloaded mdBook executable is a separate third-party tool, licensed under MPL-2.0 by its maintainers. Its official release URL and archive digest are recorded in `tool-lock.json`; the executable/archive and local receipts are ignored.
