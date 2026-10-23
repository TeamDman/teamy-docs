# Publication and audiences

`teamy-docs` contains unclassified documentation suitable for public distribution. Its source and GitHub Pages website are public. Apply that boundary to every chapter, generated search record, print view and copied asset.

## Public source boundary

This repository is for generic guidance and synthetic examples. Keep private review comments, inventories, credentials and machine-specific paths in separate local/private records. Unclassified describes the classification; public describes the audience. Information must meet both requirements to enter this book.

GitHub warns that an ordinary Pages site can be publicly accessible even when its source repository is private. Private Pages access is a separate Enterprise Cloud organization feature. Do not infer account eligibility from the repository name or source visibility.

Sources: [Creating a GitHub Pages site](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site), [Changing Pages visibility](https://docs.github.com/en/enterprise-cloud@latest/pages/getting-started-with-github-pages/changing-the-visibility-of-your-github-pages-site).

## Publish with GitHub Pages

The public source is [TeamDman/teamy-docs](https://github.com/TeamDman/teamy-docs). The static site is [Teamy Docs](https://teamdman.github.io/teamy-docs/).

The deployment workflow runs on a push to `main` or manual dispatch. Its build job verifies the pinned mdBook archive, builds the book and checks local links. A separate deployment job publishes only the generated `book/` artifact. Repository source, downloaded tools and local review records are not part of that site artifact.

Review the source and outgoing commit metadata before pushing. Changes pushed to `main` are published automatically. The site has no sign-in or private-documents area.

**Working policy, 2026-10-23:** publish the exact reviewed and tested commit. Record its full SHA, build/link results and any unrun checks; if integration changes that commit's content, validate the resulting candidate again. A local feature-branch commit is neither a remote backup nor a deployed site. Desktop/cloud handoff goes through GitHub with explicit branch selection; [cloud-environment guidance](cloud-environments.md) covers the separate prepared setup and task state.

Prefer meaningful local/cloud validation over routine CI churn, while retaining the existing Pages publication workflow. CI checks, release jobs, Pages and security automation have distinct responsibilities; reducing redundant checks does not justify disabling all automation.

The workflow does not compile every example. See [plan records and validation](plan-records.md#know-what-the-book-validates) for its current guarantees and the proposed executable-example harness. Private plans and comments can produce a separately approved public decision record; they are not automatically site content.

The default address uses `/teamy-docs/`; `book.toml` records that path so generated 404-page asset links work there. Connecting `docs.teamdman.ca` remains a separate DNS task.

Source: [GitHub Pages custom workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).

## Plan the custom domain through existing IaC

Prefer the existing Terraform/OpenTofu and 1Password pattern for managing `docs.teamdman.ca`, while retaining GitHub Pages as the book host. This is a planning reference reviewed on 7 October 2026; no credentials were retrieved, state refreshed, Terraform commands run, DNS changed or site published during the inspection.

### Separate registration, delegation and host records

The SFM website reference uses `sfm-website/iac/terraform.tf`, `provider.namecheap.tf`, `resource.namecheap_domain_records.main.tf` and `Load-NameCheap-Config.ps1`. It pins `namecheap/namecheap` 2.3.4 with an empty provider block and an Azure state backend. Its Namecheap resource sets `superfactorymanager.ca` nameservers from an Azure DNS zone. It is a registrar-delegation example, not a recipe for adding one CNAME to Namecheap DNS. Public NS queries showed Azure DNS for that domain and `registrar-servers.com` for `teamdman.ca`; registration at Namecheap does not imply both domains use Namecheap as authoritative DNS.

Retain the SFM website as prior art and revisit its website/IaC separately later, alongside its ongoing migration. No SFM changes or website migration are part of the docs-domain plan. The helper filename in its README is stale (`set-namecheap-vars.ps1`); the inspected script is `Load-NameCheap-Config.ps1`.

Older teamdman IaC under `old/old/namecheap/main.tf` contains literal sandbox placeholders, including `use_sandbox = true`; it does not establish current production ownership. The Raddest reference uses the same environment-based provider pattern, an older 2.2.0 provider, and a Namecheap-to-Azure nameserver resource. Locate the current root/state that owns `teamdman.ca` before adding a resource; do not copy a legacy backend, introduce competing ownership or move nameservers just to publish a docs subdomain.

### Authenticate through process environment variables

The SFM helper captures three `op read` results into environment variables; it optionally accepts a client IPv4 address. It instructs callers to dot-source it in the shell that will launch OpenTofu/Terraform. The tool and its provider subprocess inherit that process environment. The [2.3.4 provider schema](https://github.com/namecheap/terraform-provider-namecheap/blob/v2.3.4/namecheap/provider.go) maps:

| Environment variable | Provider field | Source-defined role |
| --- | --- | --- |
| `NAMECHEAP_USER_NAME` | `user_name` | Namecheap account username. |
| `NAMECHEAP_API_USER` | `api_user` | API username. |
| `NAMECHEAP_API_KEY` | `api_key` | API key retrieved from 1Password. |
| `NAMECHEAP_CLIENT_IP` | `client_ip` | Explicit public client IPv4 address; provider default is `0.0.0.0`. |
| `NAMECHEAP_USE_SANDBOX` | `use_sandbox` | Sandbox selection; provider default is false. The helper does not set it. |

Namecheap authenticates using API username/key and requires enabled API access plus an allowlisted IPv4 address. Confirm those prerequisites and the effective sandbox setting before any authorized operation. See [Namecheap API authentication and IP allowlisting](https://www.namecheap.com/support/api/intro/). An authenticated 1Password CLI session is also required, but the helper does not show how that session is established. Namecheap access does not authenticate the separate Azure backend or Azure DNS provider.

The observed script uses `op read`, not `op run`; it captures plaintext secret values in the shell environment and does not clear them afterward or explicitly check each native command's exit status. Keep private vault/item references outside this public book. Do not print `op read` results, enumerate the environment, persist keys in `.tf`/`.tfvars`, or include secrets in logs or artifacts. Verify successful retrieval without displaying values and limit the credential-bearing process lifetime.

A future wrapper could use secret references with [`op run`](https://www.1password.dev/cli/reference/commands/run) to make resolved credentials available only to the child process. That is a candidate improvement, not existing helper behavior or a guarantee that Terraform logs/state are secret-free. [`op read`](https://www.1password.dev/cli/reference/commands/read) ordinarily returns the field value on stdout. Authentication, credentials and wrapper execution were not tested here.

### Keep the DNS change bounded

The intended host record is CNAME `docs` pointing to `teamdman.github.io`, under root domain `teamdman.ca`. Verify the domain in the GitHub account and configure the repository's Pages custom domain before pointing DNS at Pages. Its Actions workflow does not require a CNAME file. Change mdBook's `site-url` to `/`, review public content, build/check links and publish only the approved changes. Finish with DNS, certificate, HTTPS, search, navigation and 404 checks. See [GitHub custom domains](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site) and [domain verification](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/verifying-your-custom-domain-for-github-pages).

The [provider's record resource](https://github.com/namecheap/terraform-provider-namecheap/blob/v2.3.4/docs/resources/domain_records.md) defaults to `MERGE`; `OVERWRITE` removes records omitted from its configured record set. The underlying [Namecheap setHosts API](https://www.namecheap.com/support/api/methods/domains-dns/set-hosts/) replaces host records omitted from the request. Preserve existing apex/www, MX, TXT, verification and other records. Do not transplant the SFM nameserver-delegation resource into a one-record change. Choose one owner, inspect its complete intended effect and review a future plan before an authorized apply. MERGE still needs checked behavior and does not resolve competing writers automatically.

Remaining prerequisites are identifying current IaC/state ownership, confirming API/1Password/backend access without exposing secrets, obtaining the reviewed DNS change and approval for live DNS/Pages/publication operations. No such live operation is authorized merely by recording this plan.

## Private documentation belongs elsewhere

Private documentation needs separate permitted inputs and separate generated output, outside `teamy-docs`. Build only what its target audience may read; do not build a mixed book and attempt to redact it afterward.

Future authenticated hosting must protect every asset and access path, including search data and default hostnames. Signing in proves an identity; an audience policy still determines whether that identity may read the material. Choose and verify that hosting separately from the static-book authoring workflow.

GitHub Pages serves the public book. Identity providers, private hosting, DNS and other cloud resources require their own design and setup.

## Review what is actually shared

Before publication, review source, generated output and outgoing commit metadata. Keep the workflow insight while replacing incidental machine details with project-relative paths or clearly synthetic placeholders. Review licensing and attribution for copied material.

mdBook copies non-Markdown source files into the output directory, so adding a local file under `src/` can publish it even without a chapter link. Its configuration can reject missing chapters with `create-missing = false`.

Sources: [mdBook build behavior](https://rust-lang.github.io/mdBook/cli/build.html), [mdBook build configuration](https://rust-lang.github.io/mdBook/format/configuration/general.html).
