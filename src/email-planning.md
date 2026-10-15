# Plan an owned email CLI

`teamy-email.exe` is a proposed Rust CLI for inspecting, composing and eventually submitting and synchronizing email. Start from [teamy-rust-cli](rust-cli-template.md), with [Facet/Figue arguments](cli-parsing.md), typed [results](output-shape.md), tracing and cooperative [cancellation](cancellation.md). This chapter records source-defined prior art and possible stages; no email implementation or dependency adoption is established here.

Revisit it when we need an offline `.eml` inspector, a message composer, an SMTP submission client, mailbox synchronization, or control of our email protocol stack. SMTP transports messages; IMAP accesses and synchronizes stored mail. Message syntax and MIME are a separate layer from either protocol.

## What Makepad already provides

Reviewed on 7 October 2026 against public Makepad revision [`3b1be7023`](https://github.com/makepad/makepad/tree/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1). Inspection covered the mail application, its manifest and the networking/base64 components; it was not an exhaustive audit of every repository file. No builds, tests, mailbox reads or email connections were performed.

| Purpose | Source and API | Evidence and limits |
| --- | --- | --- |
| Decode an existing message | [MIME decoder](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/apps/mail/src/mime.rs): `makepad_mail::mime::parse(raw)` returns `Result<MailMessage, String>`; `attachment(raw, index)` extracts a part; `decode_words(text)` handles encoded header words. | Decodes multipart messages, base64/quoted-printable and text/HTML. Lives inside the mail app and depends on Makepad widgets, HTML and presentation code. No RFC message serializer was found in the inspected components. |
| Separate discovery from indexing | [Source contract](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/apps/mail/src/source.rs): `MailReader`, `MailSource`, `MailMessage`, `SourceBatch`. | Bounded batches, revisions, cursors and removals are useful adapter patterns. The implemented [Apple Mail adapter](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/apps/mail/src/apple_mail.rs) reads files; a Gmail adapter appears as prospective commentary. |
| Draft and folder behavior | [Application engine](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/apps/mail/src/engine.rs): recipient parsing and `send_draft`. | Sending changes local state to Sent and reports "Saved to Sent · Demo". It does not perform SMTP delivery. Existing tests cover application behavior; they are not SMTP interoperability evidence. |
| Reuse a transport library | [makepad-network](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/platform/network/Cargo.toml) and [SocketStream](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/platform/network/src/socket_stream.rs): `connect(host, port, use_tls, ignore_ssl_cert)`, `into_tls`, `Read`/`Write`. | Separate CLI-capable library for TCP/TLS, HTTP and WebSockets. No SMTP or IMAP client/server was found in the inspected components. The existence of an API does not establish backend support. |
| Encode/decode base64 | [makepad-base64](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/libs/base64/src/base64.rs): `base64_encode`, `base64_decode`. | Independently reusable encoding primitive; not MIME formatting, authentication or encryption by itself. |

The MIME decoder bounds headers to 256 KiB, nesting depth to 16 and visited parts to 4096. These are its implementation limits, not email standards or a complete decoded-byte budget. It normalizes message fields, tolerates incomplete parts and can decode text lossily. Windows charset support is limited compared with the macOS adapter. Preserve original bytes separately if exact archival fidelity matters. No inline MIME tests were found in the inspected module; application tests do not establish full MIME conformance.

The [Windows stream backend](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/platform/network/src/backend/windows/socket_stream.rs) uses WinRT for TLS at connection time, but `into_tls` returns unsupported and its read/write timeout setters are no-ops. It therefore needs further work or a different adapter for STARTTLS and bounded deadlines. Linux wraps OpenSSL; Apple wraps Secure Transport. In the inspected [Linux wrapper](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/platform/network/src/backend/linux/socket_stream.rs), chain verification and SNI are configured, but no hostname-verification call was found. This is a specific inspection finding, not a complete TLS audit. Networking tests include plain TCP and HTTP/WebSocket paths; their presence does not establish email or certificate-negative coverage.

Makepad's [root license](https://github.com/makepad/makepad/blob/3b1be7023ddfe7e8f56161cbf34c66c103ed37e1/LICENSE) is MIT. The networking and base64 manifests declare MIT OR Apache-2.0. Keep applicable notices when reusing source; inspect individual dependencies before adoption.

## Correct the SMTP versus IMAP reference

The reviewed repository is [jonhoo/rust-imap at `6fe22ed11`](https://github.com/jonhoo/rust-imap/tree/6fe22ed11a1ccffe1799a73f48e589366d8d100f), an IMAP client. Its [manifest](https://github.com/jonhoo/rust-imap/blob/6fe22ed11a1ccffe1799a73f48e589366d8d100f/Cargo.toml) identifies package `imap` version `3.0.0-alpha.14` and license Apache-2.0 OR MIT. `lettre = "0.11"` is a development dependency: [integration tests](https://github.com/jonhoo/rust-imap/blob/6fe22ed11a1ccffe1799a73f48e589366d8d100f/tests/imap_integration.rs) use its SMTP transport to seed mail. SMTP is not rust-imap's implemented product.

Its [client API](https://github.com/jonhoo/rust-imap/blob/6fe22ed11a1ccffe1799a73f48e589366d8d100f/src/client.rs) exposes `Client::{login, authenticate}` and `Session::{select, fetch, uid_fetch, idle, append, ...}`. `Authenticator` provides challenge/response integration. The [builder](https://github.com/jonhoo/rust-imap/blob/6fe22ed11a1ccffe1799a73f48e589366d8d100f/src/client_builder.rs) defaults to `AutoTls`, requiring encryption; `Auto` allows plaintext fallback. Default TLS uses native-tls, with optional rustls support. These are reference designs, not adopted dependencies or exercised account integrations.

## Own the domain and protocol layers

Own typed addresses, envelopes, messages, MIME parts, parse diagnostics, submission results and outbox records. Keep the SMTP envelope (`MAIL FROM` and `RCPT TO`) distinct from visible `From`, `To` and `Cc` headers. Bcc recipients belong in delivery decisions without exposing a Bcc header in the transmitted message. Facet models describe our versioned contracts; RFC message serialization requires its own encoder rather than JSON output.

Own the SMTP state machine, capability negotiation, response framing and authentication orchestration. Define supported extensions explicitly, including what happens when SMTPUTF8 or 8BITMIME is required but unavailable. Keep TLS, certificate validation and cryptographic primitives behind a mature implementation such as rustls or platform TLS. Owning email behavior does not require inventing encryption. Authentication mechanisms and OAuth token acquisition/storage are separate concerns from transport encryption.

## Start with offline, inspectable behavior

All stages below are proposed and unimplemented. Example command names describe a possible surface, not runnable commands.

| Stage | Proposed capability | Acceptance evidence |
| --- | --- | --- |
| 1. Inspect and compose | `teamy-email message inspect` reads an explicitly supplied `.eml`; `message compose` writes a message from typed inputs. Begin with a documented text-message subset. | Synthetic fixtures, stable typed diagnostics, original-byte retention, encoder/decoder checks and independent expected RFC bytes. No account discovery or network access. |
| 2. Exercise SMTP offline | A transcript harness runs greeting, EHLO, MAIL FROM, RCPT TO, DATA and completion against synthetic peers. | Fragmented reads, multiline replies, CRLF framing, dot-stuffing, injection rejection, unsupported capabilities, partial recipients, failure and cancellation fixtures. This validates a state machine, not Internet interoperability. |
| 3. Secure submission | Submit through an explicitly configured authenticated provider with implicit TLS or required STARTTLS. | Certificate/hostname rejection tests, enforced encryption, deadlines, redacted diagnostics and a reviewed account integration. Distinguish provider acceptance from final delivery. |
| 4. Synchronize mail | Add an owned IMAP adapter behind a source/index boundary. | Literal framing, unsolicited responses, stable UIDs plus UIDVALIDITY, resumable sync and explicit read-only versus mutating commands. A UID alone is not permanent mailbox identity. |

Follow the template's subcommand directory modules and descriptive `*_cli.rs` files. Parse once, create the runtime and logging once, pass cancellation and output sinks into domain code, and keep UI/storage adapters separate from protocol parsing.

## Offer useful arguments before interchange files

The command surface should accept practical arguments and construct shared typed requests underneath. Callers should not need to write a request JSON file or learn its schema to compose or send email. Optional JSON interchange can reuse those types later, with the same validation; it is not the primary interface. Typed output remains useful to both people and scripts.

These commands and option names are proposals, not implemented behavior or a stable contract:

```text
teamy-email message inspect draft.eml
teamy-email message validate draft.eml
teamy-email message compose --from sender@example.invalid --to reader@example.invalid --subject "Example" --body "Hello" --output draft.eml
teamy-email message attachment list draft.eml
teamy-email message attachment extract draft.eml --part 2 --output attachment.bin
teamy-email message attachment add draft.eml --attach report.pdf --output revised.eml
teamy-email message attachment remove draft.eml --part 2 --output revised.eml
teamy-email message send draft.eml --profile example
```

Composition accepts repeatable `--to`, `--cc`, `--bcc` and `--attach`. Choose one body source: `--body`, `--body-file path`, or `--body-file -` for stdin. Define missing-body and overwrite behavior explicitly. Attachment commands belong under `message`; extraction chooses an explicit output path, while add/remove produce a revised draft. Attachment composition can follow the first text-only milestone without delaying it.

### Explain delivery without requiring an envelope file

The SMTP envelope is the delivery instruction exchanged with the server: `MAIL FROM` identifies the return address used for delivery failures; each `RCPT TO` identifies an actual recipient. The message headers are the content recipients see: `From` names the author, and `To`/`Cc` display addressees. Envelope and headers often agree, but they serve different purposes. A hidden Bcc recipient is a delivery recipient without appearing in the transmitted headers. See [SMTP mail objects](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.3.1) and [message destination fields](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.6.3).

Ordinary `message send draft.eml --profile example` should derive recipients from the draft's To/Cc/Bcc fields, validate and deduplicate them, and remove Bcc from the wire message. A private draft may retain Bcc for later sending; its bytes and any associated metadata are sensitive and must not be mistaken for the sanitized outgoing message. Do not modify the original draft merely to strip its outgoing Bcc field.

Proposed delivery overrides are arguments: `--envelope-from address` changes only the SMTP return address, and repeatable `--recipient address` replaces the entire derived recipient set when supplied. It does not append to To/Cc/Bcc or rewrite visible headers. Without those overrides, derive a single validated sender from From and recipients from the draft; reject ambiguous or unsupported inputs with a useful diagnostic. Preview/report the effective delivery addresses and require provider policy to permit the sender. No mandatory `--envelope request.json` is proposed.

### Complete the folder operations when synchronization arrives

The later IMAP surface should include `folder list`, `folder show`, `folder create`, `folder rename`, `folder delete`, `folder subscribe` and `folder unsubscribe`, with explicit folder arguments and profile selection. Include `message move` and `message copy` with an explicit destination folder alongside later `message list`, `message show`, `message search` and `sync` operations. Folder subscriptions describe server subscription state; they are distinct from deleting a folder or moving its messages.

Folder deletion and message removal need separate documented semantics for the selected server: name whether an operation moves to trash, marks a message deleted, permanently removes a mailbox, or expunges messages. Do not hide an expunge or irreversible deletion inside a read/sync command. Expose the affected folder/messages and irreversible effects before destructive work; define behavior when the server lacks a required operation rather than silently choosing a more destructive fallback.

## Protocol and security decisions to retain

- Reject CR/LF injection in structured addresses and header inputs. Fold and encode generated headers deliberately; preserve the distinction between raw imported bytes and validated structured input. Bound raw and decoded bytes, header lengths/counts, MIME nesting/parts and attachment sizes. Do not extract attachment filenames as unchecked paths or fetch remote HTML content during inspection.
- Track each RCPT outcome. Partial recipient acceptance must have an explicit policy and result; one success flag cannot represent it. SMTP reply classes, command sequencing and DATA dot-stuffing follow [RFC 5321](https://www.rfc-editor.org/rfc/rfc5321.html); message syntax follows [RFC 5322](https://www.rfc-editor.org/rfc/rfc5322.html) and [MIME](https://www.rfc-editor.org/rfc/rfc2045.html).
- A connection lost after DATA transmission can leave acceptance unknown. Persist an ambiguous outcome and reconcile or explicitly choose a retry policy; blindly retrying can duplicate mail. A stable Message-ID is useful correlation, not a server-enforced exactly-once guarantee.
- Require encrypted, authenticated transport with hostname verification before releasing credentials. STARTTLS and implicit TLS need separate adapter support; never silently fall back to plaintext. Keep pre-upgrade buffering and post-upgrade capability state correct. See [SMTP STARTTLS](https://www.rfc-editor.org/rfc/rfc3207.html) and [email submission/access TLS](https://www.rfc-editor.org/rfc/rfc8314.html).
- Keep passwords and OAuth tokens out of command-line arguments, serialized plans, logs and SMTP transcripts. Select a credential-store boundary separately; redact authentication exchanges and treat message bodies as sensitive diagnostic material. Cancellation after transmission cannot recall a message.
- Use [IMAP4rev2](https://www.rfc-editor.org/rfc/rfc9051.html) as protocol reference when synchronization becomes necessary. A client and a mail server are different scopes: direct-to-MX delivery, relay policy, queues, bounces, abuse prevention and deliverability require separate decisions. They are not implied by planning a submission CLI.
