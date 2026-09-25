# ROTT Reader and Communications Overview

Date: September 24, 2026

Status: Design discussion summary; documentation only, not an implementation specification.

This overview records the desktop reader and communications direction alongside the [design document](ROTT-DESIGN.md), now updated with the September 24 storage and sync decisions. Both supersede the earlier narrow durable-storage boundary: relevant post and conversation text/context belongs in durable personal communication history alongside saved links and personal state. The behavior below distinguishes established user direction from supported possibilities and questions that still need design work.

## Established direction

ROTT will be a desktop and iPhone application combining an account reader with a personal collection of saved links, tags, and notes. On first launch, the user authenticates to a SlugKit-compatible website, such as erikvancraddock.com, and browses its catalog to choose accounts to follow in ROTT.

Connect on the Mac with a website address and API key. Non-secret connection details—the website address and account name/identity—sync through Automerge; each device authenticates separately. Credentials stay in that device's platform secure storage and never sync in Automerge. No new OAuth/browser flow or platform credential-cloud sync is selected, and devices need not use the same key. Detailed authentication/identity API contracts remain under Q26; full-access keys are accepted initially.

Account identity belongs in the existing `GET /auth/check` response, with an ActivityPub actor address added where supported. The reviewed deployed schema currently returns only `data.authenticated: true`; this is a planned extension, not an implemented feature or a new `whoami` route. A future SlugKit task should specify exact fields and multiple-account/site-wide-key handling; no task is created here. One initially connected account and future switching remain the direction, with ActivityPub optional. Exact identity fields and account mapping remain implementation work; the initial key-access policy is settled below.

The user confirms current SlugKit API keys have full access and accepts this for initial ROTT. This is user-confirmed behavior, not a newly code-verified claim. Narrower scopes/permissions are deferred, not an initial blocker. Per-device secure credentials and no Automerge secrets remain unchanged; no new task or authentication implementation is created.

The user views ROTT as a communications application: communicating with themselves through private notes, contributing to public Fediverse conversations, and potentially communicating with selected people. The core requirement is to preserve their communications regardless of audience or channel. Durable personal communication history in Automerge, synchronized across devices, is an explicit decision; additional messaging integrations are not implied by that requirement.

SlugKit is a protocol-independent website/content-management API contract used by the Slug CLI; an implementation may have no ActivityPub actor. The inspected template and erikvancraddock.com use Fedify for ActivityPub, which is not a universal contract requirement. SlugKit supplies sources, contacts, and accounts. A source or contact can own multiple accounts, and sources can also link to contacts. Local SlugKit specifications reviewed during the discussion confirmed these relationships, including RSS feed accounts and ActivityPub social accounts. Those catalog models do not establish that SlugKit fetches RSS content or that catalog access is authorized separately for each user.

ROTT provides an account discovery and selection screen driven by the SlugKit catalog. Users can browse sources, contacts, and their accounts and choose which accounts to follow.

Account search is in scope here: a field at the top accepts a name or handle, with results below showing source/contact associations and their accounts. Users choose an account to follow and see the appropriate follow status. Catalog search exists in current SlugKit source, but the reviewed website code and deployed schema lack its search parameters; deployment and feature fit remain dependencies to verify (see the API gap review). Searching received or saved post content is deferred.

Initial ROTT uses one connected website supplying one catalog. Combining catalogs is outside initial scope; duplicate handling across catalogs should be revisited only if that feature is introduced, with no merge policy selected. Initially ROTT connects one account configuration, with website/API identity distinguished from any social actor identity so future multiple configurations and switching remain possible. Follow and publishing identities are scoped distinctly; multi-account UI is deferred. ROTT owns RSS subscriptions and other local selections. SlugKit's confirmed follow list is authoritative for ActivityPub: ROTT requests follows/unfollows and reconciles to server-confirmed state, including changes made elsewhere. Local intent does not override that state.

An account's presence in the catalog does not subscribe the user to it. The Following sidebar lists local selections, such as RSS subscriptions and external destinations, and accepted ActivityPub follows. ActivityPub intent alone does not qualify an account for Following. The main pane displays incoming posts when a supported integration can retrieve them. A selected account can remain useful as an external destination even when ROTT cannot retrieve its posts.

ROTT stores and syncs basic subscription account metadata in Automerge: name, handle, source type, address, avatar information, and last known follow status. This supports an offline sidebar with consistent device account lists. One thousand subscribed accounts means metadata for one thousand accounts, not a decision to create one thousand Automerge documents. Offline follow status remains a snapshot and never overrides server authority. Schema/document layout, rotation thresholds, and avatar image/blob storage remain implementation questions; avatar references and local image caching are possibilities rather than selected policies. Credentials remain per-device secrets outside Automerge, with document engineering deferred to implementation.

Pressing Download refreshes the catalog. ROTT matches stable account identities and updates names, avatar information, and details while preserving subscriptions, saved posts, and notes. Offline changes remain undiscovered until a successful refresh. Confirmed absence from the authoritative catalog marks an account unavailable while retaining saved posts, notes, and communications. A failed fetch or omission from a paginated/search subset is not confirmation of deletion. Catalog disappearance does not automatically unfollow the server-side actor or erase personal data; server follow authority remains unchanged.

Following requires an online connection, with no offline follow queue. Show failed requests to the user and reconcile with the website’s authoritative follow list on reconnect. Retry/operation identifiers are implementation details. Pending requests submitted online remain distinct from an offline queue.

Selecting an ActivityPub account requests a real follow by the configured social actor through optional protocol-neutral API actions. The website handles federation/signing; ROTT need not receive the actor's private key. The actor server's confirmed follow list remains authoritative, with pending requests separate until acceptance. Exact authentication, item/account identifiers, capability handling, API coverage, and state coordination remain open under Q26.

An ActivityPub account must not appear in Following until its follow request is accepted. Outstanding requests remain visible as Requested/Pending in a dedicated, discoverable pending-follows view or section, where users can inspect and cancel them. Rejected or failed requests also stay out of Following and show their status in that management interface. The visibility and cancellation behavior are decided; the exact layout remains open, including a separate screen, tab, or filter within the account picker.

## Behavior by account type

| Account type | Selection in ROTT | Content and navigation | Remote follow |
| --- | --- | --- | --- |
| RSS feed | Adds the account to the sidebar and subscribes in ROTT. | The Rust core fetches/parses/caches feed items and serves them to the UI under the cache policy below. | No social-network follow is implied. |
| ActivityPub | Records subscription intent and shows Requested/Pending in follow management. Adds the account to Following only after acceptance; rejected or failed requests remain in management with their status. | Users can inspect and cancel outstanding requests. Display incoming posts through a supported integration; incoming-post access still needs verification or design under Q26. | Request a follow by the configured actor through the server-mediated semantic API, whose exact contract remains open in Q26; server-confirmed acceptance is distinct from the request. |
| Bluesky / AT Protocol — design possibility | Could add the account to the sidebar as a public-post subscription. | The public author-feed API allows reads without Bluesky login; a ROTT integration is not implemented by this discussion. | Public reading does not create a follow. An actual follow requires an authenticated AT Protocol account and a separate design decision. |
| Other accounts, such as Facebook or X | Keep selected accounts in the sidebar even without content retrieval. | Offer opening the account's web page or an appropriate app where supported. | Selection does not perform or promise a remote follow. |

External navigation broadens the original draft's exclusion of platforms such as Facebook and X: they can be useful sidebar destinations without a promised feed integration.

## Optional social actions and remaining API questions

**New API additions/extensions belong in the SlugKit project.** The [API gap review](ROTT-API-GAP-REVIEW.md) separates shared schema/API work, optional website backend/federation implementation, and ROTT Rust consumption. It verifies current code/schema coverage without claiming live authenticated validation; detailed contract work proceeds per feature.

The confirmed direction updates the existing API with optional protocol-neutral actions such as “like this item.” A request identifies the account and item without exposing ActivityPub messages or signing details. The website implements the action using ActivityPub internally or another mechanism, or does not support social features. SlugKit sites without social functionality remain compatible; their mandatory content-management contract is not expanded to require ActivityPub.

Reuse the existing protocol-neutral follow endpoint where applicable. A necessarily separate `/api/rott/v1` namespace or duplicate API is not the direction. No new route/schema is chosen, including the previously suggested `POST /api/rott/v1/likes` or `actorId`/`objectId` fields. Server-mediated semantic actions supersede direct actor-key signing; the website handles federation/signing and ROTT need not receive actor private keys. This is a design decision, not authorization to implement website changes.

External-item identity is still open: Alice's remote post is distinct from a site-owned post addressed by slug, and a local following ID identifies a relationship rather than a post. Authentication details and account identity mapping under the initial full-access key policy, exact routes/schema, capability-report fields/versioning/mapping, unsupported responses, incoming-post/action coverage, remain contract work; existing Follow/Undo and Accept/Reject code needs ROTT integration and live validation, as recorded in the API gap review.

The local template review found authenticated `POST /api/v1/following` record creation and Fedify delivery in `api/routes/following.ts`, and server-side actor key generation/persistence in `keys.ts`. The deployed [OpenAPI schema](https://erikvancraddock.com/api/v1/openapi.json) lists follow/list/unfollow, catalog CRUD, post publishing/editing/deletion, engagement GET, and site comments. No documented incoming timeline or outbound like/boost/quote/arbitrary remote reply endpoints were found. These are code/schema findings, not live authenticated tests, and are not universal SlugKit capabilities.

Q26 is partially settled by this server-mediated, protocol-neutral optional-action direction and remains open for the specific contracts above. Agreed feature behavior is unchanged.

## Shared SlugKit contract and ROTT doctor

ROTT is another client of the same SlugKit API schema/authentication contract used by the CLI. The existing schema is the source of truth; ROTT calls it through its Rust core rather than running CLI subprocesses. Website implementations handle ActivityPub signing/delivery behind optional semantic actions, not through a separate ROTT federation implementation. Compatible optional additions may be needed for missing social operations; current schema coverage is not assumed complete.

The approved read-only **ROTT doctor** checks API compatibility/supported operations, credentials/account access, and configured ActivityPub actor-profile reachability where applicable. The website must explicitly report capabilities such as incoming reading, follow, like, and share. Actor URLs alone do not establish supported client actions. Unsupported features are reported clearly without assuming the entire app is unusable or making an actor mandatory for every SlugKit site. Doctor must not create test posts, follows, likes, or other social mutations.

An existing health or metadata endpoint is only a candidate for capability reporting; exact location, fields, versioning, and schema mapping remain design work. Rust type alignment and contract validation mechanics also remain implementation tasks, with no selected generator, shared TypeScript package, or new authentication scheme. Q26 retains detailed capability/error/authentication and coverage questions.

## Application architecture and platforms

This is a new application with a fresh codebase. Existing Rust ROTT is not the starting point or a required reference; no legacy code reuse assessment is required. Any later migration uses a separate helper built on ROTT facilities; no initial importer is required.

The shared Rust core owns domain/data, saving/tagging/private notes, follows and publishing/reply/boost API operations, feed fetching/parsing/caching, serving filtered post lists/content/replies, and Automerge persistence/sync. UI code owns presentation, navigation, and editors; platform glue handles browser authentication, secure credentials, and OS lifecycle as necessary. The library interface can be independent of UI. Both macOS and iPhone use native Swift/SwiftUI interfaces bridged to a compiled Rust library. Mobile is a definite product goal. UniFFI and swift-bridge are candidates; neither is selected or preferred. Rust is chosen for shared logic, not because iPhone development intrinsically requires it.

Start with **one reusable Rust core library/crate**, internally organized around accounts, downloading, saved items, publishing, storage, and sync. Independent interfaces call its domain operations; there is no upfront split into multiple domain libraries. Exact module names and signatures are not selected, and platform bridge/build glue need not share the same crate. Code generation or a shared TypeScript package is not selected.

The core stays independent of any particular bridge, and each consuming platform app supplies appropriate adapters/bindings. macOS/iPhone may share Swift bridge code while compiling the core for each target. Rust Windows/Linux callers or a hypothetical future Rust TUI can call it directly; other languages use suitable bridges, without a universal multi-language bridge requirement.

Thin Rust-side export glue may be needed alongside app-language bindings and may live separately from the single core crate. Tool choice, signatures, async operations/cancellation, events/UI updates, error translation, ownership/lifetimes, and platform services remain application implementation requirements.

A monorepo was discussed and recommended but not finalized. Standalone core package publication is possible, not required. Forgejo hosting is only under consideration; no repository migration is authorized.

Focus on Mac and iPhone first, then Windows and Linux. All four remain required; no order within either pair or release dates are selected. Windows/Linux toolkits and target-specific build/packaging remain implementation requirements. Both can use Rust GUI frameworks: Slint or Iced are candidates for sharing their UI, while GTK4/libadwaita is an option for a GNOME-specific Linux interface. None is selected; C#/WinUI is not required. A future TUI calling the core directly is hypothetical, not committed scope.

The earlier TypeScript/Electron desktop direction is superseded. Electron was considered for shared desktop UI and does not target iPhone; native-plus-Rust is a choice for this product, not a claim of categorical superiority. Sharing the core does not mean sharing all UI or identical binaries. Per-target builds/packaging, bridge/API design, UI change notifications, platform services, and iOS lifecycle/background-sync constraints need work. An npm `slug-client` shared with the CLI is no longer an assumed dependency; the Rust integration should follow a consistent SlugKit API contract.

## Reading and Read Later

The current Rust ROTT is a three-pane saved-link organizer: filters on the left, a title/URL list in the middle, and details and notes on the right. The proposed reader reuses that interaction with followed accounts on the left, a compact incoming-item list in the middle, and the selected post, readable content, and conversation in the reading pane. Defaulting to All Unread across followed accounts and filtering by clicking an account are favored UI directions, not a finalized detailed specification.

Compact entries use a title when available or a first line/snippet when there is no title. Long posts open in the reading pane. RSS may provide only a summary or link; this layout does not assume that every feed supplies the full article body.

iPhone navigation and lifecycle/background-sync behavior are deferred to iPhone-specific app design. The three-pane direction describes desktop interaction; no iPhone screen layout, navigation, background scheduling, or device-sync policy is selected. The shared Rust core must assume neither three panes nor a continuously running app. Explicit manual Download of website incoming posts remains agreed and separate from optional device sync, whose mobile background implementation is deferred. Four-platform support and Mac/iPhone-first priorities are unchanged.

Preserving the existing favorites-as-read-later workflow is a requirement: users need to mark items rapidly for a later visit. Read Later is a private save action, distinct from a federated Like/Favourite. A separate Like action is wanted in the reading pane for supported social posts, as the configured ActivityPub actor where applicable; the required client API must be verified.

Private comments and notes should be available across account types, including ActivityPub, without publishing anything. The expected primary use is annotating saved items. Private notes must be visually distinct from the public reply composer.

## Incoming history, cache limits, and read state

RSS and ActivityPub caches default to **unlimited**: no application-imposed item cap, not a guarantee of publisher history. A configurable maximum article/post count applies per feed/account, counting read and unread items together. When exceeded, the oldest cached items are removed. Saved items and durable communications are protected from this eviction. Fifty unread items and page-size limits are not the selected retention policy.

The initial ActivityPub version does not backfill historical posts. It displays only posts received after the actor's follow/subscription and does not automatically surface pre-follow cached history. ROTT can manually download post-follow items still on the website; other clients receive downloaded content through optional ROTT sync. A new device must not assume website copies already removed are available there. This is post-follow retrieval, not historical backfill. Future backfill is possible but not in initial scope. Exact follow-boundary identifiers/cursors and pagination remain engineering work.

Read state syncs with read-wins behavior: reading on one device makes the post read elsewhere after sync, and stale unread state must not undo it. The conflict algorithm is implementation work; this neither rules out a later deliberate Mark Unread nor selects its semantics. After an old unsaved post is removed, keep read tracking for a limited period in rolling documents, then discard it. Saved items retain read state. Exact limits and rotation/retirement/offline-peer mechanics remain during implementation.

Exact stable IDs and record relationships are deferred to internal data-model/Rust storage implementation. Remote post identity, local saved-record identity, and destination article URLs are distinct concepts; no scheme is selected. Different posts remain separate even if they link to the same article, with no destination-URL or cross-post canonical-link merging of saved entries. This does not require duplicates of exactly the same record on sync.

Source edits/deletions learned from the website apply to unsaved cached posts and sync across devices. Explicitly saved content survives deletion of its original, with the original marked deleted. Automerge propagates learned changes; it does not discover remote changes. Participated threads are followed for future replies and retained with context; learned comment edits update retained text without a revision-history feature.

## My Feed and conversations

A My Feed tab (provisional name) will show the user's own SlugKit-published feed with associated replies/comments and engagement, including likes on their posts. ROTT also provides a Liked posts view for posts the user has liked; its exact placement relative to My Feed is not specified. The user wants comments visible inside ROTT without requiring their display on the public SlugKit website. Omitting them from that website is a preference recorded here, not an authorized website change.

Conversation viewing also applies to other people's ActivityPub posts: users should be able to open a post, inspect available replies, and reply as the configured ActivityPub actor. Federation may expose only part of a conversation, so ROTT must not promise a complete global thread. Reading engagement in My Feed does not imply notification alerts; notifications are deferred.

## Commentary, sharing, and publishing

The agreed interaction starts with an item selected in the reading pane. The user writes commentary, then chooses **Keep Private** or **Share**. Keeping a comment private never publishes it merely because it is saved. Before posting, ROTT must show the intended publishing action for that item type.

| Action | Intended result | Boundary |
| --- | --- | --- |
| Keep Private | Retain a private comment/note with enough context to understand it later in durable Automerge history. | No public post or federated interaction; exact context capture and automatic saved-item mechanics remain undecided. |
| Share an RSS/article link or other supported URL | Create a SlugKit link post containing the source URL and commentary, akin to sharing through the existing `slug` CLI. | This is a new post; normal SlugKit link posts federate when federation is enabled. It is not article authoring. |
| Share an ActivityPub post with commentary | Create a quote post where supported. | Quotes are in scope. Verify SlugKit and target-protocol support and permissions; do not silently substitute an ordinary link post. A quote is not a reply and is not necessarily part of the original conversation. |
| Share a followed post without commentary | Redistribute the original; an ActivityPub implementation sends Announce (boost). | Plain reshare is distinct from quoting, replying, or publishing a new link post. |
| Reply to an ActivityPub post | Publish a reply in the original thread as the configured ActivityPub actor. | A distinct public composer/action from private notes and quotes. |
| Publish a standalone short update | Create a SlugKit note post. | Article authoring is excluded from ROTT. |
| Like a supported social post | The website sends a Like/Favourite that notifies the original author; the item appears in ROTT Liked posts. | Does not become a Share or automatically enter the public website feed; separate from private Read Later. |

The protocol-neutral API vocabulary is **Share** for the action and **Shares** for records. ActivityPub translates a plain reshare into `Announce` (boost); it is not link publishing or a quote/commentary post. No exact endpoint paths or deployed route renames are selected or authorized. The existing `GET posts/{slug}/boosts` reports incoming shares on the user's own post, not an outgoing list of shared posts.

Likes follow Mastodon favourite behavior. Their notification to the original author is separate from the deferred ROTT notification-alert feature. Read Later/private Save remains separate: it must not automatically publish or upload the original to the website.

For liked remote posts, the intended split is a remote-item reference plus the user's like on the website, and post content plus like state in ROTT's Automerge data for Mac/iPhone viewing. The website need not archive the full remote content. Retention protection and pending/confirmed reconciliation remain open, as do generic external-item IDs and the API contract. One call that creates the minimal reference if necessary and likes it is a recommendation, not a chosen request schema. Liking does not by itself decide saved-item status.

These are product requirements, not claims of complete backend support. Client APIs for standalone notes, link posts, Shares, quotes, replies, likes, own-feed engagement, and conversation retrieval need verification or design. This scope does not extend publishing to Bluesky.

Users also want to edit and delete their own published posts from ROTT. Website content management uses the SlugKit contract; the website implements social updates and deletions through semantic actions, with exact API coverage still part of Q26. The required APIs need verification. Editing applies to the user's own posts, not other authors' originals. Removing a private saved item and deleting an owned published post are separate actions. Federated deletion cannot guarantee erasure of every remote copy.

## Bluesky possibility and hosting boundaries

Bluesky is a supported design possibility, not a completed feature or a commitment to implement remote following. Its [author-feed API](https://docs.bsky.app/docs/api/app-bsky-feed-get-author-feed) supports public reads without login, so ROTT could retrieve selected accounts' public posts independently of a Bluesky account login. Authenticating to a SlugKit site does not itself authenticate an AT Protocol account.

A self-hosted Personal Data Server (PDS) can host an AT Protocol account, but it does not alone assemble incoming feeds. An AppView serves assembled feeds; even a user with a self-hosted PDS commonly uses Bluesky's AppView. Relays aggregate repository events for downstream consumers. These are distinct roles, described in [The AT Stack](https://atproto.com/guides/the-at-stack) and the [self-hosting guide](https://atproto.com/guides/self-hosting).

Self-hosting is optional for reading public posts. No decision has been made to deploy a PDS, AppView, relay, or other server, or to choose a Bluesky authentication method.

## Durable communications and saved items

RSS subscriptions, local selections, subscription/request intent, read state, and explicitly saved links with their tags and notes remain durable personal state in Automerge. ActivityPub intent never overrides SlugKit's confirmed follow state. The expanded requirement also stores private notes, the user's own published contributions, and retained conversations involving the user in Automerge, synchronized across devices. This includes relevant post/conversation text and enough retained context to understand the communication later. Retaining communications is decided; the exact capture boundary and schema are not.

SlugKit remains the canonical website publication/content API boundary. The website handles optional social delivery/signing and authoritative follow state. Participating in a conversation makes ROTT follow that thread, retrieve future replies, and retain the conversation and relevant context in Automerge. This extends beyond keeping only the user’s reply and its parent. Test the effectiveness of thread following during implementation; complete global discovery is not guaranteed. Learned comment edits update retained text; a product revision-history feature is not required, and this does not require erasing Automerge’s underlying history. Saved posts survive deletion of their originals and mark the originals deleted. Capture mechanics and automatic saved-item handling remain implementation work.

Website incoming posts are now persisted in Automerge for optional sync, but remain temporary content unless protected as saved items or communications. Merely following an account does not archive all its posts, and discovering a post or link does not automatically save it to the personal collection. Clearing cached posts must not erase durable saved items, annotations, or retained communication history.

Permanent saving preserves post text and note context when originals disappear. Automatic snapshot and record mechanics remain implementation details. Webpages open in the default system browser or appropriate application. ROTT saves URL, title, and notes; it does not provide a built-in full webpage reader, reformatter, or full article archive. RSS/social post text and retained conversations remain displayable and retainable in ROTT.

Attachments are downloaded device-local files. Their binaries are not stored in Automerge or synchronized through it. Post records in Automerge retain original URLs and basic media metadata; each client fetches its own copy on demand. If the original disappears, a new fetch may fail while an existing retained local copy remains usable. No peer blob transfer, cloud attachment service, eviction policy, or exact automatic-fetch timing is selected.

The existing Rust CLI/TUI must remain isolated and usable while the new applications are developed. The shared Rust core and Apple UI direction do not settle migration, Windows/Linux toolkits, cache technology, or detailed retrieval architecture.

## Manual Download and website-copy removal

The website exposes one combined incoming stream across followed accounts for the connected account/actor. Manual Download retrieves combined batches; ROTT filters locally by the selected followed account. Separate per-followed-account streams are not required initially, and identities are not mixed. This is post-follow-only content with no historical backfill, not an RSS proxy; RSS fetching remains in the core. Exact endpoint, batch/cursor, and removal details remain implementation work.

Opening ROTT displays local items. Downloading website incoming posts requires an explicit **Download** action; it does not happen automatically on open or in the background. ROTT persists the downloaded content locally in Automerge for other clients to receive through optional ROTT sync.

The default then sends a separate request to delete exactly those downloaded/persisted incoming website copies. Fetching alone does not delete them. Successful local persistence is sufficient; no second-copy confirmation is required. A configurable option keeps website copies indefinitely. This is not deletion of the original author's post or the user's own published content.

Temporary incoming content still follows the default-unlimited/configurable-count cache policy; saved items and durable communications remain protected. Automerge sync is intended for downloaded incoming content, not a guarantee of indefinite retention or a backup. Without optional sync, other devices cannot assume they receive content another device has removed from the website. Private Save/notes still do not upload or publish originals, and the website incoming store remains separate from an optional sync server.

Rare overlap/duplicates from concurrent manual downloads are acceptable. Basic stable-ID deduplication, batch pagination/cursors, follow-boundary identification, exact removal targeting, retries/idempotence, and authentication remain implementation requirements without an exactly-once guarantee. No endpoints or batch sizes are chosen. Persistence must succeed before removal is requested. Recovery, source-change delivery mechanics, and actual server cost effects still need attention.

## Bounded read-state tracking and document lifecycle

RSS bookkeeping should avoid unbounded per-item tracking; a timestamp alone is unsafe. Bounded recent-ID tracking is suggested, with no final ID/date algorithm or window selected. Rotating disposable Automerge tracking documents by size while keeping durable saved content separate is also a proposed direction, not a completed implementation design.

Implementation work covers thresholds, document boundaries, rotation/retirement protocol, carrying needed recent state, offline peers, and history/storage reclamation. Automerge field deletion must not be assumed to prune history or reclaim storage. This bookkeeping proposal does not change the unlimited-by-default per-feed content count or configurable maximum of read and unread items together.

## Local data and optional sync

Automerge is selected as a good fit for ROTT's local personal data. A single-device installation needs no sync server; independent/offline edits across configured devices merge on reconnect. ROTT needs no hosted web frontend. Desktop and iPhone clients maintain local data through the Rust core, including a phone-only installation without sync infrastructure. SlugKit website accounts/catalog/content APIs and the social account integration remain separate from local data sync; Fediverse delivery is not a universal SlugKit capability.

Current Rust sync uses a WebSocket server; peer-to-peer sync is not implemented. The chosen plan for this user reuses the default Automerge sync server behind the user’s VPN, with network-restricted access and no separate app-level authentication for that private setup. New-client compatibility is not verified; connection/protocol/version checks remain routine implementation validation, and document organization remains implementation work during implementation. This is not mandatory for every user. Other multi-device users need self-hosted or third-party hosted sync, or future implemented P2P. A shared hosted service requires authentication and document authorization. VPN/access controls do not provide end-to-end encryption against the server.

Document boundaries, sizes, rotation/retirement, offline-peer handling, and conflict engineering are deferred to implementation. Bounded temporary tracking and protection of saved data/communications remain requirements; no exact lifecycle protocol is selected. Backup, export, and recovery design is deferred as the design evolves. Personal data should not be trapped, but no initial export feature, format, or timing is committed. Hosted sync authentication, device authorization, pairing, and any Keyhive evaluation are deferred until a hosted service is considered; the personal VPN sync plan is unchanged. Sync is not backup.

## Possible communication channels

Private/direct ActivityPub messaging was explored as a possible channel, not committed integration scope. The discussion noted that Mastodon-style addressed/private mentions are not end-to-end encrypted and can be accessible to server operators; SlugKit support remains unverified. This is separate from ROTT's private local notes and does not establish a secure messaging feature.

Contacts can have multiple accounts and channel types, so the durable communications concept should not assume every communication is ActivityPub. Email, SMS, and XMPP were hypothetical examples supporting that architectural rationale, not additions to the feature list or current implementation scope.

## Deferred and uncommitted features

- Deferred: notifications, content warnings, polls, custom account lists/timelines, and full-text search of received or saved posts.
- Audience controls were provisionally grouped into later work without elaboration; a default publishing audience still needs explicit design.
- Publishing media uploads and alt text remain uncommitted; downloading attachments follows the decided device-local policy. Mute, block, and moderation filtering behavior likewise has no separate decision.

Quote posts are in scope, rather than part of the deferred list. Article authoring is explicitly excluded.

## Confirmed retention and deferred engineering

Participating in a conversation makes ROTT follow that thread, retrieve future replies, and retain the conversation and relevant context in Automerge. This extends beyond keeping only the user’s reply and its parent. Test the effectiveness of thread following during implementation; complete global discovery is not guaranteed. Learned comment edits update retained text; a product revision-history feature is not required, and this does not require erasing Automerge’s underlying history. Saved posts survive deletion of their originals and mark the originals deleted. Capture mechanics and automatic saved-item handling remain implementation work.

Attachments are downloaded device-local files. Their binaries are not stored in Automerge or synchronized through it. Post records in Automerge retain original URLs and basic media metadata; each client fetches its own copy on demand. If the original disappears, a new fetch may fail while an existing retained local copy remains usable. No peer blob transfer, cloud attachment service, eviction policy, or exact automatic-fetch timing is selected.

Webpages open in the default system browser or appropriate application. ROTT saves URL, title, and notes; it does not provide a built-in full webpage reader, reformatter, or full article archive. RSS/social post text and retained conversations remain displayable and retainable in ROTT.

Document boundaries, sizes, rotation/retirement, offline-peer handling, and conflict engineering are deferred to implementation. Bounded temporary tracking and protection of saved data/communications remain requirements; no exact lifecycle protocol is selected. Backup, export, and recovery design is deferred as the design evolves. Personal data should not be trapped, but no initial export feature, format, or timing is committed. Hosted sync authentication, device authorization, pairing, and any Keyhive evaluation are deferred until a hosted service is considered; the personal VPN sync plan is unchanged.

If migration is pursued later, use a separate helper script or application built on ROTT facilities. A built-in importer is neither required for the initial application nor authorized now. The existing CLI/TUI remains isolated and usable.

## Relevant open questions

- **Q26**: As each feature is implemented, what SlugKit contract additions are needed for external-item identity, account identity, optional capability reporting, incoming batches/removal and source changes, remote thread following/retrieval, and outgoing Like/Share/quote/reply actions, including pending/confirmed reconciliation and unsupported-action responses? Use the API gap review as the baseline; existing follow lifecycle support needs integration validation, not an invented replacement.

- Implement and validate the SlugKit additions, website behavior, and Rust client integration identified in the [API gap review](ROTT-API-GAP-REVIEW.md), feature by feature; existing operations are not all missing APIs.
- Which layout should expose follow management: a separate screen, tab, or filter in the account picker? Requested/Pending visibility, cancellation, and rejected/failed status display are established requirements.
- What quote support and permissions do SlugKit and target protocols expose, and how should unavailable quoting be shown? What default audience should publishing use?
- What retention protection applies to liked posts in Automerge, independently of explicit saves?
- Should an initial Bluesky integration only read public author feeds, or also support authenticated remote follows? If the latter, which account and authentication flow should it use?
- How should the interface communicate accounts with readable feeds versus external navigation, and which app-opening links can each platform support?

Remaining interface and API details are open; storage engineering, migration helpers, export/recovery, and hosted authorization are deferred as described above. Full webpage archiving is excluded. Incoming cache limits and the initial no-backfill policy are decided above.
