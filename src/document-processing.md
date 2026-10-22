# Extract text and layout from documents

Evaluate OCR for scanned pages separately from extracting an existing PDF text layer. These are surveyed references, not adopted Teamy dependencies or TeamyStyle requirements. This entry was reviewed on 18 October 2026; no package, model or inference code was installed or executed.

## Indra OCR

| Source | Purpose and evidence status |
| --- | --- |
| [Indra implementation](https://github.com/datalab-to/indra) | Official Python pipeline, runtime requirements and benchmark methodology. |
| [Indra OCR model](https://huggingface.co/datalab-to/indra-ocr) | Official model card, weights and separate model-license terms. |
| [Supplied discovery/announcement post](https://x.com/VikParuchuri/status/2111150822323052689) | Retained as discovery provenance. The post returned HTTP 403 during review; its contents remain unverified. |

The [implementation README at `3d773ae`](https://github.com/datalab-to/indra/blob/3d773aeedc5d79d1d0a2a04007a9075cd71a59f4/README.md) and [model card at `ed345c7`](https://huggingface.co/datalab-to/indra-ocr/blob/ed345c7f158eddbc7dee0c6a7269afb75a94f7e8/README.md) describe CPU-oriented PDF/image OCR and document parsing. Outputs include Markdown, plain text, or JSON with line/layout boxes and reading order. Auto mode reuses a valid PDF text layer and OCRs missing content; text-layer coverage across scripts does not establish OCR support for those scripts.

The documented runtime uses Python 3.10+, ONNX Runtime, OpenCV and pypdfium2, without PyTorch. The [Python API](https://github.com/datalab-to/indra/blob/3d773aeedc5d79d1d0a2a04007a9075cd71a59f4/indra_ocr/__init__.py#L27) defaults to `CPUExecutionProvider`, requiring no GPU for that path; callers can select other providers. Upstream reports tests on Linux x86_64/aarch64 and Apple-silicon macOS; Windows readiness was not established here. Clean print is the intended scope: handwriting, degraded scans, complex tables and scripts outside the documented OCR list remain limitations. The headline throughput is an upstream multi-core benchmark on a specified workload, excluding model loading and warm-up; no local accuracy, latency or throughput was measured.

The [implementation code license](https://github.com/datalab-to/indra/blob/3d773aeedc5d79d1d0a2a04007a9075cd71a59f4/LICENSE) is Apache-2.0. The weights have a separate [modified OpenRAIL-M license](https://huggingface.co/datalab-to/indra-ocr/blob/ed345c7f158eddbc7dee0c6a7269afb75a94f7e8/LICENSE), including commercial revenue/funding thresholds, competitive-use restrictions, and attribution/share-alike terms that also address outputs. The code license does not supply permission for every model or output use; inspect the pinned model terms before reuse. Keep code and weight revisions separate in any [evaluation bundle](model-adoption.md).
