# ROTT: Reader, Saved Collection, and Personal Communications

Date: September 24, 2026

Status: Working design draft. No redesign implementation has started.

This repository document incorporates the discussion draft from `~/Vault/01-Projects/rott/ROTT-DESIGN.md` and the subsequent ActivityPub client brainstorming for task `task-d922c229`. The [Fediverse research conclusion](ROTT-FEDIVERSE-RESEARCH.md) compares retrieval approaches and summarizes the recommendation, limitations, and follow-ups for that original task.

The September 24 discussion expands durable storage to personal communication history and selects Automerge for local personal data with optional sync. The [reader overview](ROTT-READER-OVERVIEW.md) records the accompanying reader and publishing decisions in more detail; those decisions remain in effect.

## Purpose

The new ROTT will be a desktop and iPhone application combining a reader for multiple kinds of feeds and accounts with a personal collection of saved items and durable communications. Users discover accounts through SlugKit-compatible websites, choose subscriptions, browse incoming items, and save selected items with tags and notes. Their own contributions and participated conversations must remain understandable later, regardless of audience or channel. Additional messaging channels remain exploratory.

The application will retain the ROTT name. The chosen architecture is a shared Rust application core with native Swift/SwiftUI interfaces for both macOS and iPhone. Mobile is a definite product goal. Windows and Linux support are also required. Focus on Mac and iPhone first, then Windows and Linux; no order within either pair or release dates are selected. ROTT needs no hosted web frontend; choosing Automerge does not require a sync server for a single device.

## Application architecture and platforms

This is a new application with a fresh codebase. Existing Rust ROTT is neither the starting point nor a required reference, and assessing legacy code reuse is not a prerequisite. Any later import uses a separate helper; no initial importer is required.

The shared Rust core owns domain/data operations, saving, tagging, private notes, follows, publishing/reply/boost API operations, feed fetching/parsing/caching, serving filtered post lists/content/replies, and Automerge persistence/sync. UI code owns presentation, navigation, and editors. Platform glue owns browser authentication, secure credentials, and OS lifecycle integration as necessary. The core library interface can be independent of any UI. Apple interfaces bridge to a compiled Rust library. UniFFI and swift-bridge are candidates; neither is selected or preferred. Rust is chosen to share application logic; it is not intrinsically required to build an iPhone app.

Start with **one reusable Rust core library/crate**, organized internally around accounts, downloading, saved items, publishing, storage, and sync. Interfaces call domain operations through this library; there is no upfront split into multiple domain libraries. These are responsibility areas, not selected module names or function signatures. Platform bridge/build glue may live separately and need not be forced into the core crate. UI interfaces remain independent.

The reusable Rust core remains independent of any particular bridge. Each consuming platform application supplies the adapter/bindings appropriate to it. macOS and iPhone applications may share Swift bridge code while compiling the core for each target. Rust Windows/Linux callers, or a hypothetical future Rust TUI, can call the core directly; other languages use a suitable bridge. There is no universal multi-language bridge requirement.

A bridge may need a thin Rust-side export adapter as well as app-language bindings. This glue may live separately without changing the one-core-crate decision. Bridge/tool selection and exact signatures remain implementation work, including asynchronous operations and cancellation, events/UI updates, error translation, ownership/lifetimes, and platform services. These responsibilities belong to each application's integration; they do not couple the core to a selected bridge.

A monorepo was recommended and discussed, but is not finalized. Publishing a standalone core package is possible, not a required feature. Forgejo hosting is under consideration only; no repository migration is authorized.

| Target | UI direction | Status |
| --- | --- | --- |
| macOS | Native Swift/SwiftUI over the Rust core | Chosen direction |
| iPhone | Native Swift/SwiftUI over the Rust core | Required product target; chosen direction |
| Windows and Linux | Can use Rust GUI frameworks over the same core | Required targets after Mac/iPhone; toolkit deferred to implementation. Slint or Iced are candidates for a shared Windows/Linux UI. |
| GNOME-specific Linux UI | GTK4/libadwaita | Candidate alternative, not selected |
| Future TUI | Could call the Rust core directly | Hypothetical, not a feature commitment |

Mac and iPhone are the first focus, followed by Windows and Linux. Support for all four platforms remains required, with no ordering within either pair and no release dates chosen. Windows/Linux toolkit selection and target-specific build/packaging details remain implementation requirements; no candidate toolkit is selected.

C#/WinUI is not required for Windows support. Sharing the core does not mean sharing every UI or distributing identical binaries: each target needs its own build and packaging. The bridge/API, change notifications into the UI, and platform services such as credential storage and lifecycle handling need design. iOS lifecycle and background-execution limits require explicit sync behavior; desktop-style continuous background sync must not be assumed. The proposed desktop three-pane interaction must be adapted to iPhone navigation.

iPhone navigation and lifecycle/background-sync behavior are deferred to iPhone-specific application design. No iPhone screen layout, navigation, background scheduling, or device-sync policy is selected now. The shared Rust core must not assume a desktop three-pane interface or a continuously running application. The agreed explicit manual Download action for website incoming posts remains unchanged and is distinct from optional device-to-device data sync. Mobile lifecycle and background behavior for that sync remain deferred, without selecting a new implementation. This does not change four-platform support or the Mac/iPhone-first priority.

The earlier TypeScript/Electron desktop direction is a historical alternative superseded by this decision. Electron was considered for sharing desktop UI and does not target iPhone. Native interfaces plus Rust are not categorically better than Electron; this choice shares logic while supporting the intended native Apple interfaces.

## Working decisions

- ROTT connects to a SlugKit website/content API and separately needs access to a configured ActivityPub account for social features. A SlugKit implementation need not host any ActivityPub actor.
- Initially there is one connected account configuration. Website/API identity and social actor identity must be distinguished explicitly; future multiple configurations and switching remain possible, with multi-account UI deferred.
- ActivityPub follows and publishing operations are scoped to the configured actor. SlugKit's confirmed follow list is authoritative; ROTT submits follow/unfollow requests and reconciles to server-confirmed state.
- ROTT should connect to the Slug-compatible website in a manner similar to the existing `slug` CLI.
- Social actions use the existing API through optional protocol-neutral operations. The website handles federation/signing; ROTT need not receive actor private keys. Exact authentication, identifiers, capabilities, and API coverage remain open under Q26.
- Slug websites provide account catalogs. ROTT owns RSS subscriptions and other local selections; catalog presence alone never creates a subscription.
- Website incoming posts are downloaded manually into local Automerge data and shared through optional ROTT sync. They remain temporary content subject to cache policy unless protected as saved items or communications. Opening ROTT shows local items and does not trigger website downloads.
- Automerge is selected for local personal data: subscription intent, read state, saved items, tags, notes, and durable own/participated communication history with enough retained text and context. Multi-device sync is optional.
- A discovered link is not automatically saved. The user reviews discovered links before adding them to the durable collection.
- This is currently a research and design effort. It does not authorize implementation or following accounts yet.

## Existing projects and context

### Current ROTT

[evcraddock/rott](https://github.com/evcraddock/rott), "Record of Tagged Topics," is a Rust application used daily. Its core models and storage code already support saved links, metadata, tags, and attached notes. It provides a CLI and terminal interface and uses Automerge for local storage, with a sync server for exchanging changes between devices.

The essential behavior to preserve is local-first ownership of the saved collection: links can be saved and organized offline, remain available on the machine, and sync to other machines after connectivity returns.

### Slugkit and Slug-compatible websites

SlugKit is a protocol-independent website/content-management API contract used by the Slug CLI. The inspected TypeScript/Node.js template and erikvancraddock.com implement ActivityPub using Fedify, but arbitrary implementations may have no ActivityPub actor. Its source, contact, and account models provide context for discovering accounts. Accounts can be associated with sources or contacts and include information such as URL, kind, and protocol.

The existing website at [erikvancraddock.com](https://erikvancraddock.com) is a concrete catalog provider. During the original discussion, the command below passed all reported checks:

```sh
slug --site erikvancraddock.com doctor
```

The checks covered configuration, API reachability, API metadata, package metadata, OpenAPI, and authentication. The site reported API version 1.0.0 and OpenAPI 3.1.0. Its [API schema](https://erikvancraddock.com/api/v1/openapi.json) includes sources, contacts, and accounts. This verifies the CLI's compatibility checks; it is not an exhaustive test of every API operation.

The current `slug` CLI also exposes commands for listing, following, and unfollowing ActivityPub accounts. Its login flow opens the Slug site's `/cli/auth` page, asks the user to paste the generated API key, verifies the key through `/auth/check`, stores the API base URL and key, and sends the key as a bearer token on later requests.

## Intended user experience

1. Configure ROTT with a SlugKit-compatible website and authenticate to its catalog/content API.
2. Configure ActivityPub account access for social features where available; its detailed authentication/identity mapping and API contract remain open under Q26; initial full-access keys are accepted.
3. Browse the website's catalog of sources, contacts, and accounts.
4. Choose accounts to subscribe to locally; ActivityPub selection additionally requests a follow by the configured actor.
5. Browse locally stored items on open; use an explicit Download action to retrieve website incoming posts and persist them in Automerge. By default, ROTT then requests deletion of exactly those persisted website copies; keeping them indefinitely is configurable.
6. Track which items have been read.
7. Extract external links from incoming posts and place them in a review workflow.
8. Save selected links for reading later or another purpose, assigning tags and optionally notes.
9. Use the saved collection and retained communications offline; optionally sync durable personal state between configured devices when connected.

RSS and ActivityPub accounts are intended inputs. Bluesky public reading is a possible additional input, not a publishing commitment. Other account types, such as Facebook or X, can remain sidebar destinations for opening their page or supported app without promising content retrieval or remote following.

The account picker supports browsing sources, contacts, and their accounts, with name/handle search that needs deployment and feature validation: search exists in current SlugKit source but is absent from the reviewed website code and deployed schema. ActivityPub accounts enter Following only after acceptance. Requested/Pending follows remain discoverable and cancellable in follow management; failed/rejected requests stay out of Following and show status there.

The proposed three-pane reader uses accounts on the left, compact incoming items in the middle, and selected content/conversations on the right. All Unread and account filtering are favored directions. Read Later preserves the private favorites workflow and remains separate from social Like. My Feed shows the user's SlugKit posts and available replies/engagement; conversations on other people's posts are also readable and replyable where supported, without promising complete global threads.

From an item, users write commentary and choose Keep Private or Share. Sharing an RSS/article URL creates a SlugKit link post; sharing an ActivityPub post with commentary creates a quote where supported and permitted, with no silent substitution of an ordinary link. A plain Share (reshare without commentary) and Reply in the original thread remain separate actions. Standalone notes, likes, and editing/deleting owned posts are in scope; article authoring is excluded. The intended publish action must be visible before posting. Private saving never publishes. Website publication uses the SlugKit content contract; social actions use the server-mediated semantic API direction below, with the exact contract and coverage still open under Q26. The overview retains the detailed scope, deferred features, and unresolved audience settings.

## Shares, likes, and private saving

The SlugKit/ROTT-facing API vocabulary uses **Share** for the action and **Shares** for its records. An ActivityPub implementation translates a plain reshare into `Announce` (a boost). Plain resharing remains distinct from publishing a new link post or a quote with commentary. This names the semantic action, not an endpoint: no exact paths, implementation, or deployed route rename are authorized.

The existing `GET posts/{slug}/boosts` describes incoming shares on the user's own site post. It is not a list of posts the user has shared outward. Existing route terminology must not be mistaken for the new outgoing-action contract.

Likes behave like Mastodon favourites: the website sends the interaction that notifies the original author, and ROTT shows the item in a **Liked posts** view. A like does not automatically enter the public website feed or become a Share. This author-facing interaction does not reverse the deferral of ROTT notification alerts. Read Later remains a separate private save. Saving or annotating privately must not automatically publish or upload the original to the website.

The intended storage split for a liked remote post is a remote-item reference plus the user's like on the website, while ROTT keeps the post content and like state in Automerge for viewing across Mac and iPhone. The website need not retain the full remote content for this purpose. Retention protection for liked content, pending/confirmed reconciliation, exact generic external-item identifiers, and the request contract remain open. Whether liking also changes saved-item status is not decided by the existence of Liked posts.

A single API call that creates the minimal remote reference if needed and likes it was recommended. It is not a selected exact contract. The server-mediated, optional, protocol-neutral API direction remains in effect.

## Optional social actions through the existing API

The confirmed direction is to update the existing API with optional, protocol-neutral actions such as “like this item.” Requests identify the account and item without carrying ActivityPub wire messages or signing details. The website implements the action through ActivityPub internally, another mechanism, or does not support social features. SlugKit implementations without social functionality remain compatible.

Website content management and social-account access remain distinct concerns, but this does not require a separate namespace or duplicate API. Reuse the existing protocol-neutral follow endpoint where applicable. The earlier suggestion of a necessarily separate `/api/rott/v1` API is superseded. No new endpoint paths or request schema have been selected: `POST /api/rott/v1/likes` and field names such as `actorId`/`objectId` are not approved contracts.

This selects server-mediated semantic actions: the website handles federation and signing, and ROTT need not receive the actor's private signing key for these actions. The earlier direct actor-key proposal is no longer the selected direction. An API credential and an actor signing key have separate roles; having either does not create missing incoming-post or interaction APIs.

All ActivityPub functionality on the example template and the user's websites remains optional, outside SlugKit's mandatory content-management contract. Existing extensions may be changed to support these actions without requiring every SlugKit implementation to provide social features. This design decision does not authorize implementation of website changes now.

External-item identity remains unresolved. An action on Alice's remote post needs to identify that item separately from a site-owned post addressed by its slug. A local following ID identifies a relationship record, not a post. Detailed authentication and account identity mapping, external-item identifiers, endpoint paths, the exact capability-report schema/versioning/mapping, unsupported-action responses, and inbox/timeline/action API coverage remain to be designed.

The server-confirmed ActivityPub follow list remains authoritative. The server-side implementation must coordinate outbound Follow/Undo with inbound Accept/Reject and its follow records, and expose enough state for ROTT to reconcile. Pending requests remain separate until accepted; current source includes Follow/Undo delivery and Accept/Reject processing; ROTT integration and live validation remain implementation work (see the API gap review).

Evidence reviewed during the discussion:

- In the local SlugKit template, `api/routes/following.ts` registers authenticated `POST /following` under `/api/v1`, creates a follow record, and invokes Fedify delivery. `keys.ts` generates and persists actor key pairs on the server.
- The deployed [OpenAPI schema](https://erikvancraddock.com/api/v1/openapi.json) lists following POST/GET/unfollow, catalog CRUD, post publishing/editing/deletion, engagement GET, and site comments.
- No documented incoming timeline or outbound like/boost/quote/arbitrary remote reply endpoints were found in that review. Site comments do not establish arbitrary remote ActivityPub reply support. Documentation/code presence is not a live authenticated test.

Q26 is partially settled by the server-mediated, optional semantic-action decision and the Share/Shares vocabulary and like behavior above. It now tracks external-item identity, the exact API/authentication/account/capability contract, remaining reading/action coverage, and reconciliation/state coordination. Agreed reading, publishing, retention, and follow-state behavior is unchanged.

## Authentication and connection

On the Mac, connect ROTT using the website address and an API key. Non-secret connection details—the website address and account name/identity—sync through Automerge. Each device authenticates separately and keeps its credentials in its platform secure storage; credentials do not sync in Automerge. This does not require reusing the same API key across devices or prescribe platform credential-cloud synchronization. The initial full-access key policy and identity-discovery endpoint are decided below; detailed API/identity contracts remain implementation work under Q26.

Extend the existing `GET /auth/check` response to include account identity, plus an ActivityPub actor address where the website implementation supports ActivityPub. Do not introduce a separate `whoami` route. The deployed schema reviewed in the discussion currently returns only `data.authenticated: true`; the identity extension is planned, not implemented.

A future SlugKit task should define the exact response fields and handling of multiple accounts or site-wide keys, then implement the compatible auth-check extension. No such task has been created by this documentation work. Initially ROTT still connects one account, with explicit identity supporting future switching; ActivityPub remains optional. The initial full-access policy is described below; exact identity fields and account mapping remain implementation work.

The user confirms that the current SlugKit API key has full access and explicitly accepts that for initial ROTT. This records user-confirmed behavior, not a new code-verification finding. Narrower scopes and permission refinement are deferred to later work, not initial blockers. This changes neither per-device secure credential storage nor the prohibition on credentials in Automerge. No new authentication implementation or follow-up task is created here.

The initial connection model should remain compatible with the existing `slug` CLI unless research identifies a reason to change it. Today that means browser-assisted creation of an API key followed by bearer-token API access.

A future OAuth 2.1 Authorization Code flow with PKCE may provide a stronger application-oriented login experience, scoped access, token rotation, and revocation. That is a suggestion rather than a current decision. Any authentication redesign should define a consistent Slug API contract usable by ROTT and the CLI, with platform-specific credential and browser interactions. This does not require both clients to consume the same language-specific library.

Secrets must remain device-local and should be stored in the operating system credential store. Credentials must not be stored in Automerge or synchronized as ordinary application state.

## Shared contract and ROTT doctor

ROTT is another SlugKit API client, using the same schema and authentication contract as the Slug CLI. The existing API schema is the source of truth. ROTT calls it through the Rust core, not by shelling out to the CLI. The website implements ActivityPub signing and delivery behind semantic API calls; ROTT does not implement a separate federation layer. SlugKit remains protocol-neutral and ActivityPub remains optional for implementations generally.

Missing social operations may require compatible optional contract additions. The current schema is not assumed to cover all ROTT features. Keeping Rust types aligned with the schema and validating request/response and authentication compatibility are implementation requirements; no code generator, shared TypeScript package, or new authentication scheme is selected.

ROTT will provide a **doctor** diagnostic. The website must explicitly report its supported capabilities, such as incoming reading, follow, like, and share. Doctor checks:

- API compatibility and supported operations against the shared contract and the site's capability report;
- credentials and access to the configured account;
- reachability of the configured ActivityPub actor profile where applicable.

An actor URL alone does not prove support for any client action. Unsupported capabilities must be reported clearly; their absence does not by itself make the whole app unusable or require every SlugKit site to have an actor. Doctor is read-only: it must not send test posts, follows, likes, or other social mutations.

An existing health or metadata endpoint is a candidate location for capability reporting, not a selected endpoint. Exact location, fields, versioning, schema-to-capability mapping, and unsupported/error responses remain implementation design under Q26.

## Slug integration and client boundaries

The earlier TypeScript investigation did not identify a supported reusable npm package for the complete Slug client connection flow.

Packages examined in that investigation included:

- `@evcraddock/slug-cli`, which implements connection and API-key authentication internally but exposes only the `slug` executable;
- `@evcraddock/slug-core`, which provides shared types, constants, and helpers rather than an HTTP client;
- `@evcraddock/slug-auth`, `@evcraddock/slug-api`, and `@evcraddock/slug-federation`, which are primarily server-side building blocks.

Extracting an npm package such as `@evcraddock/slug-client` for both the CLI and ROTT was an earlier proposal tied to the TypeScript direction. It is not a prerequisite of the chosen Rust architecture. ROTT's platform-independent SlugKit integration belongs in the Rust core; it starts as one library/crate with internal modules. Exact signatures/module names and any generated bindings or types remain undecided. The CLI may evolve separately against the same API contract.

The Rust integration should cover these responsibilities where platform independent, using platform adapters where needed. Catalog/content operations use the SlugKit contract; optional social operations extend the existing API with protocol-neutral semantics. Their detailed account, item, capability, and reading contracts remain open under Q26:

- Slug site discovery and API URL normalization;
- API metadata and compatibility checks;
- authentication and credential abstractions;
- typed HTTP requests and consistent error handling;
- actor selection;
- catalog operations;
- following and unfollowing operations;
- incoming-item or timeline operations once their API contract exists.

All ROTT interfaces should share this integration through the core. API compatibility and consistent authentication behavior with the `slug` CLI follow the shared schema/contract, without requiring reuse of its TypeScript implementation.

## Catalogs and subscriptions

Initial ROTT connects to one website supplying one catalog. Combining catalogs is outside initial scope; cross-catalog duplicate handling should be revisited only if catalog combination is introduced later. No cross-catalog merge policy is selected.

The Slug website supplies the catalog of accounts the user might want to follow. Catalog presence alone does not create a subscription. A source or contact can have multiple accounts, and users can choose individual accounts.

Initially ROTT connects one account configuration. The model explicitly distinguishes website/API identity, any configured social actor, and catalog accounts, scoping follow relationships and publishing operations to the appropriate identity. Future multiple configured accounts and switching must remain possible, but multi-account UI is deferred.

RSS subscriptions and local selections are ROTT-owned and stored in Automerge. ActivityPub intent and the status of requests submitted online may be retained as personal state, but they do not form an offline follow queue or override SlugKit's confirmed follow list. ROTT requests follows/unfollows and reconciles to that authoritative server state, including changes made by other clients. Pending requests remain separate from Following until accepted; failed/rejected requests remain visible in management.

The catalog remains authoritative for the account information it publishes. Pressing Download refreshes the catalog. ROTT matches accounts by stable identity and updates names, avatar information, and other details while preserving subscriptions, saved posts, and notes. While offline, changes are not discovered until a successful refresh.

If a refresh confirms that an account is absent from the authoritative catalog, ROTT marks it unavailable and preserves saved posts, notes, and communications. A transient fetch failure or absence from a paginated/search subset does not establish deletion. Catalog disappearance does not automatically unfollow the server-side actor or erase personal data; server-confirmed ActivityPub follow state remains authoritative.

ROTT stores and syncs basic subscription account metadata in Automerge—name, handle, source type, address, avatar information, and last known follow status. This supports an offline sidebar and consistent account lists across devices. One thousand subscribed accounts means retaining metadata for one thousand accounts, not selecting one thousand Automerge documents. The last known ActivityPub status is an offline snapshot; it never overrides authoritative server-confirmed state.

The schema, per-account document strategy, rotation thresholds, and avatar caching details are implementation work. Attachment storage follows the device-local policy below. Avatar references or local image caching remain implementation possibilities, distinct from the agreed avatar information. Document boundaries are deferred to implementation. Credentials remain in per-device platform secure storage and never in Automerge.

Following requires an online connection; there is no offline follow queue. Show failed requests to the user and reconcile with the website’s authoritative follow list on reconnect. Retry and operation-identifier mechanics remain implementation work. A pending request already submitted to the website is distinct from an offline queue.

## Data ownership and storage

The agreed distinction is between unrelated disposable incoming content and durable personal data, including communications. This replaces the earlier narrow boundary that excluded retained post bodies from Automerge. SlugKit remains separate as the website catalog/content API and canonical website publication boundary. The website implements optional social actions and handles ActivityPub delivery/signing when used; exact inbox/timeline/API coverage remains open under Q26, and the actor server retains authoritative follow state. Local communication history replaces neither website nor federation responsibilities. References to syncing in the table below apply when optional multi-device sync is configured.

| Data | Intended storage | Behavior across machines |
| --- | --- | --- |
| Source, contact, and account catalog | Slug website; full-catalog caching details undecided | Retrieved from the configured catalog provider |
| Basic subscription account metadata | Automerge: name, handle, source type, address, avatar information, last known follow status | Syncs for offline sidebar/account lists; status snapshot does not override server authority |
| RSS subscriptions/local selections and ActivityPub request intent | Automerge | Syncs between machines; ActivityPub intent does not override confirmed server state |
| Confirmed ActivityPub follow list | SlugKit authoritative; locally reflected by ROTT | Reconciled from the configured actor's server-confirmed state |
| Website address and account name/identity | Automerge, non-secret connection details only | Syncs across configured devices; each device authenticates separately |
| Authentication credentials | OS credential store | Device-local and never stored in Automerge |
| Website incoming posts | Locally persisted Automerge content, temporary unless protected | Shared through optional ROTT sync; no indefinite retention guarantee |
| Other feed/cache data | Local storage with disposable-content policy unless protected | Exact document/storage arrangement remains to be designed |
| Feed-fetch bookkeeping | Bounded tracking design still open; rotating disposable Automerge documents is a proposal | Carry-forward, retirement, and offline-peer behavior remain open during implementation |
| Per-item read/unread state | Automerge | Syncs independently of cached content |
| Explicitly saved items, tags, and private notes | Automerge | Preserved locally; syncs when multi-device sync is configured |
| Own published contributions and retained participated conversations | Automerge, including relevant text/context | Durable local history synchronized across configured devices |
| Liked remote posts and like state | Intended split: content/state in Automerge; minimal remote reference plus like on website | Available across Mac/iPhone; protection and reconciliation details remain open |
| Large media/attachments | Device-local files; original URL/basic metadata in post Automerge records | Binaries do not sync through Automerge; each device fetches its own copy on demand |

An item appearing in a feed does not automatically become a saved item. Saving it creates a durable entry in the personal collection. Clearing or expiring feed content must not remove saved items, their annotations, or retained communication history. Following an account does not archive all of its posts.

Participating in a conversation makes ROTT follow that thread, retrieve future replies, and retain the conversation and relevant context in Automerge. This extends beyond keeping only the user’s reply and its parent. Test the effectiveness of thread following during implementation; complete global discovery is not guaranteed. Learned comment edits update retained text; a product revision-history feature is not required, and this does not require erasing Automerge’s underlying history. Saved posts survive deletion of their originals and mark the originals deleted. Capture mechanics and automatic saved-item handling remain implementation work.

The exact database or cache technology is not yet chosen. SQLite was discussed as the kind of local storage a reader such as Newsboat uses, not as a finalized implementation requirement.

## Manual Download and website-copy removal

Opening ROTT shows locally stored items. Website incoming-post downloads happen only through an explicit **Download** action, not automatically on open or in the background. Downloaded posts are persisted locally in Automerge so other clients can receive their content through optional ROTT sync.

After successful local persistence, the default is for ROTT to send a separate request deleting exactly those downloaded/persisted incoming copies from the website. Fetching itself does not delete anything. Local persistence is sufficient before that request; confirmation of a second durable copy is not required. Users can configure ROTT to keep website copies indefinitely. This is removal of incoming copies, not deletion of the original author's post or the user's own published content.

Receiving incoming content does not automatically mark it saved/favorite or publish it. The default-unlimited/configurable-count cache policy still applies, while saved items and durable communications remain protected. Syncing temporary content through Automerge is now intended, but does not guarantee indefinite retention. Private Save and private notes never automatically upload or publish an original. The website incoming store remains separate from the optional Automerge sync server.

Concurrent manual downloads may rarely overlap or produce duplicates; that is acceptable, with basic stable-ID deduplication to implement. No exactly-once delivery guarantee is required. Without optional sync, a different client cannot assume it receives copies another client has downloaded and removed. Sync is not backup; recovery and retention remain separate design tasks.

Implementation requirements:

- Define batch pagination/cursors and follow-boundary identifiers so pre-follow ActivityPub history remains excluded; choose no fixed batch size here.
- Persist downloaded content successfully before requesting removal, target exactly the persisted incoming copies, and define authenticated removal/receipt and retry/idempotence behavior without selecting endpoint names.
- Implement basic stable-ID deduplication and failure handling for partial downloads, persistence failures, overlapping requests, and retry after uncertain responses.
- Implement the keep-website-copies option; document server storage bounds, remote edit/delete handling, and recovery implications. SQL deletes do not establish immediate hosting-cost reductions.

Q26 retains the exact API/authentication contract work; document lifecycle and communication capture mechanics are implementation work, while backup/recovery design is deferred.

## Incoming items and link discovery

The server-mediated pipeline is (the exact reading/action API contract remains open under Q26):

1. The configured actor follows a remote ActivityPub account.
2. The actor server receives activities; ROTT needs incoming-post access through an interface still to be verified or designed.
3. On explicit Download, ROTT persists incoming posts locally in Automerge for optional sync, then by default requests removal of exactly those persisted incoming website copies. Temporary local content remains subject to cache policy.
4. ROTT extracts external links from each post while retaining the source actor and post identity.
5. ROTT compares candidate links with the local review queue and saved-link collection.
6. The user reviews candidates, optionally edits metadata or assigns tags and notes, and explicitly saves selected links.

Saving and participating in communications also retain the relevant content/context in durable personal history; the exact capture mechanics remain to be designed.

Research still needs to define:

- which ActivityPub object and activity types count as posts;
- whether boosts, replies, quote posts, and pinned posts are included;
- which links are considered external rather than actor, hashtag, mention, attachment, or navigation links;
- URL normalization and canonicalization rules;
- how source-change payloads are delivered and represented, including delivery of updated comment text and original-deletion markers;
- how pagination, post-follow retrieval cursors, server retention, and rate limits work in the Slug API;

Different posts remain separate even when they link to the same article. ROTT does not merge posts or saved entries based on a shared destination URL or add cross-post canonical-link merging. This does not require duplicate copies of exactly the same record when syncing.

Apply source edits and deletions learned from the website to unsaved cached posts and sync those changes across devices. Preserve explicitly saved content when its original is deleted, and mark that original as deleted. The website must supply changes before ROTT can learn them; Automerge distributes learned changes but does not discover remote changes itself. Participated threads are followed for future replies and retained with context; learned comment edits update the retained text without requiring a revision-history feature.

## Feed fetching and reading state

The Rust core fetches, parses, and caches feeds and exposes filtered post lists, content, and available replies to the UI. For both RSS and ActivityPub, the default cache is **unlimited**, meaning no application-imposed item cap, not guaranteed access to a publisher's entire history. Users can configure a maximum cached article/post count per feed/account. The count includes read and unread items together; exceeding it removes the oldest cached items. Saved items and durable communication history are exempt from this eviction policy. Neither fifty unread items nor a page-size limit is the retention policy.

ActivityPub has **no historical backfill in the initial version**. Only posts received after the actor's follow/subscription are displayed; pre-follow cached history must not automatically appear. Post-follow posts still present on the website can be manually downloaded. Other clients receive downloaded content through optional ROTT sync; a new device must not assume removed website copies remain available there. This post-follow retrieval is not historical backfill. Future backfill may be considered separately. Exact follow-boundary identity/cursor APIs and pagination remain engineering details.

Website downloads are explicit user actions; optional Automerge sync distributes downloaded content across clients. Availability can still differ while devices are offline or after temporary content expires. Upstream content, website copies already removed, and expired local content are not guaranteed recoverable.

Readers distinguish several kinds of state:

- Fetch metadata, such as cursors, ETags, and Last-Modified values when supported, helps avoid downloading unchanged content.
- Item identifiers allow repeated fetches to recognize an existing item rather than create duplicates.
- Read/unread state records the user's interaction with an item, independently of whether it was fetched.

Read state uses read-wins behavior: reading a post on one device makes it read on other devices after sync, and stale unread state must not undo that read. The detailed conflict algorithm remains implementation work. This does not rule out a later deliberate Mark Unread action or select its semantics.

Read/unread state remains distinct from content retention. Website-downloaded bodies now also enter Automerge for optional cross-device sync, while temporary content can expire under the cache policy. Saved content and retained communications follow their protected durable-data policy.

Stable identifiers and record relationships are deferred to ROTT’s internal data-model/Rust storage implementation. Distinguish remote post identity from local saved-record identity and from a shared article URL where relevant. No ID scheme is selected; the design must accommodate protocol differences and missing or unreliable upstream IDs.

The website is not a guaranteed common historical archive after the default removal requests. ROTT uses optional sync to distribute manually downloaded content rather than requiring every device to fetch the same website copies.

## Bounded read-state tracking and document lifecycle

After an old unsaved post is removed, retain its read-state tracking for a limited period in rolling tracking documents, then discard that tracking. Saved items retain their read state. Exact limits are storage implementation details; this accepts bounded rolling retention without settling the full rotation/retirement/offline-peer protocol during implementation.

RSS bookkeeping should avoid an unbounded record for every item ever seen. A timestamp alone is unsafe as the sole deduplication boundary. Bounded recent-ID tracking is a suggested approach, not a selected algorithm; exact identifiers, timestamp use, and window sizes remain open.

Rotating disposable Automerge tracking documents by size, with durable saved content kept separate, is a proposed direction. Thresholds, document boundaries, retirement protocol, carry-forward of needed recent state, offline-peer handling, and history/storage reclamation need design during implementation. Deleting an Automerge field must not be assumed to prune its history or reclaim disk space.

Bookkeeping rotation is distinct from the content-count policy: per-feed/account content still defaults to unlimited, with an optional maximum counting read and unread together and evicting the oldest unprotected items. Rotation does not authorize losing saved content or durable communications.

## Local-first behavior and sync

The application should support browsing locally cached items and accessing, saving, tagging, and annotating locally available links without connectivity. Fetching new remote content and exchanging changes with other machines require connectivity.

Automerge is selected as a good fit for ROTT's local personal data. A single device, including a phone-only installation of the planned iPhone app, needs no sync server. Devices can make independent/offline edits and merge them on reconnect when sync is configured. Read-wins behavior is specified above; conflict semantics for other data still need design, since convergence alone does not choose the desired meaning of conflicting edits.

The current Rust implementation uses a WebSocket sync server, as documented in the [sync setup guide](../SYNC_SERVER_SETUP.md) and implemented in `crates/rott-core/src/sync/`. Peer-to-peer sync is not implemented. Sharing a Rust core across new interfaces does not itself establish compatibility with existing transport, document layout, or sync behavior; platform lifecycle constraints also need validation.

| Deployment option | Requirement and status |
| --- | --- |
| One local device | No sync server needed; local personal data remains on the device. SlugKit access is separate. |
| Home/private-network sync | Chosen plan for this user: reuse the default Automerge sync server behind the user’s VPN with network-restricted access and no separate app-level authentication for that private setup. Not a public/shared-hosting policy or a requirement for single-device use. |
| Other multi-device users | Need access to self-hosted or third-party hosted sync, or a future implemented peer-to-peer transport. No such P2P feature is currently available in ROTT. |
| Shared hosted sync service | Requires authentication and document authorization; reachability or knowing a document ID is not an adequate authorization model. |

VPN and network access controls restrict who can reach the sync service; they do not provide end-to-end encryption against that server. Pairing and authentication could be integrated separately, but are not verified ready-made features of the redesign.

Keyhive was discussed as an optional alpha access-control/end-to-end-encryption integration, not a selected dependency. No production date has been verified. Its maturity, security properties, and Swift compatibility require assessment before any adoption decision.

The plan is to reuse the default Automerge sync server behind the user’s VPN. New-client compatibility has not been verified; connection, protocol, and version checks remain routine implementation validation. Document organization remains implementation work during implementation. The private setup’s lack of separate app-level authentication does not apply to public/shared hosting; hosted-service authorization design is deferred until a hosted service is considered.

Sync is not backup. Document boundaries, sizes, rotation/retirement, offline-peer handling, and conflict engineering are deferred to implementation. Bounded temporary tracking and protection of saved data/communications remain requirements; no exact lifecycle protocol is selected. Backup, export, and recovery design is deferred as the design evolves. Personal data should not be trapped, but no initial export feature, format, or timing is committed. Hosted sync authentication, device authorization, pairing, and any Keyhive evaluation are deferred until a hosted service is considered; the personal VPN sync plan is unchanged.

## Coexistence with the existing Rust CLI/TUI

The existing Rust ROTT is used daily and must remain usable while the new application is explored.

The proposed separation is:

- A distinct development command, such as `rott-next`, while retaining ROTT as the product name.
- Separate configuration and data directories.
- Separate Automerge documents and sync identity for the new application during development.

These are isolation choices, not finalized names or paths. The new application should not write into the current Rust application's data or assume it can change its schema. If migration is pursued later, use a separate helper script or application built on ROTT facilities. A built-in importer is neither required for the initial application nor authorized now. The existing CLI/TUI remains isolated and usable.

## SlugKit integration context

**API additions and extensions must be implemented in the SlugKit project.** SlugKit owns the shared schema/API contract; optional website backends implement the corresponding storage and federation behavior; ROTT’s Rust core consumes that contract. The [API gap review](ROTT-API-GAP-REVIEW.md) is the overall baseline checked against current source and the deployed schema. Design and implement detailed contracts per feature later; this document authorizes neither implementation nor task creation.

The inspected SlugKit implementation provides catalog/source/contact/account operations and API-key login, plus ActivityPub follow management as an implementation-specific capability. Publishing a federated website and operating a reader have different requirements, so incoming-item client APIs still need focused design.

The CLI compatibility check confirms the example site's API is discoverable and usable by the configured Slug CLI. ROTT will integrate through its shared Rust core; its exact client module and API coverage still need design. ROTT calls the shared API contract directly through its core and does not invoke the CLI as a subprocess.

## Combined incoming stream

The website exposes one combined incoming stream for the connected account across its followed accounts. The explicit manual Download action retrieves combined batches, and ROTT filters the locally stored items by the selected followed account. Separate per-followed-account streams are not required initially.

The stream is scoped to the connected account/actor; it does not mix multiple configured identities. Only post-follow content is included, with no initial historical backfill. Downloaded batches are persisted locally in Automerge before ROTT's separate default request to delete exactly those persisted incoming website copies; the keep-website-copies option remains available. Fetching alone does not delete content.

This website incoming API is not an RSS proxy; RSS fetching remains in the Rust core. Exact endpoint paths, batching, pagination/cursors, follow-boundary identifiers, authentication, and removal targeting/retries remain implementation requirements under the established API contract work, not an unresolved choice between combined and per-account streams.

Direct public polling was an earlier fallback idea, not an initial ActivityPub commitment. It must not bypass the no-backfill or server-confirmed follow boundaries. RSS fetching belongs in the core; optional platform integrations remain subject to their own scope decisions.

The selected action model does not require ROTT to host a public actor/inbox or hold the actor signing key. Reading access and follow-state coordination still need an explicit server API contract.

## Confirmed retention and deferred engineering

Participating in a conversation makes ROTT follow that thread, retrieve future replies, and retain the conversation and relevant context in Automerge. This extends beyond keeping only the user’s reply and its parent. Test the effectiveness of thread following during implementation; complete global discovery is not guaranteed. Learned comment edits update retained text; a product revision-history feature is not required, and this does not require erasing Automerge’s underlying history. Saved posts survive deletion of their originals and mark the originals deleted. Capture mechanics and automatic saved-item handling remain implementation work.

Attachments are downloaded device-local files. Their binaries are not stored in Automerge or synchronized through it. Post records in Automerge retain original URLs and basic media metadata; each client fetches its own copy on demand. If the original disappears, a new fetch may fail while an existing retained local copy remains usable. No peer blob transfer, cloud attachment service, eviction policy, or exact automatic-fetch timing is selected.

Webpages open in the default system browser or appropriate application. ROTT saves URL, title, and notes; it does not provide a built-in full webpage reader, reformatter, or full article archive. RSS/social post text and retained conversations remain displayable and retainable in ROTT.

Document boundaries, sizes, rotation/retirement, offline-peer handling, and conflict engineering are deferred to implementation. Bounded temporary tracking and protection of saved data/communications remain requirements; no exact lifecycle protocol is selected. Backup, export, and recovery design is deferred as the design evolves. Personal data should not be trapped, but no initial export feature, format, or timing is committed. Hosted sync authentication, device authorization, pairing, and any Keyhive evaluation are deferred until a hosted service is considered; the personal VPN sync plan is unchanged.

If migration is pursued later, use a separate helper script or application built on ROTT facilities. A built-in importer is neither required for the initial application nor authorized now. The existing CLI/TUI remains isolated and usable.

## Open design questions

Question IDs remain unchanged when questions are resolved and removed. New questions receive the next unused number; retired IDs are not reused. Refer to questions by these IDs and/or their exact wording. Earlier references below to a Slug actor/API describe the ActivityPub-enabled example implementation, not a universal SlugKit requirement; Q26 records the clarified boundary.

- **Q26**: As each feature is implemented, what SlugKit contract additions are needed for external-item identity, account identity, optional capability reporting, incoming batches/removal and source changes, remote thread following/retrieval, and outgoing Like/Share/quote/reply actions, including pending/confirmed reconciliation and unsupported-action responses? Use the API gap review as the baseline; existing follow lifecycle support needs integration validation, not an invented replacement.

These questions are intentionally left open. The discussion establishes product behavior and data boundaries without authorizing implementation or committing to a detailed architecture.

## Potential implementation follow-ups

These are candidate follow-up tasks, not implementation authorized by this design task:

1. Implement needed schema/API additions in **SlugKit**, with optional website backend/federation support and ROTT Rust consumption, feature by feature using the [API gap review](ROTT-API-GAP-REVIEW.md). No implementation or task creation is authorized here.
2. Specify the fresh Rust core's SlugKit integration and Swift bridge/API; evaluate candidate bindings without assuming a legacy-code or npm client dependency.
3. Define subscription, incoming-item, candidate-link, and saved-link identity schemes.
4. Define Automerge schemas and document boundaries for personal state, saved items, and durable communication text/context while keeping unrelated incoming posts disposable.
5. Design the local cache and fetch bookkeeping model.
6. Prototype ActivityPub post and external-link extraction against representative Slug-hosted actors.
7. Design the review queue and save-to-collection workflow.
8. Validate connection/protocol/version compatibility with the planned default Automerge sync server and implement the document organization.
9. Preserve coexistence boundaries; consider a separate migration helper only if migration is later pursued.
