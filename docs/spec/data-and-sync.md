# Data, retention, and sync

[Specification index](README.md)

## Data ownership

Automerge must store local personal data and support optional multi-device sync. Website publication and authoritative social state remain on the website; local communication history does not replace either website state or federation.

| Data | Storage and authority |
| --- | --- |
| Sources/contacts/account catalog | Website authority; full-catalog cache arrangement remains implementation work |
| Basic subscribed-account metadata | Automerge name/handle/type/address/avatar information/last known follow status for offline lists |
| RSS subscriptions and local selections | Automerge, owned by ROTT |
| ActivityPub intent/submitted-request status | Automerge snapshot; server-confirmed follow state remains authoritative; no offline follow queue |
| Website address and account identity | Non-secret Automerge connection data |
| Credentials | Per-device platform secure store; never Automerge |
| Downloaded website incoming content | Automerge, temporary unless saved or protected as communication history |
| Read/unread state | Automerge, distinct from fetching and content retention |
| Saved items, tags and private notes | Durable Automerge personal collection |
| Own contributions and participated threads | Durable Automerge text/context, including future replies retrieved through thread following |
| Liked remote post | Intended split: minimal remote reference plus Like on website; content and Like state in Automerge |
| Downloaded attachments | Device-local files; original URL/basic media metadata in post Automerge records |

The website need not archive full remote content for Likes. Protection of liked content, pending/confirmed reconciliation and any effect on saved status remain open. One request that creates a minimal remote reference if needed and Likes it is a recommendation, not a selected API contract.

Clearing/expiring temporary content must not remove saved items, notes, or durable communications. Following an account does not archive all its posts. Permanent saving preserves relevant post text and note context without introducing full webpage archiving. Automatic saved-item/snapshot mechanics remain implementation details.

## Cache and read state

RSS and ActivityPub caches must default to unlimited item count: no application-imposed count cap, not a promise of upstream history or perpetual availability. Users can set a maximum count per feed/account. Count read and unread together; evict oldest unprotected content when exceeded. Saved items and durable communications are exempt. Page size and unread count must not substitute for this retention policy.

Separate fetch metadata (such as cursors, ETags and Last-Modified when available), item identity/deduplication and user read state. Reading an item on one device must make it read elsewhere after sync; stale unread state must not undo that read. The read-wins algorithm is implementation work. A later deliberate Mark Unread action is neither excluded nor specified.

After removing an old unsaved post, retain its read tracking for a limited period in rolling tracking documents, then discard it. Saved items retain their read state. Exact limits and lifecycle mechanics are implementation details. RSS bookkeeping must not grow without bound for every item ever seen; timestamps alone are unsafe as the sole deduplication boundary. Bounded recent-ID tracking is a suggested approach, not a selected algorithm/window.

Stable identifiers and record relationships belong to Rust data-model/storage implementation. Distinguish remote post, local saved record and destination article URL, accommodating protocol differences and unreliable/missing upstream IDs. Do not merge distinct posts based on a shared URL.

## Learned changes and attachments

Apply website-reported source edits/deletions to unsaved cached posts and sync those learned changes. Update learned comment text in retained conversations. Preserve saved content after original deletion and mark the original deleted. Automerge propagates known changes; it does not discover upstream changes. The website must supply change data, including after relevant incoming-copy removal, through a contract still to be implemented.

Attachment binaries must remain device-local and must not be stored or synchronized through Automerge. Post records retain original URLs and basic media metadata. Each device fetches its own copy on demand. If the original is gone, a new fetch can fail while an already-retained local copy remains usable. Peer blob transfer, cloud attachment services, exact eviction policy and automatic-fetch timing are not selected.

ROTT must open full webpages externally and save URL/title/notes. This excludes a built-in webpage reformatter/reader or full article archive, while preserving RSS/social text and conversations.

## Document engineering

Document boundaries, archive sizes, rotation/retirement, recent-state carry-forward, offline peers, conflicts beyond read state, and history/storage reclamation are deferred to implementation. Convergence alone does not define desired conflicting-edit behavior.

Size-based rotation of disposable tracking documents with durable saved data kept separate is a proposed direction. No exact protocol, thresholds, or one-document-per-account scheme is selected. Field deletion must not be assumed to prune Automerge history or reclaim disk. Bounded bookkeeping does not change the unlimited-by-default content-count policy or authorize loss of protected data. Exact cache/database technology is not selected; SQLite is only a possible implementation.

## Optional sync and recovery

A single-device installation, including phone-only use, must work without a sync server or hosted frontend. Locally available links can be saved/tagged/annotated offline. Configured devices make independent edits and merge on reconnect; network access is needed for remote fetching and exchange.

The chosen personal setup reuses the default Automerge sync server behind the user's VPN, with network-restricted access and no separate application authentication for that private setup. New-client connection/protocol/version compatibility remains unverified and must be tested during implementation. This is not a public/shared-hosting policy or a requirement for single-device use.

Other multi-device users need self-hosted or third-party sync, or a future implemented peer-to-peer transport. The current Rust application uses WebSocket sync; P2P is not implemented. Platform lifecycle/background behavior remains application-specific, particularly on iPhone, and is separate from manual website Download.

A shared hosted service requires authentication and document authorization; reachability or possession of a document ID is not authorization. Hosted authentication, device authorization, pairing and Keyhive evaluation are deferred until such a service is considered. Keyhive is not selected; maturity/security/Swift compatibility would need assessment. VPN access controls do not provide end-to-end encryption against the sync server.

Sync is not backup. Backup/export/recovery design is deferred as the design evolves. Personal data should not be trapped, but no initial export feature, format or timing is committed. Optional sync does not guarantee indefinite temporary-content retention or recovery of deleted upstream/server copies. Existing CLI/TUI isolation and any later separate migration helper follow [architecture](architecture.md).
