# Compare private meshes and public tunnels

Choose by reachability: a private mesh connects enrolled devices; a reverse tunnel makes a selected service reachable through a public endpoint. These are different trust boundaries. OpenTunnel belongs to the latter category; Tunnet offers both as separate capabilities. Tailscale is the comparison baseline named by Tunnet, but upstream competitor feature tables are claims to evaluate, not independent validation.

## Evidence and adoption status

Reviewed 14 October 2026. Documentation/source review only; neither tool has been installed, benchmarked, security-audited, or adopted here. Official repository snapshots: [OpenTunnel `0cad49c`](https://github.com/anomalyco/opentunnel/tree/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db) and [Tunnet `30dfeaa`](https://github.com/tunnetio/Tunnet/tree/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce). Linked hosted documentation may change independently.

Discovery provenance supplied for this survey: [OpenTunnel via thdxr on X](https://x.com/thdxr/status/2107956848318177622) and [Tunnet via orielhaim on X](https://x.com/orielhaim/status/2107984883964403791). These are discovery links, not technical evidence; their contents were not verified.

## OpenTunnel: public TLS ingress

The [official README](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/README.md) describes a blind TLS tunnel: public TCP/443 traffic is routed by TLS ClientHello/SNI through a bridge to the local client, which holds the certificate key, terminates TLS and forwards plaintext to a selected local port. It supports HTTPS use cases, but the protocol can carry other application protocols inside TLS; this is not an IP mesh VPN.

The [protocol](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/docs/protocol.md) uses an outbound WebSocket bridge and bearer-token tunnel ownership. This avoids requiring inbound reachability at the local machine; it is relay-based rather than peer-to-peer hole punching. A random route URL is **not visitor authentication**. The application must authenticate callers and must not trust requests merely because forwarding makes their peer appear local. There is no per-connection bridge flow control yet, so one slow reader can delay other connections.

Hosting is a Rust server on Fly.io with MySQL, ACME issuance and DNS integration. Server source and deployment configuration are included; operating a replacement requires those dependencies and a domain/certificate setup ([server documentation](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/docs/server.md)). The README lists Linux/macOS x64 and arm64 binaries, Rust and TypeScript clients (Bun/Node/Deno); Windows and mobile-client support were not established. [MIT license](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/LICENSE); latest release observed: [v0.5.1](https://github.com/anomalyco/opentunnel/releases/tag/v0.5.1). Release availability is not production-readiness evidence.

## Tunnet: private QUIC mesh plus optional ingress

[Mesh documentation](https://docs.tunnet.io/products/mesh/) describes an IP overlay using QUIC/iroh rather than WireGuard, direct peer paths with NAT traversal and relay fallback. It includes DNS, subnet routes and exit nodes. The [installation guide](https://docs.tunnet.io/guide/installation) documents Linux, macOS and Windows, with privileges required for the TUN interface; native Android/iOS clients were not established.

[Managed mode](https://docs.tunnet.io/modes/managed/) provides a self-hostable control plane, management API and dashboard sharing PostgreSQL, with enrollment, SSO/OIDC, centralized ACLs and audit capabilities. Here “Managed” names the architecture, not proof of a vendor-hosted service. [Direct mode](https://docs.tunnet.io/modes/direct/) uses replicated CRDT membership and Mainline DHT discovery without a control plane. Signed grants, epochs and revocation gate transport access; firewall rules are local. Direct lacks SSO, centralized policy, public tunnels and dashboard, and cannot be mixed with Managed mode.

The [current README](https://github.com/tunnetio/Tunnet/blob/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce/README.md) marks the project in development and acknowledges a maturity gap against Tailscale/Cloudflare. Latest release observed: [v0.9.1](https://github.com/tunnetio/Tunnet/releases/tag/v0.9.1). It specifies component licensing: MPL-2.0 runtime/agent/SDKs; AGPL-3.0-only managed infrastructure; Apache-2.0 common/protocol/tooling components. The [root license](https://github.com/tunnetio/Tunnet/blob/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce/LICENSE) delegates exact paths to `LICENSING.md`, which returned 404 during review. Hosted docs still show an AGPL footer. Resolve this discrepancy and inspect file-level licenses before embedding or redistributing.

## Assessment: brokers and isolated support agents

**Assessment, not tested behavior:** OpenTunnel could publish an authenticated desktop broker for a phone browser, without installing a mesh client on the phone. Its public URL grants reachability, so application authorization, scope and lifecycle remain broker responsibilities.

Tunnet is a candidate for private agent-to-broker connectivity where supported clients and enforceable Managed ACLs fit the deployment. Direct mode's local firewall model needs separate evaluation for centrally enforced isolation. Neither connectivity choice provides process isolation or authorizes support actions; broker permissions and agent sandbox boundaries remain necessary. Native phone-mesh feasibility is unresolved. Evaluate these constraints before treating either project as a Tailscale replacement.
