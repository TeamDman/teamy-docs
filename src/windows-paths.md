# Windows path length

Keep frequently used repository roots short enough to leave room for nested dependencies and generated files. Score the full expected path, not just the root name.

Many legacy Win32 APIs use `MAX_PATH` (260 characters including the terminator). Some Unicode APIs accept extended-length paths, with an approximate total limit of 32,767 characters; per-component limits remain. The `\\?\` form changes parsing rules and requires absolute paths.

Windows 10 version 1607 and later can remove `MAX_PATH` limits for many common functions, but the documented opt-in requires both the system setting and the application's `longPathAware` manifest. One setting does not make every application compatible. Shell interfaces and filesystem APIs can have different capabilities.

Before a reorganization, measure the longest expected destination, check the tools that will operate on it, and retain margin for generated children. Test Unicode, spaces, long components and supported long paths on an approved fixture. A path that can be stored may still fail in an editor, archive tool or dependency.

This guidance does not enable a registry setting or change Git configuration.

Source: [Microsoft: maximum path length limitation](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation).

## The Rust CLI template's manifest

The inspected `teamy-rust-cli` template embeds a manifest, but it does not declare `longPathAware`. Its `resources/app.manifest` contains an assembly identity and a commented Common-Controls dependency. There is no `application/windowsSettings/longPathAware` element.

`resources/app.rc` embeds that file as manifest resource 1. `build.rs` compiles the resource with `embed-resource` and calls `.manifest_required()`. That call makes resource compilation failure a build error on Windows; it does not add settings to the XML.

The inspected locator and local mover manifests have the same contents. Each project owns its copied manifest, so updating the template would require a separate adoption step for existing projects.

Sources: [template manifest at revision `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/resources/app.manifest), [template resource build](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/build.rs), [locked embed-resource 3.0.11 result handling](https://docs.rs/embed-resource/3.0.11/src/embed_resource/lib.rs.html#350-358).

The same resource script embeds [the executable icon](executable-icons.md). Piing's [native task dialogs](native-message-boxes.md#activate-common-controls-v6-in-the-manifest) use an active Common-Controls v6 dependency. Icon resources, Common-Controls activation and long-path awareness are separate declarations.

### Proposed template follow-up

Add Microsoft's application element inside the existing top-level `assembly` element:

```xml
<application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings xmlns:ws2="http://schemas.microsoft.com/SMI/2016/WindowsSettings">
        <ws2:longPathAware>true</ws2:longPathAware>
    </windowsSettings>
</application>
```

Rebuild the Windows executable and inspect its embedded manifest. Test the actual filesystem operations with a synthetic path longer than 260 characters, including Unicode and spaces. Record the system's `LongPathsEnabled` value without changing it.

The documented Win32 opt-in requires both this manifest setting and the system setting. Extended-length paths provide another route for supported APIs. The missing manifest element alone does not establish whether every Rust filesystem operation will fail. Test native dependencies and external tools separately.

This section records inspected source and a proposed follow-up. The template and application manifests have not been changed.

## Git for Windows is a separate layer

Git for Windows documents refusing long checkouts by default, with `core.longpaths` as its override. Test one invocation without persisting a setting:

```powershell
git -c core.longpaths=true status --short
```

For an explicitly approved future clone, its documented clone-local form is `git clone -c core.longpaths=true "<repository-url>" "<destination>"`. It writes the setting in the new repository. Neither form changes the Windows policy or another application's capabilities.

An operator who deliberately chooses a standard for their own Git account can set:

```powershell
git config --global core.longpaths true
```

These are documentation examples; the book build runs none of them. `--global` is user-specific; `--system` changes the Git installation's system scope and can affect other users. `--local` affects one repository. Later scopes can override earlier values. `safe.directory` concerns repository ownership/trust, and does not enable long paths.

Sources: [Git for Windows release notes: long-path limitation](https://github.com/git-for-windows/build-extra/blob/main/ReleaseNotes.md?plain=1), [Git configuration scopes and safe.directory](https://git-scm.com/docs/git-config), [Git command-scoped configuration](https://git-scm.com/docs/git).

## If a checkout stopped halfway

1. Inspect the checkout status and capture the current state before repair; later edits may already exist.
2. Identify the specific missing tracked paths and the long-path setting/tool that caused the failure.
3. Review a narrowly scoped repair with the operator, preserve later edits, and verify the repaired paths afterward.

Do not use a broad restore over later work merely because the original clone was incomplete. A repair or new clone requires its own applicable authorization.
