# Work across Codex cloud environments

Reviewed **2026-10-23**. Use a dedicated `teamy-docs` environment to maintain this book when a desktop is unavailable. Keep the reusable setup, a task's working files, Git history and delivered artifacts distinct. The sections below separate official documentation, Teamy observations and our working policy; proposed transfer tooling lives in [Plan file movement](moving-files.md#proposed-cloud-to-desktop-transfer).

## Officially documented lifecycle

An environment combines prepared repositories, dependencies, tools and access settings. Publishing captures its filesystem for isolated new tasks. Existing tasks retain their own files and installed tools. Background repository refresh preserves dependency caches without rerunning setup commands; republished setup is used by new tasks. By default, saved VM state is recoverable for **up to seven days after the last turn start or task resume**. This is a recovery window, not permanent backup. Source: [OpenAI's cloud-environment documentation](https://learn.chatgpt.com/docs/environments/cloud-environments).

## Observed setup and revision boundaries

During Teamy setup reviewed on 2026-10-23, the environment UI saved a repository selection as a full HEAD SHA. The dedicated docs task was independently checked at `50e89d6897ce2a65167ef65b17c5fa83fe849158`, matching its requested [public baseline](https://github.com/TeamDman/teamy-docs/commit/50e89d6897ce2a65167ef65b17c5fa83fe849158). Record the actual checkout rather than treating an environment name or branch label as revision evidence.

The documented background refresh does not establish a verified guarantee that every task starts at the latest tip of a chosen branch. Task feature branches do not automatically merge into the default branch or into the saved setup. Publishing an environment and publishing repository changes are separate operations.

Newly published network settings were not observed to change an already-running task. Verify a fresh task before concluding that the new configuration failed. Preserve the old task's files while doing so; recreating a task is not a file-transfer operation. See [networking boundaries](networking-survey.md#codex-cloud-private-networking).

## Teamy working policy

Every new Teamy coding environment should include a reference checkout of [TeamDman/teamy-docs](https://github.com/TeamDman/teamy-docs), alongside the project being changed. Read its `AGENTS.md`, [TeamyStyle](teamystyle.md), [Rust CLI template](rust-cli-template.md), [code standards](code-standards.md) and relevant problem guides. New first-party Rust code follows the Facet/Figue conventions. A documented local template improvement is not necessarily present in an older public checkout: inspect the actual source and full SHA in each repository.

Use standard, non-fast tasks by default unless the work calls for a different explicit choice. Prefer meaningful local or cloud checks of the affected behavior to routine CI churn. Record which checks passed, failed or were not run and why. CI validation, releases, Pages deployment and security automation serve different purposes; this preference is not a blanket instruction to disable them. Keep this book's existing [Pages publishing workflow](publication.md#publish-with-github-pages).

### Select and test an exact revision

Start by inspecting state before fetching or changing branches:

```sh
git status --short --branch
git rev-parse HEAD
git log -1 --oneline
```

Preserve dirty work in its existing branch/worktree or an independently saved copy before changing the checkout. Review untracked and ignored files too; see [repository hygiene](repository-hygiene.md). Do not use a blind hard reset or cleanup to make setup look reproducible.

In a clean checkout of `TeamDman/teamy-docs`, this example fetches and explicitly selects the remote `main` snapshot for a new feature branch. Choose an unused branch name:

```sh
git fetch origin
git rev-parse origin/main
git switch --no-track -c docs/cloud-workflow origin/main
git rev-parse HEAD
```

Fetch updates remote-tracking references; it does not merge the current branch. Review the selected commit, make scoped changes, run the existing build/link checks and commit. Record the full resulting SHA and confirm a clean working tree. Publish an environment from the exact tested commit. If a later fetch, merge or edit changes the candidate, validate that resulting state before publishing it. Repository publication separately needs the intended branch, reviewed outgoing content and authorization.

### Retain code and artifacts independently

GitHub is the shared source of truth for desktop/cloud code exchange: push an approved branch, then fetch and explicitly select it on the other side. Desktop and cloud tasks do not share a working tree. A local commit is useful history but is not a remote backup until it is pushed and verified there.

Renders and build outputs are normally ignored and need their own retention decision. Keep valuable artifacts independently of the task VM and record enough provenance to identify their source revision, generation command and content hash. An ignored file is not automatically disposable. A [producing-chat file link](making-videos.md#view-and-deliver-a-cloud-render) is useful for iteration; a Library upload is a separate retention/delivery option. Neither a task link nor Library should be described as a promise of indefinite retention.
