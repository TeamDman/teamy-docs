# Compare private meshes and public tunnels

Choose by reachability: a private mesh connects enrolled devices; a reverse tunnel makes a selected service reachable through a public endpoint. These are different trust boundaries. OpenTunnel belongs to the latter category; NetBird supplies a private mesh, and Tunnet offers mesh and ingress as separate capabilities. Tailscale is the comparison baseline named by Tunnet, but upstream competitor feature tables are claims to evaluate, not independent validation.

## Evidence and adoption status

OpenTunnel and Tunnet reviewed 14 October 2026; NetBird and the Azure Windows lab comparison added from official documentation on 16 October 2026. Documentation/source review only; none of these candidates has been installed, benchmarked, security-audited, or adopted by this survey. Inclusion is not endorsement; [TeamyStyle](teamystyle.md) remains the separate standards guide. Official repository snapshots: [OpenTunnel `0cad49c`](https://github.com/anomalyco/opentunnel/tree/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db) and [Tunnet `30dfeaa`](https://github.com/tunnetio/Tunnet/tree/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce). Linked hosted documentation may change independently.

Discovery provenance supplied for this survey: [OpenTunnel via thdxr on X](https://x.com/thdxr/status/2107956848318177622) and [Tunnet via orielhaim on X](https://x.com/orielhaim/status/2107984883964403791). These are discovery links, not technical evidence; their contents were not verified.

## Codex cloud private networking

Added **2026-10-23**. The official service contract, observations from the inspected Teamy setup and a future transfer design have different evidence. Do not apply the general mesh comparisons below as though a managed cloud task exposed every feature of an ordinary Tailscale installation.

### Officially documented access

OpenAI documents outbound access to private HTTP(S) services through the environment's configured proxy and Tailscale VPN. Destinations need both environment-network permission and VPN access rules. Setup changes should be published and checked in a new task. This does not document desktop-to-task port mapping. Source: [Cloud environments: private networking](https://learn.chatgpt.com/docs/environments/cloud-environments#private-networking-vpn).

[Tailscale's ephemeral-node documentation](https://tailscale.com/docs/features/ephemeral-nodes) explains automatic removal after inactivity and identifies ephemeral nodes with an admin-console badge. Ephemeral enrollment and key expiry are separate lifecycle properties; disabling key expiry does not turn an ephemeral node into a permanent one. See also [Tailscale auth keys](https://tailscale.com/docs/features/access-control/auth-keys).

### Observed managed-sidecar limits

The inspected setup used a managed Tailscale sidecar for **task-initiated outbound** HTTP(S), with approved raw TCP through CONNECT also described by its runtime guidance. That setup explicitly excluded inbound connections, UDP, QUIC, ICMP and transparent routing from the task. No supported desktop-to-task port-mapping surface was exposed. These are dated setup observations; raw CONNECT support is not claimed as a broader public product guarantee.

A node's appearance in a tailnet does not establish that a desktop can connect to a task listener. The observed tagged nodes were ephemeral. “Online” reports network presence, not an active task CPU or durable task storage. Before removing a node, establish which task or service owns it and whether it is still needed. Do not blindly delete unmapped nodes to tidy the device list.

### Keep the network gates separate

The destination allowlist, VPN ACLs/grants, application authentication and TLS trust each answer a different question. An allowed domain does not grant VPN permission, a VPN connection does not authorize an upload, and successful routing does not validate a server certificate.

In the inspected runtime guidance, raw TCP CONNECT was an opaque byte tunnel; the HTTPS proxy path could intercept TLS. A certificate-pinning design therefore needs qualification on the selected, approved path. Consult current runtime proxy guidance rather than assuming a direct socket works or that every CONNECT request has the same interception behavior. Do not use proxy bypasses or `--noproxy` to evade the configured route or policy.

Newly published settings were not observed to affect an existing task. Preserve its files and test a fresh task against the exact approved destination before changing more policy. Record the layer and outcome of each check: destination policy, VPN access, TLS identity and application response. A failed request at one layer does not diagnose all the others.

Changing which peer initiates a connection needs a separately reviewed, narrow grant for that source, receiver and port. A rule permitting desktop-initiated traffic does not automatically permit cloud-initiated traffic. Authorizing reply traffic on an existing connection is different from authorizing a new connection in the reverse direction.

### Proposed transfer direction

The [Teamy Mover proposal](moving-files.md#proposed-cloud-to-desktop-transfer) puts a receiver on the desktop and sends from the cloud over approved outbound access. Its direct adapter does not depend on task ingress; a desired future Veilid adapter needs separate network qualification. [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve) is a documented alternative for exposing a local service to a tailnet; it is retained as an optional reference, not the selected TLS-termination design or a tested integration. The direct adapter keeps TLS termination inside Rust. Both adapters remain unimplemented.

## OpenTunnel: public TLS ingress

The [official README](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/README.md) describes a blind TLS tunnel: public TCP/443 traffic is routed by TLS ClientHello/SNI through a bridge to the local client, which holds the certificate key, terminates TLS and forwards plaintext to a selected local port. It supports HTTPS use cases, but the protocol can carry other application protocols inside TLS; this is not an IP mesh VPN.

The [protocol](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/docs/protocol.md) uses an outbound WebSocket bridge and bearer-token tunnel ownership. This avoids requiring inbound reachability at the local machine; it is relay-based rather than peer-to-peer hole punching. A random route URL is **not visitor authentication**. The application must authenticate callers and must not trust requests merely because forwarding makes their peer appear local. There is no per-connection bridge flow control yet, so one slow reader can delay other connections.

Hosting is a Rust server on Fly.io with MySQL, ACME issuance and DNS integration. Server source and deployment configuration are included; operating a replacement requires those dependencies and a domain/certificate setup ([server documentation](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/docs/server.md)). The README lists Linux/macOS x64 and arm64 binaries, Rust and TypeScript clients (Bun/Node/Deno); Windows and mobile-client support were not established. [MIT license](https://github.com/anomalyco/opentunnel/blob/0cad49c74303bfdb2c0495e9c58fd1bdb9d619db/LICENSE); latest release observed: [v0.5.1](https://github.com/anomalyco/opentunnel/releases/tag/v0.5.1). Release availability is not production-readiness evidence.

## Tunnet: private QUIC mesh plus optional ingress

[Mesh documentation](https://docs.tunnet.io/products/mesh/) describes an IP overlay using QUIC/iroh rather than WireGuard, direct peer paths with NAT traversal and relay fallback. It includes DNS, subnet routes and exit nodes. The [installation guide](https://docs.tunnet.io/guide/installation) documents Linux, macOS and Windows, with privileges required for the TUN interface; native Android/iOS clients were not established.

[Managed mode](https://docs.tunnet.io/modes/managed/) provides a self-hostable control plane, management API and dashboard sharing PostgreSQL, with enrollment, SSO/OIDC, centralized ACLs and audit capabilities. Here “Managed” names the architecture, not proof of a vendor-hosted service. [Direct mode](https://docs.tunnet.io/modes/direct/) uses replicated CRDT membership and Mainline DHT discovery without a control plane. Signed grants, epochs and revocation gate transport access; firewall rules are local. Direct lacks SSO, centralized policy, public tunnels and dashboard, and cannot be mixed with Managed mode.

The [current README](https://github.com/tunnetio/Tunnet/blob/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce/README.md) marks the project in development and acknowledges a maturity gap against Tailscale/Cloudflare. Latest release observed: [v0.9.1](https://github.com/tunnetio/Tunnet/releases/tag/v0.9.1). It specifies component licensing: MPL-2.0 runtime/agent/SDKs; AGPL-3.0-only managed infrastructure; Apache-2.0 common/protocol/tooling components. The [root license](https://github.com/tunnetio/Tunnet/blob/30dfeaa92fe2c816e23277ca9d497d0c9326b6ce/LICENSE) delegates exact paths to `LICENSING.md`, which returned 404 during review. Hosted docs still show an AGPL footer. Resolve this discrepancy and inspect file-level licenses before embedding or redistributing.

## NetBird: private WireGuard mesh with a self-hostable control plane

Revisit [NetBird](https://netbird.io/) when comparing managed mesh access with an operator-owned control plane. Its [architecture](https://docs.netbird.io/about-netbird/how-netbird-works) uses peer WireGuard tunnels, a Management service for enrollment/policy/network state, Signal for connection negotiation, and Relay when a direct connection is unavailable. Management is not the traffic gateway. [Client firewall guidance](https://docs.netbird.io/about-netbird/ports-and-firewalls) documents outbound establishment and relay fallback; a client does not require public inbound RDP or a public WireGuard listener to join. Host firewall and access policies still determine which services are reachable.

The [Windows guide](https://docs.netbird.io/get-started/install/windows) supports silent EXE or MSI installation under administrator/`SYSTEM`, then CLI enrollment. The background service is separate from the tray UI and can keep the peer connected without a logged-in desktop. Check the selected image against the [Windows support matrix](https://docs.netbird.io/help/support-matrix/netbird-client/windows); its current entry lists Windows 10/11 and Server 2016 or newer for current clients. This is documented support, not a tested Azure image qualification here.

### Separate user identity from machine enrollment

[Setup keys](https://docs.netbird.io/manage/peers/register-machines-using-setup-keys) enroll unattended servers without interactive SSO. They can be one-off or reusable with usage/expiry limits and auto-assigned peer groups. Revoking or expiring a setup key prevents future enrollment; it does **not** disconnect already enrolled peers. Device removal/revocation therefore needs its own lifecycle step.

The [bootstrap guide](https://docs.netbird.io/manage/peers/bootstrap-via-config-file) distinguishes setup-key machines from user devices: a key-enrolled peer has no associated user identity and cannot use user-session reauthentication or user-based policy in the same way. Enroll human laptops through SSO. For a disposable lab server, evaluate a short-lived, one-off key assigned to a restricted lab group; that is a proposed enrollment design, not an adopted account configuration. Inject the key at runtime rather than embedding it in an image. Do not clone an enrolled client's state/private key into another VM. `netbird status --check startup` and `netbird status --json` provide documented startup/connection evidence, not proof that RDP authorization is correct.

### Choose who operates coordination

[NetBird Cloud](https://docs.netbird.io/about-netbird/self-hosted-vs-cloud) operates the coordination and relay infrastructure; self-hosting transfers upgrades, backup, security, availability and relay operation to us. Current self-hosting supports built-in local users with optional external OIDC providers; a separate IdP is no longer universally required. The [quickstart](https://docs.netbird.io/selfhosted/selfhosted-quickstart) describes a Linux VM, Docker Compose, public DNS and server ingress on TCP 80/443 and UDP 3478. Those are **server** requirements, not ports to open on every Windows client. Some capabilities require an [Enterprise Commercial License](https://docs.netbird.io/selfhosted/enterprise); inspect selected-version component licenses and feature terms before adopting or redistributing. “Self-hostable” does not mean maintenance-free or feature-identical to every SaaS plan.

For comparison, [Tailscale describes its own coordination server as proprietary and hosted](https://tailscale.com/opensource), while its core client and DERP relay are open source. Headscale is an independent, community-maintained self-hosted control plane for Tailscale clients, with its own scope and support boundary. Compare actual policy, identity and operational requirements rather than treating NetBird self-hosting as equivalent to the supported Tailscale SaaS or assuming Headscale is that same service.

NetBird's [Azure site-to-site article](https://netbird.io/knowledge-hub/getting-started-with-an-azure-site-to-site-vpn), dated July 2024, mainly demonstrates Azure VPN Gateway/IPsec and then describes NetBird as an alternative. It is not a Windows NetBird VM-extension recipe. Its current [cloud-to-on-premise guide](https://docs.netbird.io/use-cases/remote-access/cloud-to-on-premise) adds routing-peer considerations such as Azure NIC forwarding and NSGs. A single enrolled VM used only as an RDP destination does not need subnet routing merely because that wider-network guide uses it; this is a scope assessment, not an Azure deployment result.

## Bootstrap an Azure Windows lab VM, then choose the access path

A VM extension installs/configures software. NetBird or Tailscale supplies overlay reachability. RDP supplies a Windows desktop session. These are complementary layers rather than three interchangeable products.

| Choice | What the official guidance establishes | Practical boundary for a lab VM |
| --- | --- | --- |
| Windows Custom Script Extension + NetBird | Azure executes a noninteractive bootstrap as `LocalSystem`; NetBird supports silent Windows installation and setup-key enrollment. | Proposed combination, not an end-to-end recipe tested here. Verify installer exit, service readiness, restricted group membership and reboot persistence. No browser login in the extension. |
| Windows Custom Script Extension + Tailscale | Tailscale documents a Windows MSI and unattended operation; its Azure Windows guide recommends auth-key CLI login for a cloud VM. | The same bootstrap pattern can enroll the existing Tailscale lab. It does not require switching products to use an extension. |
| Linux cloud-init | Azure can process cloud-init on supported Linux images. Windows custom data is stored but not executed. | Do not paste a Linux shell/cloud-init example into Windows `custom_data` and expect installation. |
| RDP over the enrolled mesh | Tailscale documents connecting to a Tailscale IP/MagicDNS name without public IP or router port forwarding. | RDP must still be enabled on a supported Windows edition, permitted by host/mesh policy and authenticated with an authorized Windows account. Enrollment is not an OS login credential. |
| Direct public RDP | It supplies a route to the Windows RDP service without a mesh client. | It still needs Windows authentication and an explicitly bounded NSG/host-firewall exposure. For private-only access, use a mesh or a separately evaluated Azure private-access route instead of adding a public 3389 rule by default. |

Sources: [Azure Windows extension](https://learn.microsoft.com/en-us/azure/virtual-machines/extensions/custom-script-windows), [Azure custom data](https://learn.microsoft.com/en-us/azure/virtual-machines/custom-data), [Tailscale Azure Windows](https://tailscale.com/docs/install/cloud/azure/windows), [Tailscale Windows MSI](https://tailscale.com/docs/install/windows/msi), [Tailscale unattended](https://tailscale.com/docs/how-to/run-unattended), [Tailscale RDP](https://tailscale.com/docs/solutions/access-remote-desktops-using-windows-rdp), and [Windows RDP prerequisites](https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/remotepc/remote-desktop-allow-access).

The Azure extension requires idempotent scripts with no user prompts, has a 90-minute execution limit and does not resume through an inline reboot. Separate reboot orchestration from installation. Preserve native installer/enrollment errors as extension failure; a blanket success exit hides a broken bootstrap. Pin and verify the installer before execution, retain secret-free diagnostics, and qualify reruns against an already enrolled VM. These are proposed implementation checks; no script was run here.

For Tailscale, [unattended mode](https://tailscale.com/docs/how-to/run-unattended) keeps connectivity available after sign-out/reboot. The MSI exposes `TS_UNATTENDEDMODE="always"` and `TS_NOLAUNCH="1"`; CLI [`tailscale up`](https://tailscale.com/docs/reference/tailscale-cli/up) supports `--unattended` and an auth key read from a file. Its [Azure Windows guide](https://tailscale.com/docs/install/cloud/azure/windows) uses `--accept-dns=false` to retain Azure DNS: review that choice against the lab's name-resolution requirements rather than copying all routing/DNS examples. Host-only RDP does not require advertising the whole VNet. The [auth-key policy](https://tailscale.com/docs/features/access-control/auth-keys) needs its own expiry, approval and device-tag decisions; NetBird setup keys and Tailscale auth keys are analogous enrollment capabilities, not interchangeable credentials.

Plan outbound access even when public inbound access is closed. [Azure's current default-outbound guidance](https://learn.microsoft.com/en-us/azure/virtual-network/ip-services/default-outbound-access) says new VNets using API versions released after 31 March 2026 default to private subnets; earlier versions/existing networks differ. Installer downloads and public overlay coordination still need working internet egress, such as an approved NAT path; DNS resolution must also work. Azure-provided DNS is a platform service, so its availability is a separate check from NAT. “No public VM IP” does not imply “no internet egress needed.” An extension's successful installation or a peer's connected status is insufficient: test an allowed RDP connection, a denied unrelated peer/port, sign-out/reboot recovery and peer deletion.

### Keep Windows passwords and enrollment keys out of ordinary IaC output

Overlay enrollment does not authenticate the Windows RDP session. Windows authentication remains separate; a password-based VM still needs its OS credentials. Treat VM administrator credentials and mesh enrollment keys as separate secrets. Azure [protected settings](https://learn.microsoft.com/en-us/azure/virtual-machines/extensions/custom-script-windows#extension-schema) encrypt the secret-bearing extension configuration for delivery/decryption inside the VM; that is a different boundary from Terraform's storage.

[Terraform `sensitive`](https://developer.hashicorp.com/terraform/language/manage-sensitive-data) redacts normal CLI/UI presentation but can still persist values in plan/state. If a VM password or enrollment key flows through an ordinary state-backed resource argument, assume the state needs secret-level access controls, encryption and retention. `random_password` and Azure `protectedSettings` are not by themselves a state-exclusion mechanism. Ephemeral/write-only alternatives depend on the exact Terraform/provider/resource support and permitted use; do not assume they work for every VM/extension argument. A guest-side secret-manager retrieval flow can avoid putting the key value into Terraform configuration, but needs a separately reviewed identity/access design and must avoid logging the retrieved value. No credentials or state were read for this survey.

**Assessment:** for a lab already using Tailscale, first evaluate Windows extension-based unattended enrollment plus private RDP using that existing mesh. Evaluate NetBird separately when its control-plane ownership or policy model solves a stated need. This is a proposed comparison path, not an endorsement, migration decision or claim of tested deployment.

## Assessment: brokers and isolated support agents

**Assessment, not tested behavior:** OpenTunnel could publish an authenticated desktop broker for a phone browser, without installing a mesh client on the phone. Its public URL grants reachability, so application authorization, scope and lifecycle remain broker responsibilities.

Tunnet is a candidate for private agent-to-broker connectivity where supported clients and enforceable Managed ACLs fit the deployment. Direct mode's local firewall model needs separate evaluation for centrally enforced isolation. Neither connectivity choice provides process isolation or authorizes support actions; broker permissions and agent sandbox boundaries remain necessary. Native phone-mesh feasibility is unresolved. Evaluate these constraints before treating either project as a Tailscale replacement.
