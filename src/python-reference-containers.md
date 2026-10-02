# Produce Python reference fixtures in a container

Use a pinned Python environment to produce independent fixtures for a [Rust model implementation](python-ml-to-rust.md). Keep the reference code, weights and test inputs identifiable. Mount only those inputs and a dedicated output folder.

The general runner below remains a proposed recipe. A local Julia-specific harness has now built a pinned Linux AMD64 image and produced independent CPU and CUDA fixtures after a verified GPU setup. A passing [synthetic finite-choice example](finite-choice-models.md) does not establish Python or model parity.

## Completed Julia CPU and CUDA references

The local `python/julia-reference` harness imports the pinned publisher's `JuliaDecisionModel`, strict `sequence` encoder and `Collator`. It bypasses the optional router, Bend bridge, compilation and INT8 paths. Native source and weights use [Julia-1 revision `a85b127`](https://huggingface.co/SupersonicLabs/Julia-1/tree/a85b127321d580d65176c89ced8273f305745d85); the separately acquired graph uses [ONNX revision `82a2fad`](https://huggingface.co/SupersonicLabs/Julia-1-ONNX/tree/82a2fadf8fccfccdc5fd4e1009ba8f1a265eb7a8). Source and artifact checksums are verified before model loading.

The frozen environment uses Python 3.12.15, uv 0.11.26, Torch 2.11.0+cu128, Transformers 5.0.0, tokenizers 0.22.2 and safetensors 0.7.0. The image bases are pinned by digest and transitive dependencies by `uv.lock`. Julia's [published dependency range](https://huggingface.co/SupersonicLabs/Julia-1/blob/a85b127321d580d65176c89ced8273f305745d85/pyproject.toml) requires Transformers `>=5.0,<5.1`; a different model's reference environment must not be reused unchanged merely because it already contains Torch.

Six synthetic text-only cases cover 2, 3, 5, 20, 4 and 2 ordered choices, Unicode text, an explicit question override and identical descriptions with distinct IDs. Defaults are the question `Which option should be chosen?`, 1024 total tokens and 256 head tokens. Each output preserves raw and padded token IDs, masks, marker positions, option IDs, logits, unit-temperature probabilities, selected IDs, intermediate marker vectors and input/environment digests.

Two CPU runs and two CUDA runs used float32, math SDPA, no autocast or TF32, seed 42 and deterministic-operation checks. Within each device's run pair, tokens, intermediate vectors, logits, probabilities and selected IDs matched exactly. Across CPU and CUDA, all six cases preserved exact encoding, option order and selected IDs, and passed the previously declared score tolerances: logits absolute/relative `1e-4`; probabilities absolute `1e-5`, relative `1e-4`. The comparison uses `abs(actual - expected) <= absolute + relative * abs(expected)`.

The CUDA reference ran on an RTX 4090 with Windows driver 610.88 through Podman's WSL2 machine. The guest received `nvidia-container-toolkit-base`, `libnvidia-container1` and `libnvidia-container-tools`, each version `1.20.1-1.x86_64`. Public RPM bytes independently matched their recorded official metadata SHA512 checksums; installed package-file verification reported no differences. The reviewed official repository configuration verifies signed repository metadata but disables package OpenPGP checks. WSL generated only `nvidia.com/gpu=all`; the non-root container probe verified that this selector exposed exactly one CUDA device. It is not a per-card isolation claim.

Eight model-free contract tests also passed. These results establish repeatability and cross-device numerical agreement for this pinned publisher reference. They do not establish native Rust or ONNX parity, calibrated confidence, a speed advantage or compatibility on other machines. Intermediate-vector capture adds work, so its timings are diagnostic evidence rather than an optimized benchmark. See [performance analysis](performance-analysis.md).

The runs used an offline, read-only, non-root container with dropped capabilities, read-only model inputs and a dedicated writable output folder. No host home, credential or engine socket was mounted, and no host-environment access was added. The approved setup changed guest packages and its CDI specification; it changed no Windows driver or host toolkit. Requested CUDA fails when unavailable; selecting CPU is explicit. The local model harness and fixtures are not yet a published product API.

## Establish GPU access before model execution

On Windows, Podman's documented NVIDIA route uses WSL2. Its Hyper-V machine route does not provide this GPU support. Use a compatible Windows NVIDIA driver and install the NVIDIA Container Toolkit inside the selected Podman machine. NVIDIA says not to install a Linux display driver inside WSL. See [Podman's Windows GPU instructions](https://podman-desktop.io/docs/podman/gpu) and [CUDA on WSL](https://docs.nvidia.com/cuda/wsl-user-guide/index.html).

Inspect the machine before starting it:

```powershell
podman --version
uv --version
podman machine list --format json
$machineName = '<selected-wsl-machine>'
podman machine inspect $machineName
```

If the selected machine is stopped, the operator's setup step is `podman machine start $machineName`. Starting a machine does not prove GPU access. After setup, inspect available devices with `podman machine ssh $machineName nvidia-ctk cdi list`. This chapter does not install a toolkit or change machine configuration.

CDI describes the device access supplied to a container. Select an explicit listed device, such as `nvidia.com/gpu=0`, and verify access as the intended non-root user. Driver changes can invalidate a specification. Current toolkit releases support automatic refresh, with documented exceptions. See [NVIDIA's CDI guidance](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/cdi-support.html).

## Pin the environment and the evidence

Prepare a small reference bundle:

| File | Purpose |
| --- | --- |
| `Containerfile` | Reviewed Python and uv images, installation and runtime settings. |
| `pyproject.toml` and `uv.lock` | Exact dependency resolution, package sources and artifact hashes. |
| `reference.py` and `vendor/` | Reviewed adapter and pinned publisher source. |
| `inputs/` | Explicit requests, tokenizer, model files and their evidence manifest. |
| `outputs/` | Initially empty, dedicated fixture destination. |

Record code and weight revisions separately, with licenses and file checksums. Reuse inspected artifacts before downloading another copy. Acquire dependencies and weights in a separate step; do not resolve them while generating golden outputs. A publisher's Python module executes code even when its weights use safetensors. Review pinned custom source before importing it.

Choose the exact Torch build and CUDA wheel index supported by the reference. Use an explicit uv index for Torch so unrelated packages continue to use their intended source. Do not let automatic backend selection silently change the experiment. Check Python, architecture, Torch, CUDA runtime and driver compatibility together. See [uv's PyTorch configuration](https://docs.astral.sh/uv/guides/integration/pytorch/) and [NVIDIA's compatibility guide](https://docs.nvidia.com/deploy/cuda-compatibility/minor-version-compatibility.html).

Pin the Python and uv images by digest, with their platform recorded. Create and review `uv.lock` during acquisition. `uv sync --locked` fails when the lock is missing or stale. `--frozen` skips the stale-lock check, so it is not equivalent. See [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/).

This illustrative `Containerfile` requires resolved `PYTHON_BASE` and `UV_BASE` build arguments. It assumes a compatible Python image and wheel-only dependencies:

```dockerfile
ARG PYTHON_BASE
ARG UV_BASE
FROM ${UV_BASE} AS uv
FROM ${PYTHON_BASE}
COPY --from=uv /uv /usr/local/bin/uv
ENV UV_PYTHON_DOWNLOADS=0 UV_LINK_MODE=copy PYTHONDONTWRITEBYTECODE=1
WORKDIR /app
COPY pyproject.toml uv.lock ./
RUN uv sync --locked --no-dev --no-install-project --no-build
COPY reference.py ./reference.py
COPY vendor/ ./vendor/
ENV HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 HOME=/tmp
USER 10001:10001
ENTRYPOINT ["/app/.venv/bin/python", "/app/reference.py"]
```

`--no-build` rejects dependency source builds; it does not establish that installed wheels or imports are harmless. Resolve reviewed image references such as `python:<approved-tag>@sha256:<digest>` before building. Record the resulting image ID using `podman build --iidfile image-id.txt`. Use the installed virtual environment directly during inference, avoiding uv's automatic runtime locking or syncing. These choices follow [uv's container guidance](https://docs.astral.sh/uv/guides/integration/docker/).

## Run with explicit inputs and outputs

The proposed runner accepts `--request`, `--model-dir`, `--output-dir` and `--device`. It must fail if the requested GPU is unavailable. It must not silently select CPU or download missing files. For Julia-1, the request is text-only with 2 to 20 choices; visual inputs are rejected. See [finite-choice contracts](finite-choice-models.md).

A text-only request can preserve this explicit option order. The runner must match the selector to its recorded model artifacts:

```json
{
  "selector": { "model": "SupersonicLabs/Julia-1", "model-dir": null },
  "input": { "text": "Choose a category for a code example.", "visuals": [] },
  "options": [
    { "id": "writing", "content": { "text": "Writing programs", "visuals": [] } },
    { "id": "playing", "content": { "text": "Playing games", "visuals": [] } }
  ]
}
```

After the bundle, image and device have been verified, the PowerShell command shape is:

```powershell
$inputs = (Resolve-Path -LiteralPath './inputs').Path
$outputs = (Resolve-Path -LiteralPath './outputs').Path
$image = (Get-Content -LiteralPath './image-id.txt' -Raw).Trim()
if ($image -notmatch '^sha256:[0-9a-f]{64}$') {
    throw 'A recorded local image ID is required.'
}
$referenceArgs = @(
    'run', '--rm', '--pull=never', '--network=none', '--http-proxy=false',
    '--read-only', '--user=10001:10001', '--cap-drop=all',
    '--security-opt=no-new-privileges', '--device=nvidia.com/gpu=0',
    '--tmpfs=/tmp:rw,nosuid,nodev,size=256m',
    '--volume', "${inputs}:/inputs:ro",
    '--volume', "${outputs}:/outputs:rw",
    $image,
    '--request', '/inputs/request.json',
    '--model-dir', '/inputs/model', '--output-dir', '/outputs',
    '--device', 'cuda:0'
)
& podman @referenceArgs
if ($LASTEXITCODE -ne 0) { throw 'Python reference execution failed.' }
```

Both folders must already exist. Verify Windows path translation and output permissions for this user before processing a model. Do not add `:U` to repair a permission failure: it can change source ownership. Windows mount behavior is described in [Podman's Windows tutorial](https://github.com/podman-container-tools/podman/blob/main/docs/tutorials/podman-for-windows.md).

The example grants one GPU, read-only inputs, a writable output folder and temporary memory storage. It passes no host home, credentials or container-engine socket. It uses no network or privileged mode. Podman documents these [runtime flags and mount behavior](https://docs.podman.io/en/latest/markdown/podman-run.1.html). If a device or security-policy setting fails, diagnose that specific failure rather than broadening access. A container provides useful isolation, but Python still runs native code and GPU operations still reach the host driver. It is not a perfect sandbox.

## Retain evidence that Rust can compare independently

The runner should emit a versioned manifest and bounded JSON or tensor-array fixtures. Keep diagnostics on stderr and fail on unrecoverable execution errors. Do not treat an incomplete fixture folder as a successful reference.

| Required record | Comparison purpose |
| --- | --- |
| Image, lock, source and weight identities | Identify exactly what produced the fixture. |
| Device, runtime versions, dtype and precision settings | Explain execution differences. |
| Request digest and ordered option IDs | Bind scores to the original task. |
| Token IDs, masks and intermediate tensor metadata | Locate preprocessing or numerical divergence. |
| Ordered logits and probabilities | Compare model outputs without regenerating expected values in Rust. |
| Artifact checksums and declared tolerances | Preserve bytes and define acceptance before optimization. |

Use evaluation mode and inference-only execution. Record seeds and applicable deterministic settings. Preserve failures when deterministic operations are unavailable. Pinning packages makes the environment repeatable; it does not guarantee identical floating-point results across devices or releases. PyTorch documents these [reproducibility limits](https://docs.pytorch.org/docs/main/notes/randomness.html).

Compare preprocessing and tokens before final scores. Keep the Python reference independent of the Rust path under test. Check task behavior and performance separately from numeric agreement. See [model adoption](model-adoption.md#prove-parity-before-changing-performance-defaults).
