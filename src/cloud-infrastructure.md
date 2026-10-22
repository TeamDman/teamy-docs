# Cloud infrastructure and infrastructure as code

Use these public references for different jobs: describe how infrastructure is deployed, or build tools for operating it. Reviewed on 18 October 2026 from public repository documentation only. Inclusion records prior art, not dependency adoption, a TeamyStyle requirement or a verified deployment. No cloud tool or Terraform operation was run for this review.

| Need | Reference | Purpose and evidence status |
| --- | --- | --- |
| Study Terraform bootstrap, delegated ownership and deployment pipelines | [Reference Architecture](#reference-architecture) | Azure/Azure DevOps example; source-reviewed patterns with external prerequisites. |
| Browse Azure resources, activate PIM or study Azure/Terraform operator tooling | [Cloud Terrastodon](#cloud-terrastodon) | Rust CLI/SDK; public overview linked to the existing implementation guides. |

## Reference Architecture

[AAFC-Cloud/Reference-Architecture](https://github.com/AAFC-Cloud/Reference-Architecture) is a public Azure/Azure DevOps infrastructure-as-code example. The [README at `ea612f1`](https://github.com/AAFC-Cloud/Reference-Architecture/blob/ea612f1da952e836d8093b241b2174dcd7ca4cd5/README.md) explains Terraform roots, deployment identities, scoped service connections, approval environments and pipelines. It demonstrates delegated infrastructure ownership, including VMSS build agents and workload state access.

The reference requires existing bootstrap state storage and a shared network, with an explicit bootstrap/deployment order. Adapt identities, backend access and approval requirements to the target environment. The repository's [license](https://github.com/AAFC-Cloud/Reference-Architecture/blob/ea612f1da952e836d8093b241b2174dcd7ca4cd5/LICENSE.txt) is the Québec Free and Open-Source Licence – Reciprocity (LiLiQ-R).

## Cloud Terrastodon

[AAFC-Cloud/Cloud-Terrastodon](https://github.com/AAFC-Cloud/Cloud-Terrastodon) is already referenced throughout this book for CLI and terminal implementation patterns. Its [public README at `ea90bf8`](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/ea90bf830a4d38c83aeb2d9cec76b2b07dcec4bb/README.md) describes a Rust CLI and SDK for Azure and Terraform: interactive browsing, PIM activation, subscription/network queries and Azure DevOps commands. Azure CLI is a documented dependency. Cached information can become stale before expiry.

For reusable engineering details, follow [argument parsing](cli-parsing.md), [structured logging](logging.md), [terminal ownership](terminal-ownership.md) and [async terminal work](async-terminal-ui.md). Those guides retain their own inspected revisions and limits. The public [manifest](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/ea90bf830a4d38c83aeb2d9cec76b2b07dcec4bb/Cargo.toml) declares LiLiQ-R-1.1; inspect the shipped [license](https://github.com/AAFC-Cloud/Cloud-Terrastodon/blob/ea90bf830a4d38c83aeb2d9cec76b2b07dcec4bb/LICENSE.txt) for reciprocity and redistribution terms.
