# ROTT API gap review

Reviewed: September 25, 2026. Scope: one overall gap pass for the accepted ROTT requirements; detailed contracts follow per feature. No API changes, tasks, authenticated requests, or mutations were performed.

**API additions and extensions must be implemented in the SlugKit project.** SlugKit owns the shared protocol-neutral schema/API contract. Website implementations provide optional backend storage and federation behavior. ROTT consumes the contract through its Rust core. ActivityPub is not a mandatory SlugKit capability, and ROTT does not hold actor signing keys or implement federation separately.

## Evidence and limits

- SlugKit checkout: `/Users/erik/Private/code/forgejo/slugkit`, HEAD `791c239`; inspected current files, particularly `template/site/src/api/openapi.ts`, `template/site/src/api/routes.ts`, route handlers, and account filtering. Shared server building blocks also reside in `packages/slug-api/src/index.ts` and `packages/slug-core/src/index.ts`.
- Website checkout: `/Users/erik/Private/code/forgejo/erikvancraddock.com`, HEAD `5e46f65`; inspected current `src/api/openapi.ts`, route registration/handlers, and federation/comment handlers.
- Read-only GET of the deployed [OpenAPI schema](https://erikvancraddock.com/api/v1/openapi.json) succeeded. It advertises Slugkit API `1.0.0`; SHA-256 of the fetched response: `ffc95d0c21733a4b64e5f5649a82258b13f086216915eb08bfae5480031e10e1`.
- All routes below are relative to `/api/v1`. Schema references use the fetched document's `paths` and `components.schemas` unless explicitly labeled SlugKit-only. Source paths below are relative to the applicable checkout. A HEAD label identifies the checkout, not proof that every inspected working file is committed or deployed.
- Schema presence establishes advertised operations; code establishes local implementation evidence. Neither establishes live authenticated behavior, successful delivery to remote servers, or deployment parity. Only retrieval of the public schema was live-validated in this review.

## Existing usable contract surface

These operations can be reused as the integration baseline, subject to feature-level validation.

| ROTT need | Verified methods/routes | Evidence and boundary |
| --- | --- | --- |
| Discovery and diagnostics | `GET /health`, `GET /meta`, `GET /openapi.json`, `GET /auth/check` | Both source route sets and deployed paths; `HealthResponse`, `MetaResponse`, `AuthCheckResponse`. Authentication currently returns only `data.authenticated: true`. |
| Catalog browsing | `GET /sources`, `GET /sources/{id}`, `GET /contacts`, `GET /contacts/{id}`, `GET /accounts`, `GET /accounts/{id}` | Both route sets and deployed paths; `Source`, `Contact`, `Account`. Accounts expose stable numeric ID, owner association, label, URL, avatar URL, kind and protocol. `/accounts` filters by `ownerType`/`ownerId`. Catalog CRUD is also advertised, but ROTT browsing does not require mutations. |
| Own feed and note/link publication | `GET /posts`, `GET /posts/{slug}`, `POST /posts`, `POST /posts/{slug}/publish`, `PUT /posts/{slug}`, `DELETE /posts/{slug}` | Both `api/routes/posts.ts` implementations and deployed paths; `CreatePostRequest` supports `note`, `link`, `article`. Ordinary creation is a draft followed by publication; supplying migration `publishedAt` creates an already-published post without follower delivery. ROTT excludes article authoring and has no initial importer. Website `src/federation/publish.ts` supplies Create/Update/Delete delivery code. |
| Own-post comments and engagement | `GET /posts/{slug}/comments`, `GET /comments`, `GET /posts/{slug}/engagement`, `GET /posts/{slug}/likes`, `GET /posts/{slug}/boosts` | Both comments/engagement route files and deployed paths; `Comment`, `EngagementSummary`, `EngagementActor`. Incoming likes/boosts on site-owned posts are not outgoing action records. |
| Authoritative follow management | `GET /following`, `POST /following`, `POST /following/{id}/unfollow` | Both `api/routes/following.ts` files are identical. `Following` includes pending/accepted/rejected/failed/cancelled/unfollowed, timestamps, profile information and `lastError`. Unfollow accepts pending or accepted records. Website `src/federation/following.ts` sends Follow/Undo and processes Accept/Reject; `src/federation/routes.ts` registers listeners. Pending cancellation is covered by the existing unfollow operation. Live federation remains untested here. |

## Existing operations needing extensions or deployment work

- [ ] **Identity — SlugKit contract extension:** extend `GET /auth/check` / `AuthCheckResponse` with account identity and optional ActivityPub actor address. Specify exact fields and site-wide-key/account mapping per feature. Keep the accepted initial full-access-key policy and per-device secure credentials; no new `whoami` route.
- [ ] **Catalog search — deploy existing SlugKit work and check fit:** current SlugKit `template/site/src/api/openapi.ts` and `api/routes/accounts.ts` support `q` on `GET /accounts`; `accounts/accounts.ts` matches a trimmed case-insensitive literal substring of label or URL. Current SlugKit also documents `q` for `GET /sources` and `GET /contacts`. Website account route code lacks `q`, and the deployed schema does not advertise these search parameters. Name/handle discovery needs feature validation against real account labels/URLs and associations; do not claim a dedicated handle-search field or live readiness.
- [ ] **Capabilities — SlugKit contract addition:** neither `HealthResponse` nor `MetaResponse` currently declares incoming-read/follow/like/share capabilities. Select report location, fields, versioning and schema mapping later. Existing `NotImplementedResponse`/501 behavior provides a baseline; define consistent unsupported-action behavior for added optional operations. An actor URL is insufficient.
- [ ] **Follow integration — reuse and validate:** existing server delivery/state transitions are present. ROTT still needs online requests, pending/rejected/failed presentation, cancellation and reconciliation against the authoritative list; feature tests must cover failure/retry and incoming decisions. This is not a missing follow API redesign.

## Missing required client operations

No supporting client routes were found in either inspected API registration/route set or the deployed schema. The descriptions below are requirements, not proposed route names.

- [ ] **Combined incoming stream and exact-copy removal:** post-follow-only batches across the connected actor's follows; pagination/stable identifiers/follow boundary; a separate removal operation targeting exactly the persisted website copies; configurable indefinite server retention. Fetching must not delete. The website needs the backing incoming store and optional federation ingestion; ROTT persists locally before removal. This is not an RSS proxy.
- [ ] **Remote conversations and thread following:** retrieve remote post/conversation content and future replies after participation; deliver learned edits/deletions so ROTT can retain updated text/context. Existing `POST /posts/{slug}/comments` creates a site comment, not an arbitrary remote reply. Website `src/federation/replies.ts` handles incoming Create replies; `src/comments/comments.ts::getPublishedPostTarget` accepts only published posts under the local origin's `/posts/{slug}`. This does not implement a general followed-account inbox or remote thread store. The registered inbox listeners also do not establish a general incoming Update/Delete stream.
- [ ] **Outgoing Like and Share:** generic external-item references, action submission and state/record reconciliation, including pending/confirmed Like state. Existing own-post `GET .../likes` and `GET .../boosts` do not satisfy this. Use Share/Shares in the semantic contract; optional ActivityPub backend maps plain reshare to Announce. No route rename is chosen.
- [ ] **Quote and remote reply publication:** explicit relationships to remote items, supported quote permissions and clear unsupported behavior. `CreatePostRequest` has no quote or reply relationship fields; publishing a link is not equivalent. Extend the SlugKit contract per feature and implement corresponding optional website delivery.

## Follow-up ownership

1. **SlugKit:** implement shared schema/API additions and compatibility/error semantics for identity, capabilities, external items, incoming retrieval/removal/source changes, threads, and outgoing actions. Specify only the feature being built; do not design every endpoint upfront.
2. **Website backend:** adopt applicable SlugKit changes, deploy catalog search, implement optional incoming storage/thread retrieval and social delivery, and retain authoritative follow/action state. Federation presence does not prove every operation works.
3. **ROTT Rust core:** consume typed requests/responses directly, validate compatibility, implement read-only doctor and per-device credential integration, reconcile server state, and persist content before requesting removal. No CLI subprocess or shared TypeScript client is required.
4. **Feature validation:** verify deployed search and authenticated reads/actions in separately authorized implementation work. This pass does not run mutations or certify delivery. Retention of liked content and the default publishing audience remain product details in the design follow-up list.
