# Reader and communications

[Specification index](README.md)

## Reading and navigation

Opening ROTT must show locally available items without starting a website incoming-post download. Users must be able to browse cached content and the saved collection offline.

The proposed desktop layout has accounts on the left, compact incoming items in the middle, and selected post text/content/conversations in the reading pane. All Unread across followed accounts and filtering by account are favored directions, not finalized layout contracts. Compact entries use a title or first-line/snippet; long posts open in the reading pane. RSS may supply only a summary or link. iPhone navigation is deferred and must not be constrained to three panes.

Webpages must open in the default system browser or appropriate application. ROTT retains URL, title and notes; it does not implement a full webpage reader, reformatter or full article archive. RSS/social post text and conversations remain displayable and retainable inside ROTT.

Read Later must preserve the rapid private save-for-later workflow. It is distinct from social Like. Private notes/comments must be available across account types, primarily for saved-item annotation, and visually distinct from the public reply composer. Private saving/annotation must not automatically publish or upload the original to the website.

## Manual Download

The required website incoming interface is one combined stream across the connected actor's followed accounts. ROTT downloads batches and filters locally by selected account. Separate per-followed-account streams are not initially required; identities must not be mixed. The website interface is not an RSS proxy; RSS fetching belongs in the core.

1. The user explicitly invokes Download, which also refreshes the catalog.
2. ROTT retrieves eligible incoming batches and persists their content locally in Automerge.
3. After successful local persistence, ROTT by default sends a separate request deleting exactly those persisted incoming website copies.
4. Optional ROTT sync distributes downloaded content to other clients, subject to temporary-content retention.

Fetching alone must not delete copies. Website incoming downloads must not occur automatically on open or in the background. Local persistence is sufficient before removal; confirmation of a second durable copy is not required. Users must be able to configure indefinite retention of website copies.

Removal concerns incoming website copies, not the author's original or the user's owned published content. Receipt/download does not mark an item saved, Like it or publish it. Without optional sync, another device cannot assume it receives copies removed by a downloading client. The website incoming store is separate from the Automerge sync service.

The initial ActivityPub reader must display only post-follow incoming content and must not backfill pre-follow history. Post-follow items still queued on the website are eligible for manual download. A new device must not assume removed website copies remain available. Future historical backfill is outside initial scope; direct public polling must not bypass these boundaries.

Basic stable-ID deduplication is required. Rare overlap/duplicates from concurrent manual downloads are acceptable; exactly-once delivery is not required. Batch sizes, cursors/follow boundaries, partial-failure recovery, exact removal targeting, authentication, receipt/retry/idempotence behavior and uncertain-response handling remain implementation work. Do not request removal when persistence fails. Server storage/cost effects must be measured rather than inferred from SQL deletion.

## Link discovery and saving

Extract candidate external links while retaining source actor and post identity. Compare candidates with the local review queue and collection. Users review candidates, optionally edit metadata/add tags and notes, and explicitly save selected links. Discovery alone must not create durable saved entries.

Different posts and their saved entries must remain separate even when they link to the same article; no destination-URL or cross-post canonical-link merge is selected. This does not require duplicating exactly the same record on sync.

Implementation must define eligible object/activity types and treatment of received boosts/replies/quotes/pinned posts, external-link classification versus actor/hashtag/mention/attachment/navigation links, URL normalization, rate limits and pagination. These extraction details do not reopen the decided outgoing action scope or authorize canonical merging.

## Publishing actions

Users can select an item, write commentary, and choose Keep Private or Share. ROTT must show the intended action before posting. The following actions are distinct:

| Action | Required result |
| --- | --- |
| Keep Private | Retain private text and enough context to understand it later; no public/federated action |
| Share RSS/article or other supported URL with commentary | Publish a SlugKit link post; normal website federation applies when enabled |
| Share ActivityPub post with commentary | Publish a quote where supported and permitted; never silently substitute an ordinary link post |
| Plain Share | Reshare the original without commentary; optional ActivityPub implementation sends Announce |
| Reply | Publish into the original conversation as the configured social actor |
| Standalone short update | Publish a SlugKit note; article authoring is excluded |
| Like | Send the author-facing interaction through the website and show the item in Liked posts; no automatic public-feed post or Share |
| Edit/delete owned publication | Change/remove the user's own published content, separate from deleting a private saved entry |

Likes use Mastodon-favourite behavior, including notification to the original author. This does not add notification alerts to ROTT, which remain deferred. Whether Like also changes saved status or protects content from eviction is still open. The intended website/Automerge storage split is in [data and sync](data-and-sync.md).

Quotes are in scope. Verify quote support and permissions; unsupported behavior must be visible. Default publishing audience remains unresolved, with audience controls deferred. Publishing uploads/alt text are uncommitted. Editing does not apply to another author's original; federated deletion cannot guarantee erasure of all remote copies. These product requirements do not claim that missing backend actions are already implemented.

## Own feed and conversation retention

ROTT must display the user's own published feed with available replies/comments and engagement, including likes. “My Feed” is a provisional name. A separate Liked posts view is required; placement relative to My Feed is open. Comments must be visible in ROTT without requiring public website display. Omitting them from the public site is a recorded preference, not authorization to modify that site.

Users must be able to view available conversations on other people's posts and reply as the configured actor where supported. Participation must make ROTT follow the thread, retrieve future replies, and retain the conversation and relevant context in Automerge. Keeping only the user's contribution and parent is insufficient.

Learned comment edits must update retained text. No product revision-history feature is required, and underlying Automerge history need not be erased. Saved posts must survive deletion of their originals and mark those originals deleted. Test thread-following effectiveness during implementation; complete global reply discovery is not guaranteed. Retrieval timing and capture/record mechanics remain implementation details within the explicit Download policy; no automatic background website download is selected.

Durable communications include private notes, own published contributions and participated conversations regardless of audience/channel. Additional private/direct ActivityPub messaging, email, SMS or XMPP integration is not committed. Addressed/private ActivityPub mentions are not an end-to-end-encrypted messaging guarantee; local private notes remain a separate feature.
