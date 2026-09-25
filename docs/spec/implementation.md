# Implementation and deferred work

[Specification index](README.md)

This checklist preserves unresolved work without reopening accepted behavior. It does not create tasks or authorize implementation. Detailed contracts must be specified per feature using the [API gap review](../redesign/ROTT-API-GAP-REVIEW.md), rather than designing all endpoints upfront.

## API feature work and ownership

**SlugKit owns implementation of every new shared API/schema addition or extension.** Optional website backends implement corresponding storage/federation; the ROTT Rust core consumes and validates the contract.

| Work | SlugKit contract | Website implementation | ROTT integration |
| --- | --- | --- | --- |
| Identity | Extend existing auth-check response; account/site-wide-key mapping and optional actor address | Return configured identity correctly | Bind one current configuration, preserve future switching, per-device secure credentials |
| Catalog discovery | Existing search contract; verify name/handle discovery fit | Adopt/deploy search missing from reviewed website/schema | Search associations, stable refresh, offline metadata and unavailable accounts |
| Capabilities/doctor | Select report location/fields/versioning/schema mapping and consistent unsupported/error behavior | Explicitly report supported optional operations | Read-only compatibility, credentials/account and actor reachability checks |
| Incoming downloads | Combined stream, external/stable identifiers, post-follow boundaries, pagination/cursors, exact-copy removal and retries | Incoming store, optional ingestion, keep-copies behavior and learned source changes | Explicit Download, successful local persistence before separate removal, deduplication and failures |
| Remote conversations | Retrieval and thread-following contracts, future replies, learned edit/deletion data | Retrieve/retain/deliver supported thread data and source changes | Participation follows thread, retains text/context and updates learned comments |
| Like/Share | External-item references distinct from post slugs/follow IDs; optional semantic action and state/record contracts | Minimal references/action state and optional delivery | Pending/confirmed reconciliation, Liked posts, distinct private save/Share behavior |
| Quotes and remote replies | Relationship fields, quote support/permissions, unsupported behavior | Optional publication/federation to remote targets | Explicit intended action; no silent quote-to-link substitution |
| Follows and owned posts | Reuse verified existing operations; refine only demonstrated gaps | Validate existing Follow/Undo, Accept/Reject and publication delivery behavior | Online follow management/reconciliation, own feed, edit/delete and engagement |

Validate deployed search and authenticated feature behavior separately when authorized. This specification and the gap pass performed no authenticated live operations. Schema/code coverage must not be presented as successful remote delivery. No new route names, capabilities location, ID format, receipt format or batch size is chosen here.

## Engineering work within accepted behavior

- Define core interfaces and per-platform adapters, bindings, asynchronous cancellation/events/error handling, packaging and secure credential integration. Choose Apple bridge and Windows/Linux toolkit during implementation.
- Design iPhone navigation, lifecycle and optional sync scheduling without assuming three panes or a continuously running process.
- Define internal subscription/post/candidate/saved identities and relationships, Automerge schemas/document boundaries, bounded tracking rotation/retirement, offline-peer handling and conflict behavior. Preserve protected content and read-wins behavior.
- Validate the selected private Automerge sync server's connection/protocol/version compatibility. Do not equate compatibility with an export/backup design.
- Specify received activity inclusion, external-link extraction/URL normalization and review-queue mechanics; prototype representative inputs. Preserve different posts sharing the same URL.
- Implement batch/follow-boundary cursors, partial failures, retry/idempotence and stable deduplication, with persistence before exact-copy removal. Address source changes after copy removal; measure storage/cost effects rather than assuming them.
- Implement durable thread capture and automatic note/save record mechanics; test the effectiveness of following participated threads. Retain updated comment text without requiring a product revision-history UI.
- Choose cache technology, avatar caching, attachment local-file retention/eviction and fetching details within the no-Automerge-binaries boundary.

## Remaining product/interface details

- Choose follow-management placement while preserving discoverable pending cancellation and rejected/failed status.
- Choose detailed reader/default-filter layout and Liked posts placement; the three-pane desktop direction and All Unread are proposals, and My Feed is a provisional name.
- Specify the default publishing audience and presentation of unsupported/permitted quotes. Audience controls remain deferred.
- Decide retention protection for liked content and whether liking also affects explicit saved status; pending/confirmed action reconciliation remains API/client work.
- If Bluesky is introduced, decide public-reading-only versus separately authenticated following. No Bluesky publication or infrastructure deployment is committed.
- Choose readable-feed versus external-navigation presentation and supported platform app links.

## Deferred or uncommitted scope

| Area | Boundary |
| --- | --- |
| Notifications, content warnings, polls, custom account lists/timelines, full-text received/saved-post search | Deferred; author notifications caused by Like remain required social behavior |
| Audience controls | Deferred; default audience still needs a decision |
| Publishing media uploads and alt text | Uncommitted; downloaded attachment storage is already decided |
| Mute, block, moderation filtering | No separate product decision |
| Hosted sync authentication/device authorization/pairing/Keyhive | Defer until a hosted service is considered; personal VPN setup remains selected |
| Backup, export and recovery | Deferred; data should not be trapped, with no initial feature/format/timing commitment |
| Migration | Later separate helper using ROTT facilities if pursued; no initial built-in importer |
| Additional messaging channels | Private/direct ActivityPub, email, SMS and XMPP are exploratory, not integration commitments |
| Historical ActivityPub backfill | Outside initial scope |
| Multiple configurations UI or combined catalogs | Deferred/outside initial scope; identity model permits future switching |
| Monorepo, standalone core publication, Forgejo hosting | Not finalized; no repository migration authorized |
| Peer-to-peer sync or future TUI | Possible future work, not implemented/committed |

## Explicit exclusions and invariants

ROTT does not author articles or archive/reformat full webpages. It does not automatically download website incoming content on launch/in the background, queue offline follows, merge distinct posts by shared destination URL, sync credentials or attachment binaries through Automerge, or silently replace quotes with link posts. It must not let temporary-cache eviction remove saved items or communication history. Doctor must remain read-only. Private Save/notes do not publish or upload originals.

The original CLI/TUI remains isolated and usable. No redesign implementation, task creation, commit or website mutation is authorized by this documentation work.
