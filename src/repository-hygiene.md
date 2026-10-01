# Repository hygiene and scratch

An ignored file is outside normal Git tracking; that does not establish that it is expendable. Build output, credentials, local configuration, original assets and temporary experiments can all be configured to be ignored. They may still retain value and need review before pruning.

For new work, reserve `target/` and other directories managed by build cleanup for outputs you can discard and reproduce. Store experiments, review notes and outputs you need to retain in a separate work directory. Existing contents of a build directory still need review; its name does not prove they are disposable.

## Preview cleanup before selecting it

The compact preview is `git clean --dry-run -ffdx`. Expand the available long forms for readability:

```powershell
git clean --dry-run --force --force -d -x
```

| Short form | Long form | Meaning |
| --- | --- | --- |
| `-n` | `--dry-run` | Show candidates without removing them. |
| `-ff` | `--force --force` | Repeat force to include untracked nested Git repositories. |
| `-d` | No documented long form | Include untracked directories when no pathspec is supplied. |
| `-x` | No documented long form | Include ignored files; explicit `--exclude` rules still apply. |

Treat nested repositories and worktrees as separate objects to review. These examples preview cleanup; they do not authorize executing it.

Cargo documents `cargo clean --dry-run --verbose`: preview without deleting, with verbose listing candidate files. Default cleaning targets the entire effective target directory, including scratch stored there. Resolve `CARGO_TARGET_DIR`, configuration and explicit target flags first; that directory can be shared or outside the checkout.

Sources: [Git clean](https://git-scm.com/docs/git-clean), [Cargo clean](https://doc.rust-lang.org/cargo/commands/cargo-clean.html).

## Give scratch a lifecycle

Use a dedicated work root outside the effective Cargo target directory and other build cleanup roots. A synthetic layout is:

```text
<work-root>/sessions/<session-id>/manifest.json
<ledger-root>/scratch-sessions/<session-id>.json
```

A repo-local `.work/` directory is one option. Give it its own ignore rule, with its purpose stated:

```gitignore
# Local work to retain across build cleanup.
/.work/
```

Ignoring `.work/` prevents normal Git tracking; `git clean -x` still includes it. Keep retained work outside the Git cleanup scope or explicitly exclude it from the reviewed command. For `.work/` directly inside the repository root, this preview adds an exclusion:

```powershell
git clean --dry-run --force --force -d -x --exclude=.work/
```

A versioned manifest records creator/session, purpose, creation time, outputs to retain and disposition. Mark the disposition explicitly as `active`, `retained`, `disposable` or `unknown`, with a reason. The directory name and ignore rule do not supply that decision. A minimal ledger outside the cleanup candidates preserves the ownership record if the scratch folder is removed. Both inherit appropriate audience restrictions.

Treat `active`, `retained` and `unknown` sessions as material to preserve. A `disposable` declaration is evidence for review, not permission for an agent to delete it. Age and a missing process are insufficient proof of disposability. Preserve needed outputs before approving an explicit cleanup selection. Existing scratch without trustworthy provenance remains unknown.

## Measure what the report claims

Logical bytes, allocated bytes and estimated reclaimable bytes answer different questions. Deduplicate hardlinks by file identity, declare stream coverage, avoid following reparse points by default and keep unsupported measurements unknown. Report provider, scope, capture time and incomplete items.

An index is an observation, not the current filesystem. A live sweep also spans time. Verify the exact objects again before applying an approved operation.
