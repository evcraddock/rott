# Accounts, catalogs, and follows

[Specification index](README.md)

## Discovery and subscription ownership

Initially ROTT must connect to one website supplying one catalog. Catalog combination and cross-catalog duplicate/merge policy are outside initial scope.

The catalog contains sources, contacts and accounts. A source or contact can have multiple accounts, and sources can link to contacts. Users must be able to browse these associations and select individual accounts. Catalog presence must not create a subscription.

The account picker must support name/handle discovery with results showing source/contact associations and accounts. SlugKit search source versus website deployment coverage is recorded in the [API gap review](../redesign/ROTT-API-GAP-REVIEW.md); the requirement does not imply that deployed search is ready or that the current label/URL search fully satisfies handle discovery. Full-text post search is deferred.

| Account type | Required or possible behavior |
| --- | --- |
| RSS | ROTT owns subscription/local selection. The Rust core fetches, parses and caches feed items; no remote social follow is implied. |
| ActivityPub | Online follow request through the website's optional semantic API; Following membership requires acceptance. Incoming reading depends on the required website integration. |
| Bluesky / AT Protocol | Public-author-feed reading is an optional design possibility. Reading does not create a remote follow; authenticated following requires a separate decision. No publishing commitment. |
| Other accounts, including Facebook/X | Selected sidebar destinations can open a webpage or appropriate app. No feed retrieval or remote follow is promised. |

A selected account may remain useful as an external destination when feed reading is unavailable. The UI must distinguish readable feeds and external navigation; exact presentation and supported app links remain platform design work.

## Catalog refresh and offline metadata

Pressing Download must refresh the catalog. Match stable account identities and update names, avatar information and other details while preserving subscriptions, saved posts and notes. Offline clients learn changes only after a successful refresh.

Confirmed absence from the authoritative catalog must mark an account unavailable and preserve saved posts, notes and communications. Fetch failures or omissions from search/paginated subsets are not authoritative absence. Catalog disappearance must not automatically unfollow a social actor or erase personal data.

Store basic subscription metadata in Automerge: name, handle, source type, address, avatar information and last known follow status. It must support offline sidebar lists and optional cross-device consistency. The last known social status is a snapshot and must not override server authority. Supporting one thousand account metadata records does not select one document per account.

Record schemas, stable identity mechanics, document layout and avatar caching are implementation details. Credentials remain outside Automerge; attachment storage follows [data and sync](data-and-sync.md).

## Authoritative social follows

The website's server-confirmed follow list is authoritative, including changes made by other clients. Follow and publishing operations must be scoped to the configured actor. ROTT requests follow/unfollow and reconciles to server state on reconnect.

Following requires an online connection; there is no offline follow queue. Local intent and already-submitted request status may be stored as personal state but cannot override the server. Failures must be shown to the user.

An ActivityPub account must enter Following only after acceptance. Requested/Pending follows must remain discoverable and cancellable in follow management. Rejected/failed requests must stay out of Following and show their status in management. Exact screen/tab/filter placement is open; visibility and cancellation are requirements.

Existing server follow delivery, Accept/Reject handling and pending cancellation are documented in the API review. Retry identifiers and client/server reconciliation tests are implementation work; a submitted pending request is not an offline queue.

## Optional Bluesky boundary

Public reading can use the [author-feed API](https://docs.bsky.app/docs/api/app-bsky-feed-get-author-feed) without Bluesky login. SlugKit authentication does not authenticate an AT Protocol account. A PDS hosts an account, an AppView serves assembled feeds, and relays aggregate repository events; hosting a PDS alone does not assemble incoming feeds. See [AT Stack](https://atproto.com/guides/the-at-stack) and [self-hosting](https://atproto.com/guides/self-hosting).

No PDS/AppView/relay deployment or Bluesky authentication method is selected. Self-hosting is not required for public reading. Whether any initial Bluesky integration extends beyond public reads remains uncommitted.
