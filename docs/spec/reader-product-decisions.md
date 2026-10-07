# Reader product decision register — October 7, 2026

[Specification index](README.md)

Status: product decisions approved by Erik for task-fdf47dc9. This register takes precedence over conflicting earlier reader-policy wording, including the September 26 review, only for the decisions explicitly listed here. Accepted amendments and scope boundaries otherwise remain intact. It records required behavior, not completed implementation or verified backend support.

## Approval and sources

Erik approved P1 and P3–P8 in this task's voice conversation on October 7, 2026, and authorized documentation and PR preparation afterward. P2 was already approved in chat on October 7 and recorded in task-fdf47dc9 note-ef525fe6 and task-5e49adae. Each entry below records its rationale, affected task IDs and the relevant conversation source. These approvals do not authorize live website mutations or select engineering defaults.

## P1 — Default publishing audience

**Approved by Erik.** Deliberately published notes, link posts, quotes and public replies default to public visibility: anyone may see them. Private Save and annotations remain private. Audience controls remain deferred; unsupported or impermissible publication must be visible rather than silently changing the intended action.

Rationale/source: after discussing sharing an article through the website, Erik chose visibility to anyone by default. This establishes the publishing default without adding audience controls.

Affected tasks: task-a057f6fe, task-c856395c, task-2a2fbb5d, task-84b69694, task-b7ed859f, task-6d0b459c.

## P2 — Cache expiry and count limits

**Previously approved by Erik.** Time expiry is disabled by default. Users may configure an optional duration per feed/account, measured from first local receipt rather than original publication time; repeated downloads do not reset it. Apply age expiry before count eviction. Read and unread temporary content are eligible; saved items, notes, own contributions and participated conversations remain protected. Neither expiry nor Download acknowledges an item.

The unlimited-by-default item-count policy remains unchanged. An optional age limit can expire temporary content even without a count cap; an optional count limit still counts read and unread together and evicts oldest unprotected content after age expiry.

Rationale/source: the prior approved P2 note preserves local receipt age and separates cleanup from acknowledgment. This conversation did not reopen that choice.

Affected task: task-5e49adae; Like protection also affects task-7aab15c6 under P4.

## P3 — Mark all read snapshot

**Approved by Erik.** Mark all read acknowledges only the items already loaded in the current list when the command is invoked. Capture that set of item identities; do not broaden it to unloaded items, other lists or arrivals during execution. Later arrivals remain unread. This is not limited to the visible viewport within the loaded list.

Rationale/source: Erik answered yes to current-loaded-list scope with later arrivals left unread. A stable snapshot prevents concurrent unseen arrivals from being accidentally acknowledged.

Affected tasks: task-3c923de0, task-4675af07, task-db998439, task-84b69694, task-b7ed859f, task-6d0b459c.

## P4 — Like also saves content

**Approved by Erik.** Like must retain the post content, protect it from temporary-cache cleanup and put it in the explicit saved collection as well as Liked posts. Use the existing Save boundary: preserve post text and all available comments in durable Automerge storage; attachment binaries remain local. Social Like still sends the author-facing interaction and does not automatically create a public feed post or Share.

Rationale/source: Erik said Like should save the content, then confirmed that it should appear in the saved collection as if Save had been pressed. Private Save still does not send a Like.

Affected tasks: task-7aab15c6, task-5e49adae, task-2a2fbb5d, task-84b69694, task-b7ed859f, task-6d0b459c. Unlike/unshare behavior and action failure/retry mechanics are not selected here.

## P5 — Incoming website copies after acknowledgment

**Approved by Erik.** Reading/opening acknowledges a post and requires removal of its incoming website copy. Mark all read uses the P3 acknowledgment boundary. No indefinite keep-copies option applies to acknowledged incoming copies. Removal concerns that incoming copy, not the author's original or an owned publication. Content intentionally saved persists through Automerge; unsaved bodies remain local.

Download alone and cache expiry remain distinct from reading and must not acknowledge or remove incoming copies. Unread incoming copies remain retrievable until acknowledgment. This does not reinstate persistence-then-immediate-removal after Download. Once acknowledgment and removal are applied, the website must not offer that incoming copy for another download; already-downloaded local copies are not remotely erased. Offline acknowledgment queues and retries must deliver the removal when connectivity permits; exact atomicity, timing and failure mechanics remain contract/implementation work. No new website retention duration, storage cap or unread-data-loss policy is approved here.

Rationale/source: Erik clarified that reading on any device removes the incoming website copy, that cross-device persistence of saved content uses Automerge, and that removal after reading should always occur. This replaces the earlier indefinite website-copy retention option for acknowledged content while preserving acknowledgment-based cleanup.

Affected task: task-4675af07; acknowledgment scope also affects the tasks listed under P3. Required SlugKit/backend contracts remain implementation dependencies.

## P6 — Long-offline RSS cutoff

**Approved by Erik.** Retain a checkpoint/cutoff when retiring detailed RSS read-tracking records so entries before that cutoff do not become eligible again as a new unread backlog. Older skipped entries need not remain eligible. Suppressing those entries does not claim they were read and must not delete saved items or their durable read state.

Rationale/source: Erik proposed a last-read-date-like checkpoint to avoid replaying a large backlog, answered no when asked whether older unread entries should remain eligible, and confirmed that sufficiently old skipped items will likely never be read.

Affected task: task-3c923de0. The exact cutoff threshold, timestamp/identity representation, handling of unreliable publisher dates, rotation triggers, carry-forward and retired-document reconciliation are engineering choices. A date alone is not a selected deterministic deduplication algorithm. P3 must still leave concurrent/new unseen arrivals unacknowledged.

## P7 — Delivered conversations, no save-only remote subscription

**Approved by Erik.** Saving or liking does not subscribe to a remote thread or request additional remote content that was not delivered for the configured user. Process the website's incoming content delivered for that user, including normal followed-account delivery, on explicit Download. Preserve available comments on Save and retain subsequent delivered direct replies in participated conversations as already accepted.

Opening a saved conversation may automatically refresh its available conversation from content already delivered to the connected website, when online; offline it displays retained content. This is a targeted refresh of the selected conversation, not a general incoming-feed Download or remote thread crawl. Any such targeted refresh must use defined SlugKit/backend contracts; this decision does not assert they exist. Unrelated remote reply discovery, historical backfill and broad thread subscriptions remain outside scope. Do not promise every branch of a conversation.

Rationale/source: Erik first chose automatic loading of additional replies on opening a saved post, then clarified that he wants only content delivered to his inbox and does not want ROTT to seek other remote content. That clarification supersedes the broader fetch-on-open proposal. Server-to-server inbox delivery is separate from ROTT's retrieval from its website.

Affected tasks: task-7bc526ec, task-4675af07; task-db998439 and task-84b69694 consume the resulting reader behavior. Public-reply capture remains coordinated with task-c856395c. No new background fetching or notification alerts are approved.

## P8 — Learned source changes and sync boundary

**Approved by Erik.** Apply learned edits/deletions to unsaved cached posts locally; do not sync those changes or unsaved post bodies to other devices. Saved content and learned updates to its retained records use Automerge, including updated comment text and original-deletion markers while preserving saved content. Devices operate independently; any agreed cross-device state exchange uses Automerge, without a separate direct device coordination mechanism.

Rationale/source: Erik specified that saved content comes through Automerge and answered that unsaved copies and learned source changes should remain local. This expressly replaces the earlier requirement to sync learned changes for unsaved caches. The website remains authoritative for its publication/social state; the core consumes its defined contracts rather than inventing device coordination.

Affected task: task-7bc526ec; local cache-change handling affects task-5e49adae. Read-status metadata remains the accepted explicit Automerge exception; this decision does not restrict Automerge to saved post bodies alone.

## Remaining gates and independent work

P1–P8 now have approved product directions. No product default for these gaps remains guessed or deliberately deferred. These approvals remove their product-policy gates, not task dependencies, unimplemented SlugKit/backend contracts or engineering-validation requirements. Existing task descriptions that still label P1–P8 unresolved must be interpreted with this register; they are not evidence that a contract has shipped.

P9 detailed navigation/default-filter and Liked posts placement was not decided in this discussion and remains open. The affected UI behavior remains gated in task-db998439, task-2a2fbb5d and task-84b69694. This is an existing out-of-discussion gate, not an Erik-approved deferral of a listed reader-policy gap. Independent core/adapter scaffolding, mocks, local retention/read-state fixtures, contract design and platform build work remain available subject to each task's own prerequisites. These choices do not approve desktop layout for iPhone.

Cache technology, Apple bridge, Windows/Linux toolkit, deterministic identity fallback, tracking window/rotation/retirement protocol, checkpoint mechanics, API receipts and acknowledgment retry mechanics belong to their implementation tasks. The product register does not choose them or mark their validation complete.

Bluesky, broad thread subscriptions, notification alerts, audience controls, publishing media, extra messaging channels and other exploratory/deferred scope remain unchanged. ActivityPub and other social capabilities remain optional website capabilities. Private Save/notes do not publish. The legacy CLI/TUI and historical redesign sources are unchanged.
