# Fediverse link-discovery research conclusion

Task: `task-d922c229` — Research Fediverse account following for link discovery.

Status: research conclusion, not an implementation or a claim of live interoperability. This document closes the original retrieval/comparison question; the broader [design](ROTT-DESIGN.md), [reader overview](ROTT-READER-OVERVIEW.md), and [technical specifications](../spec/README.md) retain the application requirements. No account was followed and no social mutation was performed for this conclusion.

## Retrieval approaches and tradeoffs

| Approach | Compatibility and authentication | Benefits | Limitations and cost |
| --- | --- | --- | --- |
| Website receives ActivityPub deliveries; ROTT downloads one combined incoming stream | Requires an ActivityPub-enabled website and the missing optional SlugKit incoming API. ROTT uses its website bearer API key; the website owns actor signing keys and federation. | Matches the configured actor's accepted follows; receives posts while ROTT is closed; centralizes federation and gives the client one paginated stream. | Requires website storage and new API work. Delivery and thread visibility are not globally complete. Retention, source changes, pagination and removal need explicit contracts. |
| Website exposes a stream per followed account | Same website/API credentials and federation boundary as the combined stream. | Straightforward account-specific retrieval and filtering. | Adds per-account cursors, request fan-out and client-side ordering/aggregation. Does not avoid the incoming-store/API work. No advantage sufficient to prefer it for initial ROTT. |
| Poll public ActivityPub outboxes directly | ActivityPub defines actor outboxes and public reads; actual exposed history, pagination and access policies need server-specific validation. Authenticated access is a separate concern; a SlugKit API key does not authorize another server. | Protocol-level discovery without requiring every source to offer a Mastodon-compatible API. | Public polling is not an accepted follow or inbox delivery. Coverage/history vary; polling many actors increases requests and can miss intervening changes. Client-side federation hosting/signing is a different, substantially larger project and is not selected. |
| Poll a platform API, such as Mastodon's account-status endpoint | Mastodon documents public-status reads without a token and authorized private reads with a user token plus `read:statuses`. Requires that platform's API, account IDs and any necessary credentials; ActivityPub compatibility alone does not imply Mastodon API compatibility. | Structured status records, pagination and reply/boost filters make a targeted adapter practical. | Adds platform-specific adapters and authentication. A server only exposes content available to it. Public polling does not perform a follow; independent requests face platform rate limits and need change/deletion handling. |
| Poll account RSS/Atom feeds where offered | Requires an exposed feed, not universal Fediverse support. Public feeds usually need no account login; Mastodon documents discoverable account RSS feeds. | Simplest read-only link-discovery option; fits the planned Rust feed-fetching path and needs no federation credentials. | Finite feed windows can lose posts between fetches; feeds may contain only summaries. No general follow acceptance, private-content access, reply completeness or reliable deletion notification. |

ActivityPub specifies both client-to-server and server-to-server interactions, but its presence does not establish that a particular website supplies ROTT's required client operations. The [API gap review](ROTT-API-GAP-REVIEW.md) distinguishes existing SlugKit follow support from missing incoming retrieval and remote-action contracts. The alternatives above are feasible under their stated prerequisites, not features already validated on every server.

## Recommendation

Use **server-mediated ActivityPub following with one combined SlugKit incoming stream**, and keep **RSS fetching in the Rust core** as a separate subscription path. This fits the selected website identity and follow authority without giving ROTT actor keys or turning desktop/phone clients into always-reachable federation servers. Per-account website streams add avoidable request and cursor complexity. Direct outbox/platform polling remains an alternative for separately scoped integrations, not an automatic fallback that bypasses follow acceptance or the initial no-backfill rule.

The recommended flow is:

1. Submit a follow online through the existing website API and reconcile its accepted/pending/failed state. Only accepted follows enter Following.
2. Let the website receive eligible post-follow activities. On explicit Download, ROTT retrieves the combined stream and persists content locally in Automerge.
3. Only after successful persistence, separately request removal of exactly those incoming website copies by default; offer indefinite website retention. Optional ROTT sync distributes downloaded content, but is not backup.
4. Extract candidate external links with source actor and post identity. Present them for review, metadata edits, tags and private notes before explicit saving. Discovery alone never adds a durable saved entry or publishes anything.

This preserves the existing save/tag/note workflow conceptually; it does not require changing the current CLI/TUI or reusing its code/data. The new application and any later migration remain separate.

## Deduplication, changes and request limits

- **Link extraction:** inspect structured post/feed content rather than treating every URL as a shared article. Distinguish external destinations from mentions, hashtags, actor/profile links, attachments and navigation. Exact object-type coverage, HTML parsing and URL normalization need representative fixtures during implementation.
- **Deduplication:** recognize repeat downloads by stable source-item identity and avoid duplicating the same candidate from the same post. Compare against the review queue and saved collection, but do not merge distinct posts or saved entries merely because they share an article URL. Cross-protocol identity and unreliable feed IDs remain engineering questions; a timestamp alone is not a safe deduplication boundary.
- **Edits/deletions:** apply learned source changes to unsaved content and retain saved text with an original-deleted marker. Reconcile affected pending link candidates when implementing the review queue. An item falling out of a feed or page is not proof of deletion. The website must expose changes even after incoming-copy removal; Automerge only distributes changes already learned. Delivery of every remote edit/delete is not guaranteed.
- **Polling versus delivery:** federation moves ongoing upstream reception to the website, not to a background ROTT downloader. Public API/feed polling instead scales with accounts, pages and polling frequency and can miss short-lived posts. Use conditional HTTP requests where supported and bounded recent-ID tracking; conditional requests are not assumed exempt from provider limits.
- **Rate limits:** Mastodon's documentation lists default limits of 300 requests per five minutes per account and per IP, plus response headers `X-RateLimit-Limit`, `X-RateLimit-Remaining` and `X-RateLimit-Reset`. These are a reference point, not SlugKit limits or a guarantee for every deployment. A 1,000-account polling sweep can exceed that budget when concentrated on one instance/IP, even before pagination. A future adapter should honor advertised limits and `Retry-After` when provided, back off on throttling, bound concurrency and retain cursors for resumption. Combined batches reduce client request fan-out but do not remove server ingestion/storage costs. Actual SlugKit quotas, page sizes and retry semantics remain feature-level API work.

## Limitations and implementation follow-ups

The initial ActivityPub reader is post-follow-only, with no historical backfill. Removed website copies are not available to independently configured devices unless content arrives through optional sync. Neither federation nor a platform API guarantees a complete global conversation. Public-feed/API alternatives do not provide followers-only access simply because the user has authenticated to SlugKit.

Follow-up work is identified, not implemented or created as new tasks here:

1. **SlugKit contracts and website backend:** add account identity/capabilities, combined incoming retrieval, exact-copy removal, source-change delivery and thread retrieval. Define identifiers, post-follow boundaries, pagination, access and throttling/retry behavior per feature. Reuse and validate existing follow lifecycle operations.
2. **ROTT Rust integration:** implement typed requests, explicit Download, persistence-before-removal, stable-ID deduplication and failure/retry handling; reconcile authoritative follow state.
3. **Link-review workflow:** test representative posts/feeds, link classification, repeated downloads, edited candidates and deleted originals before implementing explicit save/tag/note actions.
4. **Storage and validation:** design protected saved/communication records versus temporary caches; validate optional sync, offline peers, bounded bookkeeping and isolation from existing ROTT data. Test authorized integration behavior separately, including throttling and incomplete delivery.

The broader [implementation checklist](../spec/implementation.md) also covers publishing, platform interfaces and deferred product decisions. Those are beyond the original link-discovery research task and are not prerequisites for concluding this research.

## Sources and evidence boundaries

- [ActivityPub specification](https://www.w3.org/TR/activitypub/), particularly outbox access and server delivery. The [W3C-hosted source rendering](https://w3c.github.io/activitypub/) was read for this comparison.
- [Mastodon account-status API](https://docs.joinmastodon.org/methods/accounts/#statuses): authentication, pagination and status filters.
- [Mastodon rate limits](https://docs.joinmastodon.org/api/rate-limits/): documented defaults and response headers; not a measurement of any configured server.
- [Mastodon syndication feeds](https://docs.joinmastodon.org/user/network/#syndication-feeds): account/tag RSS discovery.
- [ROTT API gap review](ROTT-API-GAP-REVIEW.md): recorded SlugKit/website source and deployed-schema evidence, with explicit live-validation limits.

External documentation was read without authentication. No live account retrieval, rate-limit experiment, follow, publication or backend implementation was performed for this conclusion.
