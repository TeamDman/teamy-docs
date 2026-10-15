# Automate a Minecraft development client

Use a client bridge when a mod-development check needs both game state and what the player actually sees. Keep the coding-agent harness separate from the game-control interface: [agent harnesses and memory](agent-harnesses.md) describes the outer execution layer. This Minecraft harness reference was reviewed on 13 October 2026; it is a source-reviewed candidate, not an adopted or locally tested SFM integration.

## ClientDevBridge CLI

[CyclopsMC/clientdevbridge-cli](https://github.com/CyclopsMC/clientdevbridge-cli) was newly added to this survey after inspection at [`c0baf522a53f3c3c5a07a5e8a19fe2412155d373`](https://github.com/CyclopsMC/clientdevbridge-cli/tree/c0baf522a53f3c3c5a07a5e8a19fe2412155d373). It is a TypeScript shell CLI that talks to the companion [ClientDevBridge mod](https://github.com/CyclopsMC/ClientDevBridge), rather than a model, autonomous coding harness or MCP server. The [protocol client](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/src/protocol/client.ts) makes JSON-RPC calls over a loopback WebSocket, with one connection per invocation.

| Check to automate | Documented interface | Practical limit |
| --- | --- | --- |
| Start and observe a real development client | `doctor`, `start`, `status`, `screenshot`, `stop` | Requires a compatible mod-development project and client environment. |
| Inspect and drive a screen | `inspect-gui`, `snapshot --json`, `find`, `click`, `tooltip` | Structural widget/slot evidence complements a screenshot; neither alone proves behavior. |
| Reproduce a world interaction | `world-reset`, `setblock`, `give`, `teleport`, `command` | World-changing operations need a disposable test save and explicit expected results. |
| Detect rendering drift | `compare`, regions/thresholds and `screenshot --diff` | Golden images are renderer-specific; a tolerance is not a semantic assertion. |
| Change code while keeping the client alive | `hotswap`, with a JDWP-enabled launch | Method-body updates differ from class-schema changes that require restart. |

The [README](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/README.md) documents screenshot settings, GUI-space coordinates, world setup and Linux virtual-display rendering through Xvfb/Mesa llvmpipe. These are upstream capabilities and examples; no client was launched here. The [Gradle init-script generator](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/src/initscript.ts) injects the bridge artifact into `runClient` and manages pinned options. Inspect restoration behavior before applying it to a developer's normal run directory.

## Integration and version boundaries

The [package manifest](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/package.json) identifies CLI 0.9.0, Node.js 20+ and MIT licensing; the [license](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/LICENSE) applies to this CLI. The companion mod and other dependencies have their own notices. A project's Gradle wrapper, compatible Java runtime and loader build remain prerequisites; Node.js alone cannot run Minecraft.

[Loader detection](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/src/detect.ts) explicitly models Fabric and NeoForge. Its [artifact mapping](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/src/artifacts.ts) lists 1.21.1, 26.1.2 and 26.2 across three mod branches, with broader version-match expressions. Those expressions choose a branch; they do not establish that every matching point release has a published artifact or was tested. Minecraft 1.20.1 is not represented by this inspected mapping. Verify the target version and published bridge build before integrating an SFM branch; do not infer compatibility from a version-agnostic wire protocol.

[Hot-swap implementation](https://github.com/CyclopsMC/clientdevbridge-cli/blob/c0baf522a53f3c3c5a07a5e8a19fe2412155d373/src/commands/hotswap.ts) compiles/diffs classes and uses debugger redefinition. A debug port is required; ordinary supported edits preserve the running client, while unsupported structural changes need restart or `--restart-if-needed`. This is a distinct mechanism from resource reload and does not establish arbitrary state migration. The `eval` escape hatch executes Groovy; it is an execution capability requiring the same authorization discipline as other tools.

## Relevance to SFM puppet and container checks

Our proposed use is to retain the existing puppet/container orchestration and evaluate this CLI as a game-observation/control adapter. Run it beside the compatible development client so the loopback connection stays local to that environment; provide Java/Gradle and, for a Linux runner without a display, Xvfb and software GL dependencies. Container packaging, graphics setup and bridge-version compatibility still need qualification.

A useful trial is one disposable SFM screen scenario: prepare known blocks/items, open the screen, capture its JSON structure and screenshot, perform one action, then check both state and rendered output. Compare that evidence with the puppet workflow before replacing any existing automation. Keep build/restart, ready-state waits, cleanup and renderer-specific golden selection explicit. Source inspection suggests complementary capabilities; it does not demonstrate interoperability, faster iteration or regression reliability for SFM. No installation, upstream code execution, credentials or runtime tests were performed for this entry.
