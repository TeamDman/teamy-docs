# Evaluate agent harnesses and memory

Compare the execution harness separately from a memory extension. A durable archive, a bounded model context and an authorization boundary solve different problems. These references were reviewed on 13 October 2026 and remain evaluation candidates; no harness or plugin was installed or executed.

| Need | Reference | Role and next check |
| --- | --- | --- |
| Retain retrievable history while bounding each model turn | [OMP OptChat](#omp-optchat) | Memory/context extension for OMP; Linux requirements, added model-call costs and incomplete archival coverage. Confirm reuse terms and qualify recall before adoption. |
| Study an agent backend with several UI and provider integrations | [OpenCode](#opencode) | Full coding harness; client/server interfaces, tools, permissions and compaction. Pin the major version and test our protocol/security boundaries. |

## OMP OptChat

[OMP OptChat](https://github.com/ericjuta/omp-optchat) is an OMP extension and memory layer, not a standalone agent harness. Inspected [revision `1cff1a1fe060a65c575c438927720532502ba8a1`](https://github.com/ericjuta/omp-optchat/tree/1cff1a1fe060a65c575c438927720532502ba8a1), package 0.7.0. Its [store](https://github.com/ericjuta/omp-optchat/blob/1cff1a1fe060a65c575c438927720532502ba8a1/src/store.ts) and [memory tree](https://github.com/ericjuta/omp-optchat/blob/1cff1a1fe060a65c575c438927720532502ba8a1/src/memory.ts) preserve a durable append-only log and build a binary tree of contextual summaries. Each user turn replaces inference history with a bounded view; `zoom` drills into original recorded messages and `chat_search` searches their words. OMP still owns system instructions, tools, delegation and the ongoing turn.

The [README](https://github.com/ericjuta/omp-optchat/blob/1cff1a1fe060a65c575c438927720532502ba8a1/README.md) requires Linux, Bun and OMP 18.9.0+, with authenticated main/compressor models; the [manifest](https://github.com/ericjuta/omp-optchat/blob/1cff1a1fe060a65c575c438927720532502ba8a1/package.json) requires Bun 1.3.14+. Five optional judge gates add standing-rule, prefetch, value, tool-screening and recall behavior. Compressor and judge calls add cost and provider latency; a turn can cancel after waiting 25 seconds for unsettled memory. Recorded history is not a complete byte-for-byte archive of every input: reasoning is excluded, ordinary tool-result text is capped, and audio/video/original files are not archived. Backups remain the operator's job; fsync is not backup. This is useful prior art for durable history plus bounded model context, not evidence of Windows support, perfect recall or a replacement for deterministic permissions. No top-level project license was found in the inspected checkout/manifest; vendored licenses do not establish the extension's own reuse terms. Status: source inspection only, no plugin installation or execution.

## OpenCode

[OpenCode](https://opencode.ai/) is a full coding-agent harness with terminal, desktop and IDE surfaces. Its [server documentation](https://opencode.ai/docs/server/) describes a TUI client over a headless HTTP/OpenAPI server, with an SDK and session/event endpoints: useful prior art for keeping agent execution independent of a UI. [Provider configuration](https://opencode.ai/docs/providers/) supports hosted providers and custom OpenAI-compatible endpoints; compatibility still depends on the configured protocol and model capabilities.

[Tools and permissions](https://opencode.ai/docs/permissions/) use allow/ask/deny rules with command/path patterns. They need an intentional configuration and are not equivalent to OS isolation. [Context configuration](https://opencode.ai/docs/config/#compaction) controls automatic compaction, pruning and reserved tokens; this is different from OptChat's immutable history plus summary-tree navigation. [Plugins](https://opencode.ai/docs/plugins/), custom tools, MCP, agents, skills and LSP provide extension points. Provider/plugin execution and data exposure need application-specific qualification.

Version boundary: official documentation now explicitly distinguishes OpenCode 1 from OpenCode 2; do not assume a V1 configuration example describes the current development branch. Official source inspected at [`d7cfdaa3c791c444897187bf07b0a43cd25b3435`](https://github.com/anomalyco/opencode/tree/d7cfdaa3c791c444897187bf07b0a43cd25b3435); its [LICENSE](https://github.com/anomalyco/opencode/blob/d7cfdaa3c791c444897187bf07b0a43cd25b3435/LICENSE) is MIT. Official website and repository sources were inspected without executing the harness. Status: documentation/source-reviewed evaluation candidate; no installation, runtime or security tests.

## Minecraft client harnesses

[ClientDevBridge CLI](minecraft-client-automation.md) supplies a shell interface to a companion game mod. Its Minecraft-specific launch, observation and hot-swap boundaries complement an outer coding harness; see the dedicated reference for SFM puppet/container applicability.

## Adapt the useful boundaries

Keep user authorization and deterministic [reviewable execution](writing-programs.md#make-effects-reviewable) outside model summaries and judge decisions. For our tooling, study a versioned broker/service contract, bounded event delivery, original-record retrieval and explicit permission decisions. Test interrupted writes, recovery, cancellation, stale summaries and provider failures before adopting a memory layer or harness. Local logs and backups remain private data and do not belong in this public survey.
