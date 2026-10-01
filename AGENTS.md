# Documentation boundary

This repository and its generated GitHub Pages site contain only unclassified material approved for a public audience.

- Keep private user reviews, raw conversation exports, repository inventories, credentials and machine-specific records outside this repository.
- Use project-relative paths or clearly synthetic placeholders in examples. Public repository coordinates and the author's public pseudonym are appropriate.
- Review every file under `src/`: mdBook also copies assets that are not linked from `SUMMARY.md`.
- Run `scripts/build.ps1` and `scripts/check-links.ps1` after changing the book. The pinned tool bootstrap is `scripts/bootstrap-mdbook.ps1`.
- Pushes to `main` deploy the public site. Review outgoing source and commit metadata before publishing.

The site currently uses GitHub's default project address. Custom-domain DNS and private-document hosting are separate tasks.
