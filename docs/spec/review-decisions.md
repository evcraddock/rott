# Review decisions — September 26, 2026

[Specification index](README.md)

> The [October 7 reader product decision register](reader-product-decisions.md) resolves the remaining product questions and supersedes this review only where expressly stated. This September review remains the source for all other accepted amendments.

These decisions amend the September 25 specification and take precedence where its storage, Download, read-state or backfill wording differs. The preserved redesign documents remain historical sources. This review authorizes documentation updates only.

## Feed delivery and saving

- ROTT must support normal incoming ActivityPub boosts, including a new boost referencing an older original post. Following an actor does not trigger retrieval of that actor's posting history. Do not reject a boosted original solely because it predates the follow.
- Downloaded, unsaved RSS and ActivityPub post content must remain in a disposable device-local cache outside Automerge. Eviction on one device must not delete another device's cached content.
- Explicitly saving a post must retain its text and all available comments in Automerge for durable storage and optional sync. This does not require saving the author's other posts or guarantee discovery of every remote comment. Attachment binaries remain device-local.
- Posting a public reply counts as intentional saving: ROTT must automatically preserve the replied-to post, the user's reply and the available conversation/context in durable Automerge storage, without requiring a separate Save action. Future direct replies addressed to the user arrive through normal ActivityPub delivery to the connected website's inbox. ROTT must retrieve those replies on Download and add them to the retained conversation. Receiving direct replies does not require a separate thread-subscription mechanism. The earlier phrase 'follow the thread' must not be interpreted as a guarantee of receiving every branch of a conversation; any broader retrieval requires separate design. Retrieval remains subject to explicit Download; this decision does not introduce background downloads or notification alerts. Participated conversations are protected from temporary-cache eviction.
- Unsaved cached content may expire after a retention period whether opened or skipped. The period and interaction with count limits remain to be specified. Cache expiration must not itself acknowledge a post as read.

## Acknowledgment and unread lists

- Merely displaying or scrolling past titles/previews must leave items unread.
- Clicking/opening a post must mark it read. Read means acknowledged, not proof that the entire text was read.
- A Mark all read command is required. Its precise scope remains to be specified; limiting it to the loaded list was proposed to avoid acknowledging newly arriving unseen items.
- Read acknowledgments must carry across devices; once an update arrives, the item must leave the other devices' unread lists. This does not delete saved items. Offline devices may temporarily disagree, and stale unread state must not undo a read.
- Download alone must not delete website incoming copies. Unread ActivityPub items must remain available for other devices to retrieve. The agreed direction is to use the connected SlugKit website's incoming store and shared acknowledgments for ActivityPub; this is separate from the Automerge sync service and requires new SlugKit contracts/backend behavior.
- Exact website cleanup timing after acknowledgment, retention limits and offline acknowledgment retries remain implementation/design work. The earlier persistence-then-immediate-removal rule is superseded.

## RSS and bounded Automerge tracking

- Each device fetches RSS directly. A shared website RSS fetcher is not selected; hosting traffic, storage and operating cost are concerns. An entry dropped by its publisher may be unavailable to a device that never fetched it.
- Sync small RSS read-status records through Automerge, without unsaved post bodies. This metadata is an explicit exception to the restriction on unsaved content in Automerge; existing subscription/configuration metadata remains separate as well.
- Identify an item consistently across devices using feed identity and the publisher's item identifier where usable. The shared Rust implementation must define deterministic fallback behavior for missing or unreliable identifiers; no exact fallback is selected.
- Read tracking must have bounded storage and use separate, replaceable Automerge documents. Rotate them, carry forward still-needed records, and retire old documents, including storage cleanup on clients and the sync service. Deleting fields alone must not be treated as reclaiming Automerge history.
- Keep durable saved content separate from disposable tracking. Exact rotation triggers, tracking windows, offline-peer reconciliation and prevention of retired-document resurrection remain implementation work. Forgetting tracking too early can make old entries reappear unread and must be addressed.

## Remaining review questions

The original backfill/thread conflict finding is withdrawn. Normal boosts and saving available comments are compatible with the intended reader behavior. Device-local unsaved caches resolve the original cross-device eviction ambiguity.

The Download command must retrieve website ActivityPub incoming posts, fetch RSS entries directly from subscribed feeds, and refresh the account catalog. This resolves the third original review finding by defining an explicit RSS refresh trigger. No additional launch/background RSS fetching is selected by this decision. The manual website Download rule remains in place.

Automatic retention after a public reply and capture of subsequent direct replies through normal inbox delivery are confirmed above: replying counts as intentional saving. Like-related content retention and whether merely saving a thread (without replying) subscribes to future replies remain unresolved. No silent change to those behaviors is selected here.
