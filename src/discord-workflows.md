# Read Discord through the broker

For Teamy Discord work, use **teamy-discord with teamy-broker**, not browser or desktop UI automation. This is the preferred assistant entry point for reading a thread, checking mentions, or preparing a reply. If the CLI cannot perform a needed operation, report the specific gap rather than silently switching to a signed-in Discord session.

Teamy Docs explains the workflow and preferences; the repositories own command syntax and protocol details. Check their current help and READMEs before executing a command:

- [teamy-discord: Use from agents](https://github.com/TeamDman/teamy-discord/blob/58168b99034a8131fcd180f2f19cdff0b27765d5/README.md#use-from-agents)
- [teamy-broker: SDK and approval boundary](https://github.com/TeamDman/teamy-broker/blob/c1e71fafef9b82b98cfc5f0f6ae0a5fc17149338/README.md)

These are inspected source snapshots, not a claim that every installed binary matches them. Use the repositories' current README and `--help` when versions differ.

## Start with the requested operation

1. Identify the requested guild, channel or thread and whether the task is to read, draft, or send. A request to propose replies does not authorize posting them.
2. Use an existing configured profile on the authorized Windows computer. Do not copy its account names, vault references or other private configuration into public documentation.
3. Check the installed command surface. Group options go before the leaf command; message selectors and limits go after it.
4. Make a bounded read through teamy-discord. Let the broker present credential approval when required.
5. Inspect the relevant conversation and attachments, then return the requested result. Keep proposed replies in the assistant conversation until sending is explicitly authorized.

The following profile and IDs are synthetic; substitute the selected existing profile and the thread the user supplied:

```powershell
teamy-discord --version
teamy-discord message --help
teamy-discord message --profile demo --plan list --thread-id 200000000000000002 --limit 100
teamy-discord message --profile demo list --thread-id 200000000000000002 --limit 100
```

A Discord thread URL has the form `https://discord.com/channels/<guild-id>/<thread-id>`. Use the second ID as `--thread-id`. `--channel-id` and `--thread-id` are mutually exclusive. Explicit IDs override profile defaults; profiles select defaults, not permissions.

`--plan` previews resolution without requesting credentials or contacting Discord. It cannot prove the bot currently has permission. `profile list` and `profile show demo` inspect local defaults; treat their output as private configuration rather than material to publish.

## What the credential path does

```text
assistant -> teamy-discord -> teamy-broker approval and credential lease
                         -> Discord API -> JSON results
```

The Windows memory provider requests credentials through the broker SDK. The broker identifies the requested account/item, application, purpose and destination. An approved lease lasts a fixed 15 minutes; reusing an identical unexpired request does not extend its deadline. An approval for one request does not imply that every different request will reuse it.

Token bytes pass through local IPC into the consumer's memory. They are not printed in command results or stored in Discord profiles or a token cache. Do not extract tokens into chat, shell arguments, checked-in files or troubleshooting logs. Help, profiles and plans do not acquire credentials.

The broker is a credential supplier, not a Discord request proxy. It does not enforce Discord routes or message bodies, and approval metadata does not narrow the bot's Discord permissions. The CLI and any separate consumer must enforce their own operation and target constraints. This is a cooperative same-user boundary, not protection against malicious programs running as that user. SDK zeroization and expiry do not guarantee erasure of every downstream copy or recall an already-sent request.

## Reading enough context

Message listing returns one page, with a default of 100 messages and a limit clamped to 1 through 100. An empty or short page is not proof that all history has been examined. Use `--before` with an RFC3339 timestamp when earlier context is needed. It does not accept a message-ID cursor.

`attachment list` returns attachment metadata; it does not download file contents. A code attachment may contain the important context missing from a short excerpt, so retrieve and inspect the relevant attachment through an authorized route before diagnosing it. Do not substitute a broad archive sync for a narrow thread read: archive sync has separate credential handling and a wider scope.

Useful related commands, using synthetic defaults:

```powershell
teamy-discord thread --profile demo list
teamy-discord attachment --profile demo list --limit 20
teamy-discord message --profile demo list --mentions me --limit 10 --scan-limit 300
```

Thread listing returns active threads, not a complete inventory of archived threads. In mention queries, `me` means the authenticated bot, not its owner. JSON results distinguish the fetched/scanned window from returned matches. These limits matter when summarizing what was checked.

## Denials, expiry and incompatible installations

- If the user denies credential approval, stop that operation. An explicitly corrected accidental denial can authorize a new request; do not automatically resubmit prompts until one is accepted.
- If a lease expires or a protocol is incompatible, report the error and use the documented acquisition or upgrade workflow. Do not silently choose another credential source.
- An inaccessible broker endpoint can indicate a different Windows user or execution context. Check the actual command context and the documented broker error; do not change ACLs or start a replacement broker to bypass the boundary.
- An installed binary may lag the checkout. Check `--help` and `--version`; the latter includes a Git revision but cannot describe uncommitted changes. Do not rebuild or reinstall unrelated tooling just to conceal a command mismatch.

The inspected broker SDK requires Windows for broker-backed IPC. teamy-discord's Linux support includes other modes, but an explicitly injected token is not a reason for an assistant to bypass the preferred broker workflow.

## Drafting versus posting

The inspected teamy-discord CLI is **read-only for Discord operations**: it has no send, edit or delete commands. Its profile, archive and watcher operations can still change local state.

Earlier agent posting used a separate consumer that acquired a broker lease through IPC and then called Discord REST. That is historical prior art, not an available CLI send command or a standing grant to post. A future or existing posting consumer must verify the approved destination, content, action and live lease independently. Do not infer send authorization from read approval or from a request to draft a reply.

See [delegate only the authority an action needs](delegated-authority.md) for the broader capability model, and [takeout archives](takeout-archives.md) for offline data formats rather than live Discord access.
