# Show a native Windows message box

Use Piing as prior art for a native configuration-error dialog with several recovery choices. It uses `TaskDialogIndirect` and enables Common-Controls v6 in its executable manifest. It can fall back to `MessageBoxW`.

## Choose the dialog for the interaction

| Need | Windows API | Our prior art |
| --- | --- | --- |
| A short message with standard buttons | `MessageBoxW` | tb and Piing's fallback. |
| An instruction with custom recovery buttons | `TaskDialogIndirect` | Piing's configuration-error helper. |

Microsoft documents the [basic message box](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messageboxw) and [task dialog](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-taskdialogindirect). `MessageBoxW` uses User32. `TaskDialogIndirect` requires Comctl32 version 6; adding the Rust API import alone does not select that DLL version.

## Activate Common-Controls v6 in the manifest

The inspected Rust CLI template contains a commented example of the v6 dependency. A comment does not activate it. Piing's manifest has an active dependency. Adopt that setting when using its task-dialog implementation and rebuild the executable.

Grounding at Piing revision `d2c5065`: [active manifest](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/resources/piing.exe.manifest#L9-L20), [manifest resource](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/resources/app.rc#L2) and [resource build](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/build.rs#L4-L6). Compare the template's [commented dependency at `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/resources/app.manifest#L9-L22).

The dependency belongs inside the manifest's existing `assembly` element:

```xml
<dependency>
    <dependentAssembly>
        <assemblyIdentity
            type="win32"
            name="Microsoft.Windows.Common-Controls"
            version="6.0.0.0"
            processorArchitecture="*"
            publicKeyToken="6595b64144ccf1df"
            language="*"
        />
    </dependentAssembly>
</dependency>
```

Microsoft's [visual styles guidance](https://learn.microsoft.com/en-us/windows/win32/controls/cookbook-overview) explains selecting the v6 assembly through a manifest. Enabling a `windows` crate feature exposes bindings; it does not activate this dependency. The template's `.manifest_required()` checks resource compilation, not the presence of this XML element.

The manifest is embedded through the `.rc` resource script; [executable icons and resources](executable-icons.md#embed-resources-once) explains that build path. [Windows path length](windows-paths.md#the-rust-cli-templates-manifest) describes a separate manifest setting, `longPathAware`.

After building, inspect the embedded manifest rather than relying only on the source file. With the Windows SDK manifest tool available, this example extracts resource 1 into a separate XML report without launching or modifying the executable:

```powershell
& mt.exe '-inputresource:.\target\release\example.exe;#1' `
    '-out:.\manifest-review.xml'
```

Replace `example.exe` with the executable being reviewed and choose the report destination deliberately. See [Microsoft's manifest tool](https://learn.microsoft.com/en-us/windows/win32/sbscs/mt-exe). Inspecting the XML and testing the visible dialog are separate checks.

## Return a choice before performing its action

Piing's [configuration-error helper](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/ui/dialogs.rs#L81-L158) builds an instruction, message and five custom buttons. It returns a `ConfigDialogChoice`. The [outer recovery loop](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/ui/dialogs.rs#L40-L73) handles the selected action after the dialog closes:

| Choice | Recovery behavior |
| --- | --- |
| Reload now | Retry the configuration operation. |
| Copy message text, Open home directory or Show logs | Perform the chosen action, then display the dialog again. |
| OK or dialog cancellation | Return the original configuration error. |

This keeps recovery work outside a task-dialog callback. The helper does not use callbacks, expandable details or application artwork. Its Escape, Alt+F4 and close behavior maps to the same outcome as OK; that is Piing's policy, not an inherent API requirement.

Copy uses the full formatted error message. Open home directory launches Explorer. Show logs creates a console and replays the retained log buffer; see [Piing's logging prior art](logging.md#learn-from-piing-and-mft). Copy and launch failures are logged before the dialog is displayed again.

tb provides a smaller [MessageBoxW example](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs#L366-L392): Yes copies version information and No dismisses the message.

## Retain strings and account for modal dispatch

Piing uses the published `teamy-windows` 0.7.0 [`PCWSTRGuard`](https://github.com/TeamDman/teamy-rust-windows-utils/blob/ff7b7b2571a0a455164fcea42118d2cbfa2ef102/src/string/pcwstr_guard.rs#L6) to own UTF-16 text. Named guards, the button array and the configuration stay alive throughout the synchronous call. Preserve those lifetimes when refactoring: a borrowed pointer must not outlive its string allocation.

[Startup has no owner window](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/runtime.rs#L25), while [tray reload supplies its HWND](https://github.com/AAFC-Cloud/piing/blob/d2c506520cb6dc8b1d51ba2b2cd0b7681268e6f9/src/tray/window_proc.rs#L186-L191). That owner is a [hidden tray window](https://github.com/TeamDman/teamy-rust-windows-utils/blob/ff7b7b2571a0a455164fcea42118d2cbfa2ef102/src/window/create_window_for_tray.rs#L10-L48). Microsoft's [task-dialog guidance](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-taskdialogindirect) says the parent should not be hidden or disabled. Review that mismatch and choose ownership for the actual interaction when adopting the helper.

A modal call waits for the user while dispatching Windows messages. Account for reentrant callbacks and shutdown. Piing's raw mutable tray-state borrow needs a separate reentrancy review before treating it as a safe general pattern. See [modal-loop shutdown behavior](https://devblogs.microsoft.com/oldnewthing/20101008-01/?p=12573) and [window event ownership](window-creation.md).

## Preserve native errors when adopting the helper

The current Piing helper tests `TaskDialogIndirect(...).is_ok()`, discarding the failure HRESULT. Its fallback ignores the `MessageBoxW` return value, then returns the OK choice. That loses native error information. tb also treats unexpected message-box results only as debug events.

Preserve the task-dialog failure when reporting a fallback, and handle the fallback's zero result as an API failure. Keep successful choices typed and propagate unrecoverable errors through `eyre`; see [CLI output shapes](output-shape.md). A user cancellation and a failed attempt to display the dialog need distinguishable outcomes. Verify dismissal, recovery actions and shutdown when adapting this prior art.

## Reuse application artwork deliberately

A task dialog can use an existing icon handle through `hMainIcon` with `TDF_USE_HICON_MAIN`. [Executable icons](executable-icons.md) explains loading the embedded resource without another `include_bytes!` embedding. Keep the handle valid until the dialog closes and release it according to its creator.

This is a documented extension to Piing's current helper, not a feature it already implements. See [Microsoft's task-dialog configuration](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/ns-commctrl-taskdialogconfig). Richer task-dialog capabilities still need their own interaction, lifetime and outcome design.
