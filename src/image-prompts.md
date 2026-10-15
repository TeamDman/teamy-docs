# Prompt with local images

Use a Liquid `image` tag to put a local image at a specific position in a generated-text prompt:

```powershell
teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
  --template 'What is this? {% image "images/example.png" %}'
```

This chapter describes unpublished local implementation with model-free checks and small-fixture CUDA qualification. Check the matching locally built command's help. Rendering tests, CLI checks and live vision qualification are separate evidence; the qualification record below states what remains unverified.

This interface extends [local text generation](local-text-generation.md). The newer Qwen decision path also accepts images in the input or supplied choices. Julia's [finite-choice decisions](finite-choice-models.md) remain text-only.

Add `--daemon` to use an already running [resident inference service](resident-inference.md). Image acquisition stays in the invoking client; bounded encoded bytes cross IPC and the server reuses its prepared language and vision runtimes. A render-only preview still performs no acquisition or model loading.

## Write a template with direct image paths

Each image tag takes one nonempty, single- or double-quoted path literal. No separate image name, binding registry or image flag is needed. Relative paths resolve from the command's current directory, including when the template file lives elsewhere. Acquisition accepts regular files; Windows network and drive-relative paths are rejected.

For a saved `prompts/compare.liquid` template:

```liquid
{{ question }}
Before: {% image "images/before.png" %}
After: {% image "images/after image.png" %}
Describe the visible changes.
```

Run it from the directory containing `images`:

```powershell
teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
  --template-file ./prompts/compare.liquid `
  --var 'question=What changed between these screenshots?' `
  --max-new-tokens 128
```

Repeat `--var 'name=value'` for more text variables. Values remain literal text: inserting `{% image "private.png" %}` through a variable does not execute another image tag. Backslashes in quoted Liquid paths remain literal. The renderer does not add a second escaping or parsing step.

Read a template from stdin explicitly:

```powershell
Get-Content -LiteralPath ./prompts/compare.liquid -Raw |
  teamy-llm prompt --model qwen3.8-27b-uncensored-q4-k-m `
    --template-file - --var 'question=Describe the difference.'
```

Choose one input: positional prompt text, `--template` or `--template-file`. An ordinary `prompt '...'` keeps its text literal. Liquid processing requires an explicit template option.

## Score images as input or choices

The Qwen decision extension uses the same Liquid profile and acquisition rules. The [final 5 October 2026 RTX 4090 check](resident-inference.md#qualified-final-local-build) used a red image as input, then red and blue images as choices. It selected red with probability 0.99708226 using explicit 128-position prefill chunks. The 3 synthetic 64 × 64 images occupied 27 image positions in total. This checks ordered acquisition, encoding and scoring for that fixture; it is not a general decision-accuracy benchmark.

Use `--prompt-template` for the input and repeated `--choice-template` flags for choices:

```powershell
teamy-llm decide --provider qwen --model qwen3.8-27b-uncensored-q4-k-m `
  --prompt-template 'Which example matches? {% image "images/query.png" %}' `
  --choice-template 'First example: {% image "images/first.png" %}' `
  --choice-template 'Second example: {% image "images/second.png" %}' `
  --daemon
```

Use literal `--choice` strings when only the input needs a template. Choose either repeated literal choices or repeated choice templates for one invocation. Repeat `--var 'name=value'` to provide literal text variables to the selected templates. Ordinary `--prompt` and `--choice` text does not execute Liquid.

The client renders and acquires the image bytes before sending the decision request. The daemon never opens an image pathname from that request. The resulting ordered content enters the same vision tower and language weights used for image prompting. The combined input and all options share the 4-image and encoded-byte bounds; each choice does not receive a separate allowance.

The model compares the supplied options through [single-label vocabulary scoring](finite-choice-models.md#reuse-qwen-generation-weights-for-decisions). Image positions consume context before that score is read. Record processed image shapes and the combined image-position count when comparing latency or memory use.

## Preview ordered parts before reading images

Render without opening image paths or loading a model:

```powershell
teamy-llm prompt --template 'Before: {% image "before.png" %} After: {% image "after.png" %}' `
  --render-only --output-format json
```

The preview's typed data has this shape:

```json
{
  "version": 1,
  "template-profile": "teamy-liquid-prompt-v1",
  "parts": [
    { "kind": "text", "text": "Before: " },
    { "kind": "image", "path": "before.png" },
    { "kind": "text", "text": " After: " },
    { "kind": "image", "path": "after.png" }
  ],
  "images-inspected": false
}
```

Liquid renders the entire template before acquisition starts. Skipped conditional branches produce no image parts and open no files. A render failure discards the partial result. Repeating a path produces repeated image occurrences in order.

The image tag and text writer share a fresh collector for one render. The tag appends a typed `Image` part after preceding text. There are no magic image strings to recognise after rendering. Facet defines the public data shape; string variables become Liquid scalar values directly, without a JSON or Serde conversion.

The `teamy-liquid-prompt-v1` profile registers `assign`, `if` with `elsif` and `else`, `unless`, `for`, `raw` and `comment`. Its text filters are `append`, `prepend`, `capitalize`, `downcase`, `upcase`, `lstrip`, `rstrip`, `strip`, `strip_newlines`, `split`, `join`, `size` and `default`.

The profile excludes `capture`, `ifchanged`, partial `include` and `render`. Private text buffers and additional template loading need explicit semantics before we can support images inside them. Content inside `raw` remains text; comments produce no prompt parts.

Source, rendered text and variable names/values each have a 2 MiB limit. There are at most 256 variables, 4 image occurrences and 32 KiB per path. The syntax budget permits 512 opening Liquid markers, including raw/comment content, and 32 nested blocks. Loop sources permit 4,096 items before `limit`/`offset`; one render permits 32,768 loop iterations. Filter results also have bounded intermediate sizes.

## Keep acquisition and model preparation explicit

The image adapter accepts PNG and JPEG by decoded format. It limits a request to 4 image occurrences, 8 MiB per compressed image and 16 MiB compressed total. Each decoded dimension must be 2 to 4,096 pixels. Combined decoded RGB storage is limited to 64 MiB. These bounds are separate from the vision encoder's resizing policy and the language model's context capacity.

Model preparation is separate from prompting:

```powershell
teamy-llm model prepare qwen3.8-27b-uncensored-q4-k-m --with-mmproj
```

The language GGUF and its matching vision projector come from the same pinned [OrcaRouter GGUF revision `fc437a3`](https://huggingface.co/orcarouter/Qwen3.8-27B-Uncensored-GGUF/tree/fc437a3374c9977bdc339a8ec106dcd9a0357001). The projector filename is `mmproj-Qwen3.8-27B-Uncensored-f16.gguf`. Prompting uses prepared artifacts; it does not download a projector implicitly.

Despite `f16` in that filename, the frozen projector contains BF16 tensors. The local CUDA executor supports F32 and F16 weight operations. It expands those BF16 values exactly into F32 and uses its existing F32 matrix operations, with F16 matrix execution disabled for this tower. This preserves BF16's values and exponent range. It requires approximately 1.86 GB of host weight storage plus CUDA storage, rather than treating the filename as a precision guarantee. The original artifact and quantized language weights remain unchanged.

Image context counting uses a compiled vision configuration bound to the pinned projector's expected size and SHA-256. This preflight does not open the projector or parse an unverified GGUF header. The owner verifies every projector byte before parsing it or using weights, then checks that the parsed fields match the compiled configuration.

Text-only requests do not open the projector. The model's published vision capability and this adapter's qualified capability are different facts.

## Understand where image embeddings enter the model

The image filename is an instruction to our program, not model text. An image tag determines its position among surrounding text:

```text
Text tokens → vision-start → image embedding vectors → vision-end → text tokens
```

The path does not become vocabulary tokens. The adapter performs these operations:

1. Decode RGB, resize while preserving aspect ratio, and centre-pad to the patch grid. Normalize colours using the pinned vision configuration.
2. Divide the processed image into 16 × 16 patches, ordered in 2 × 2 spatial blocks. Run the 27-layer vision encoder.
3. Merge each 2 × 2 patch group and project it into a 5,120-value vector compatible with the language model.
4. Insert those vectors at the tag's sequence position, between Qwen's vision markers. Feed surrounding text through the ordinary tokenizer and shared language session.

A processed 512 × 512 image produces a 16 × 16 merged grid: 256 image positions, each with 5,120 floating-point values. The processed shape determines this count. The pinned configuration permits at most 4,096 merged positions per image. Resizing follows the compiled, artifact-bound configuration and does not preserve arbitrary input resolution.

Every image position consumes a linear context slot. Text and output need storage alongside it. Spatial M-RoPE assigns image rows and columns separately: its position cursor advances by the larger grid dimension, while context storage still counts all grid positions. Treating the image as 256 ordinary words would lose those spatial coordinates. The newer [context policy](local-text-generation.md#choose-output-and-context-budgets) can grow storage or warn before discarding oldest context. When discarding an image, it removes its complete span rather than leaving partial embeddings.

Liquid is our user-prompt template language. It does not replace the publisher's chat-template conventions or alter system-message roles. The relevant pinned Makepad source is [vision preprocessing and projection](https://github.com/makepad/makepad/blob/9e5e3d2b03214f8c2c37adffa0e308c815662060/libs/ai/llm/src/vision.rs) and [image embedding prefill](https://github.com/makepad/makepad/blob/9e5e3d2b03214f8c2c37adffa0e308c815662060/libs/ai/llm/src/session.rs).

## Record qualification separately from implementation

The implementation uses the session's explicit CUDA device and does not silently fall back to CPU. Its vision tower loads lazily, retains one shape graph, and reuses that graph for matching shapes. A new shape evicts the prior graph before allocation. The first image request includes projector verification and vision loading; a resident language model alone does not remove those costs.

CUDA vision attention streams its calculation without retaining a quadratic attention-score array. Liveness analysis plans activation arenas, and allocation admission checks available device memory with a 1 GiB reserve. These controls reduce memory demand and reject an allocation that exceeds the observed budget. They do not guarantee that every permitted image shape fits: other device use and later allocations can change that budget.

Tracing separates `llm_template_render`, `llm_decode_image`, `gguf_vision_load`, `gguf_image_preprocess`, `gguf_image_encode` and `gguf_image_prefill`. Record cold and resident timings separately, together with processed shape and context size. See [performance analysis](performance-analysis.md) and [logging](logging.md).

Cancellation checks surround preprocessing and encoding and each image-prefill batch. An executing GPU kernel cannot be preempted. See [cooperative cancellation](cancellation.md).

For the Qwen scheduler, `--timeout-ms` starts after language-runtime loading when the request reaches its GPU owner. It includes owner queueing, suspension, the first lazy projector load, image encoding and generation. `--stop-after-duration` requests cancellation from CLI startup, including earlier phases. Both are cooperative boundaries, rather than preemption of an active load or kernel.

On 3 October 2026, the unpublished renderer passed all 19 model-free unit tests. They cover exact part order, quoted paths, opaque interpolation, skipped branches, loops, raw/comments, excluded constructs, bounds, concurrent isolation and Facet JSON round trips. The core passed 11 tests, including its versioned generation contract. These checks do not load images or models.

The local release CLI also passed model-free preview checks for inline, saved-file and stdin templates; variables; skipped branches; syntax errors; excluded constructs; duplicate variables; and input exclusivity. Those checks used missing image paths and model locations, confirming that the preview does not acquire them.

The selected Rust checks passed 516 distinct tests: 238 owned-package library tests across Burn Qwen, CLI, core, GGUF adapter, renderer and service; 270 vendored Makepad tests; and 8 core integration tests. Two existing host-dependent tests remained ignored. Clippy passed for the changed owned packages. These are selected checks, rather than a claim that every workspace or third-party test ran.

Live CUDA checks recognised a synthetic 224 × 224 PNG containing a red square and a JPEG containing a blue circle. An ordered two-image request described the red square first and the blue circle second. Reversing the input produced the reverse description. Traces counted 49 image positions for one image and 98 for the pair.

A resident cancellation request at 150 ms ended with nonzero status and a clear cancellation error. Subsequent image and text requests succeeded. Missing files, unsupported PPM input, Windows drive-relative paths and image requests to the legacy text-only provider were rejected. That earlier build rejected 66 input positions, including 49 image positions, plus 128 requested output tokens against a 128-token context. The error reported the complete capacity calculation rather than truncating. This dated check predates the newer warning and `--strict` policy.

These smoke checks establish the tested image formats, order, cancellation recovery and validation paths. They do not establish independent preprocessing or embedding-tensor parity, broader vision accuracy, or memory fit for every permitted resolution. The 4-image and dimension bounds constrain acquisition; they are not a GPU-memory guarantee. Keep future numerical-reference checks separate from plausible generated descriptions.

### Local CUDA measurements on 3 October 2026

The release build used an RTX 4090, default 8,192-token context, Direct mode and a 40-token output cap. The single-image fixtures used 76 input positions and produced 11 red-square or 8 blue-circle tokens. Ordered pairs used 137 input positions and produced 10 tokens; reversed pairs produced 5. The text-only recovery request used 17 input positions and produced 2 tokens. The 10 measurements below used 224 × 224 images. A larger final-build fixture is recorded separately.

Debug tracing, NDJSON file logging and approximately 250 ms GPU-memory sampling were enabled. Token readiness starts inside `PromptCli`, before template rendering and image acquisition, after argument/logging setup. Output format was JSON: first-visible-token readiness does not mean text had been written to stdout. JSON was emitted only after completion.

The harness wall time includes process startup, completion and GPU-sampler thread shutdown. It is not an isolated launch-to-process-exit timer. Operating-system disk caches were uncontrolled; fresh processes each loaded their own models, but these are not certified cold-disk samples.

| Request | First visible token ready, seconds | Completed result ready, seconds | Harness wall, seconds |
| --- | ---: | ---: | ---: |
| First captured fresh process, red-square PNG | 13.361 | 13.559 | 16.208 |
| Later fresh process, blue-circle JPEG | 7.054 | 7.188 | 10.331 |
| Later fresh process, ordered pair | 6.947 | 7.123 | 9.852 |
| Later fresh process, reversed pair | 6.873 | 6.940 | 9.897 |
| Resident language model, first vision request | 2.071 | 2.270 | 2.377 |
| Resident vision tower, blue-circle JPEG | 0.717 | 0.848 | 0.993 |
| Resident vision tower, repeated red-square PNG | 0.623 | 0.821 | 0.945 |
| Resident vision tower, ordered pair | 1.028 | 1.204 | 1.329 |
| Resident vision tower, reversed pair | 0.956 | 1.021 | 1.141 |
| Resident text-only request after images | 0.221 | 0.221 | 0.356 |

Each resident row used a fresh CLI client connecting to one prewarmed local language service. The server recorded one vision-tower preparation for the entire 6-request sequence. Its first vision load took approximately 927 ms; that first request also included CUDA weight upload during encoding. Later same-shape requests reused the prepared tower and shape graph.

The highest sampled whole-device usage was 21,688 MiB against a reported 24,564 MiB capacity. The initial baseline was approximately 1,857 MiB. These values include other device use and sampling can miss short peaks. They are not an isolated allocation measurement for the adapter, or a qualification of the maximum permitted resolution.

The capture used executable SHA-256 `fa8d0d543125ee8e15bbc6572f37d9399111db1554b8f6c0df57faf04854247c` and Cargo lock SHA-256 `034539795cfdd80fa12e424a5d312786de45e9652af372bd823c888e6c1aa83f`. Source was an uncommitted working patch over [service revision `6aaae2c`](https://github.com/TeamDman/teamy-llm-service/tree/6aaae2cfc36dd0f27f5a5f48aea578d49ea2c32a). The implementation was not published with this capture; the base revision alone does not reproduce it.

### Final-build non-square fixture

The final locally installed executable has SHA-256 `5fbbfc508104bdb5d133e0cd7dab158bc0a2cf53af7a1fb29c77040410fe7f22`. It includes a model-capability reporting consistency fix; the earlier timing table retains the executable identity actually measured there.

This build recognised a 768 × 512 PNG with a red square on the left and a blue circle on the right, preserving that spatial relationship in its answer. The request used 384 image positions and 413 total prompt positions, and generated 21 tokens.

| Request | First visible token ready, seconds | Completed result ready, seconds | Harness wall, seconds |
| --- | ---: | ---: | ---: |
| Final build, fresh process, non-square PNG | 8.831 | 9.243 | 11.759 |

The highest sampled whole-device usage was 21,686 MiB. The timing and sampling boundaries above still apply, including uncontrolled disk-cache state. Shapes beyond this tested 768 × 512 fixture remain unqualified. This one spatial-description check does not establish independent embedding parity or broad vision accuracy.

Implementation locations in the unpublished service checkout are `crates/teamy_llm_prompt`, the prompt CLI, the service's image acquisition boundary, and the Makepad GGUF adapter. This follows Liquid prior art in SFM's source projections and Cloud-Terrastodon's `pick`; the prompt-specific typed image tag remains a local addition.
