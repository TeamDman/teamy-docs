# Reuse or extract an executable's icon

Use the icon already embedded in a Windows executable when the interface needs the same artwork. The Rust CLI template embeds `resources/main.ico`; tb loads that resource for its tray. The same artwork can also be assigned to a window explicitly. This avoids another `include_bytes!` embedding for native consumers.

## Embed resources once

The template's resource script declares the icon and manifest separately:

```rc
#define RT_MANIFEST 24
1 RT_MANIFEST "app.manifest"
main_icon ICON "main.ico"
```

`main_icon` names the icon resource. `1` identifies the manifest resource, not the icon. The `.rc` file embeds both; the XML manifest configures Windows behavior. An executable icon does not come from a `longPathAware` or Common-Controls setting.

Grounding: [template resource script at `7e62d72`](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/resources/app.rc) and [resource build](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/build.rs).

`build.rs` compiles the resource script with the locked `embed-resource` 3.0.11. Its [`.manifest_required()` result handling](https://docs.rs/embed-resource/3.0.11/src/embed_resource/lib.rs.html#350-358) makes unavailable or failed resource compilation a Windows build error and accepts non-Windows targets. It does not validate the XML's settings. The build script watches `resources` so changing the artwork or manifest triggers a rebuild.

Keep the `.ico` file as the source asset and replace the template artwork for a new application. An [ICON resource can contain several images](https://learn.microsoft.com/en-us/windows/win32/menurc/icon-resource). Select and check the sizes your window and tray need; embedding an icon alone does not establish correct DPI behavior.

The template's [icon-generation script](https://github.com/TeamDman/teamy-rust-cli/blob/7e62d72bbbf3ea1b302e48008da565e92d4b6c93/resources/make-icon.ps1) uses ImageMagick to produce several sizes. The build hook consumes the resulting `.ico`; it does not run that helper. A copied scaffold also inherits the generic manifest identity `app`, which needs review alongside the artwork.

## Load the named resource from the current executable

tb's [icon loader at `975848f`](https://github.com/TeamDman/tb/blob/975848fb27c0564cf029e96e1acf3e90e26b83a9/src/tray.rs#L254-L265) gets the current executable module, then looks up `main_icon`. The core lookup is:

```rust
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::LoadIconW;
use windows::core::w;

let module = unsafe { GetModuleHandleW(None)? };
let icon = unsafe { LoadIconW(Some(module.into()), w!("main_icon"))? };
```

The returned `HICON` can go directly to native window and tray APIs. tb logs a failed resource lookup and falls back to `LoadIconW(None, IDI_APPLICATION)`. Supplying `None` selects the system icon; supplying the executable module looks inside that module. See [Microsoft's LoadIconW contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadiconw).

The public `teamy-windows` helper [`get_icon_from_current_module`](https://github.com/TeamDman/teamy-rust-windows-utils/blob/cf9d141f86f9aa4b8c04cdaf988f5d171c9f233b/src/hicon/embedded_resource.rs#L12-L25) wraps named or ordinal lookup. It does not enumerate resources or automatically select the first EXE icon. Its `get_application_icon()` wrapper passes `IDI_APPLICATION` to the current-module lookup, so it is not equivalent to tb's system fallback.

Piing loads its named status icons, `green_check_icon` and `red_x_icon`, through `teamy-windows` 0.7.0. A status icon can differ from the application's main artwork. Keep the resource declarations and runtime names consistent.

## Convert or extract an icon when the consumer needs pixels

The extraction and conversion prior art lives in [teamy-rust-windows-utils](https://github.com/TeamDman/teamy-rust-windows-utils), published as `teamy-windows`. Release 0.11.1 records source revision `cf9d141`; the scoped helpers match the inspected later public checkout `155c5ef`. Piing's older 0.7.0 icon module lacks these additions.

| Need | Existing implementation |
| --- | --- |
| Convert a valid `HICON` into RGBA pixels | Public unsafe [`hicon_to_rgba`](https://github.com/TeamDman/teamy-rust-windows-utils/blob/cf9d141f86f9aa4b8c04cdaf988f5d171c9f233b/src/hicon/hicon_to_image.rs#L24-L171), returning `image::RgbaImage`. |
| Inspect icons in another EXE or DLL | [Icon-browser extraction](https://github.com/TeamDman/teamy-rust-windows-utils/blob/cf9d141f86f9aa4b8c04cdaf988f5d171c9f233b/src/cli/command/icon/browse/gui.rs#L327-L418), using extraction by index and requested size. These are private example functions, not exported library APIs. |

The converter obtains separate color and mask bitmaps with `GetIconInfo`, owns those copies with `windows::core::Owned`, applies transparency and swaps BGRA channels to RGBA. It leaves the input icon alive. Conversion allocates runtime pixel data while reusing the embedded source artwork; it avoids another compile-time byte embedding.

This source needs adaptation before treating it as a general decoder:

- it selects the color bitmap into a device context before `GetDIBits`, contrary to the [documented bitmap-selection contract](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdibits)
- it assumes a color bitmap and lacks explicit handling for monochrome icons, all-zero 32-bit alpha and unpremultiplication
- the browser truncates paths to 259 UTF-16 units and uses [PrivateExtractIconsW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-privateextracticonsw), which Microsoft says is unsuitable for general use

Correct the selection contract and validate representative formats, transparency and full input paths when adopting it. A successful color-icon example does not establish those cases.

## Extract an EXE icon to the format you need

Treat extraction and image encoding as separate steps. An `HICON` is a Windows object; an RGBA buffer is pixel data; PNG or JPEG is an encoded file.

1. Choose the EXE or DLL, icon index and required dimensions. [`ExtractIconExW`](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticonexw) returns large and small icon handles. These are selected images, not every image in the original icon resource.
2. Convert the selected valid handle to RGBA pixels. Use the `teamy-windows` converter above as prior art, after addressing its documented adoption gaps.
3. Choose the output encoder and its colour requirements. The `image` crate's [`save_with_format`](https://docs.rs/image/0.25.9/image/struct.ImageBuffer.html#method.save_with_format) selects a format explicitly; `save` derives it from the extension. Check that [encoding is enabled](https://docs.rs/image/0.25.9/image/enum.ImageFormat.html#method.writing_enabled) and handle encoder errors.
4. Save the image and release owned icons after their last consumer. Check the resulting dimensions, colours and transparency in an independent viewer.

PNG is a useful starting point for transparent artwork. For a format without alpha, choose a background and composite onto it before encoding. Exporting one raster image does not reconstruct a multi-size `.ico` resource or produce an SVG.

[TeamDman's answer on extracting EXE icons in Rust](https://stackoverflow.com/a/78190249) provides earlier prior art for this pipeline. It links [Cursor-Hero's extraction and conversion at `5161138`](https://github.com/TeamDman/Cursor-Hero/blob/51611380997d74f74f76fa776be4892a9906c005/crates/winutils/src/win_icons.rs), with a PNG-saving example. Its manual cleanup and permissive error handling need review before reuse. The current `teamy-windows` source is another implementation to inspect; neither establishes a validated general-purpose export command in the CLI template.

The [Stack Overflow topic index](stackoverflow-prior-art.md) records other public answers without treating them as implemented book features.

## Preserve ownership through the last consumer

| Icon origin | Ownership |
| --- | --- |
| `LoadIconW` or `LoadImageW` with `LR_SHARED` | Shared. Do not destroy it. |
| `ExtractIconExW` or an owned `LoadImageW` result without `LR_SHARED` | Release with `DestroyIcon` after all consumers finish. |
| Bitmap copies created by `GetIconInfo` | Separate resources with their own bitmap cleanup; converting the image does not release the input icon. |

Microsoft documents [shared-icon lifetimes](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroyicon), [LoadImageW flags](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadimagew) and [extracted-icon cleanup](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticonexw). An icon is not a kernel handle for `CloseHandle`.

The browser destroys extracted icons after conversion succeeds or returns an error. Its manual cleanup does not cover a conversion panic; use ownership that covers that exit too when extracting reusable library code.

Continue to [tray lifecycle](system-tray.md), [window events](window-creation.md) or [task-dialog artwork](native-message-boxes.md#reuse-application-artwork-deliberately) for the consuming interface. The CLI template embeds the resources but does not load or assign them to a desktop interface automatically.
