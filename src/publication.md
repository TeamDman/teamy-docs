# Publication and audiences

`teamy-docs` contains unclassified documentation suitable for public distribution. Its source and GitHub Pages website are public. Apply that boundary to every chapter, generated search record, print view and copied asset.

## Public source boundary

This repository is for generic guidance and synthetic examples. Keep private review comments, inventories, credentials and machine-specific paths in separate local/private records. Unclassified describes the classification; public describes the audience. Information must meet both requirements to enter this book.

GitHub warns that an ordinary Pages site can be publicly accessible even when its source repository is private. Private Pages access is a separate Enterprise Cloud organization feature. Do not infer account eligibility from the repository name or source visibility.

Sources: [Creating a GitHub Pages site](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site), [Changing Pages visibility](https://docs.github.com/en/enterprise-cloud@latest/pages/getting-started-with-github-pages/changing-the-visibility-of-your-github-pages-site).

## Publish with GitHub Pages

The public source is [TeamDman/teamy-docs](https://github.com/TeamDman/teamy-docs). The static site is [Teamy Docs](https://teamdman.github.io/teamy-docs/).

The deployment workflow runs on a push to `main` or manual dispatch. Its build job verifies the pinned mdBook archive, builds the book and checks local links. A separate deployment job publishes only the generated `book/` artifact. Repository source, downloaded tools and local review records are not part of that site artifact.

Review the source and outgoing commit metadata before pushing. Changes pushed to `main` are published automatically. The site has no sign-in or private-documents area.

The default address uses `/teamy-docs/`; `book.toml` records that path so generated 404-page asset links work there. Connecting `docs.teamdman.ca` remains a separate DNS task.

Source: [GitHub Pages custom workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).

## Private documentation belongs elsewhere

Private documentation needs separate permitted inputs and separate generated output, outside `teamy-docs`. Build only what its target audience may read; do not build a mixed book and attempt to redact it afterward.

Future authenticated hosting must protect every asset and access path, including search data and default hostnames. Signing in proves an identity; an audience policy still determines whether that identity may read the material. Choose and verify that hosting separately from the static-book authoring workflow.

GitHub Pages serves the public book. Identity providers, private hosting, DNS and other cloud resources require their own design and setup.

## Review what is actually shared

Before publication, review source, generated output and outgoing commit metadata. Keep the workflow insight while replacing incidental machine details with project-relative paths or clearly synthetic placeholders. Review licensing and attribution for copied material.

mdBook copies non-Markdown source files into the output directory, so adding a local file under `src/` can publish it even without a chapter link. Its configuration can reject missing chapters with `create-missing = false`.

Sources: [mdBook build behavior](https://rust-lang.github.io/mdBook/cli/build.html), [mdBook build configuration](https://rust-lang.github.io/mdBook/format/configuration/general.html).
