# Produce Python reference fixtures in a container

Use a pinned Python environment to produce independent fixtures for a [Rust model implementation](python-ml-to-rust.md). Keep the reference code, weights and test inputs identifiable. Mount only those inputs and a dedicated output folder.

This is a proposed Podman and uv recipe. Its command syntax has been reviewed; no image was built and no GPU reference was run during this documentation work. The model-specific `reference.py`, dependency lock and image digests must still be prepared and validated. A passing [synthetic finite-choice example](finite-choice-models.md) does not establish Python or model parity.

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
