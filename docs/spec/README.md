# ROTT technical specification

Status: design specification, September 25, 2026. This set specifies the new application; it does not indicate implementation completion or authorize code changes, API mutations, task creation, or repository migration.

ROTT combines a feed/account reader, a private saved collection, and durable personal communications. Users discover accounts, read locally available posts, download new website content explicitly, save links with tags and notes, and participate in supported social conversations. The product retains the ROTT name and local ownership of personal data.

## Architecture and scope

One reusable Rust core supplies domain operations, website integration, feed processing, local Automerge persistence, and optional sync. Native Swift/SwiftUI applications target macOS and iPhone first; Windows and Linux are required afterward. There is no selected ordering within either pair or release date. A single device needs no sync server or hosted frontend.

SlugKit owns the shared website/API contract. **All new API additions and extensions must be implemented in SlugKit.** Website implementations supply optional federation and backend behavior; ROTT consumes semantic operations through Rust. ActivityPub is not mandatory for a SlugKit implementation.

| Specification | Contents |
| --- | --- |
| [Architecture and platforms](architecture.md) | Core boundaries, platform adapters, targets, coexistence |
| [Site and API integration](site-integration.md) | Contract ownership, connection, credentials, identity, read-only doctor |
| [Accounts and follows](accounts.md) | Catalogs, discovery, refresh, subscriptions, authoritative follow state |
| [Reader and communications](reader.md) | Download flow, reading, link discovery, publishing actions, thread following |
| [Data, retention, and sync](data-and-sync.md) | Automerge ownership, cache/read state, media, document lifecycle, sync |
| [Implementation and deferred work](implementation.md) | Concrete API follow-up, engineering decisions, product details, exclusions |

## Interpretation and sources

“Must” and “must not” express accepted requirements. A section labeled proposed, implementation detail, deferred, or uncommitted does not establish a selected implementation. Required behavior whose API is missing remains a requirement, not an assertion that the deployed website supports it.

This is a separate technical presentation derived from the preserved [design document](../redesign/ROTT-DESIGN.md) and [reader overview](../redesign/ROTT-READER-OVERVIEW.md). Neither source is replaced or shortened by this set. Current verified methods/routes and source/deployment limits are centralized in the [API gap review](../redesign/ROTT-API-GAP-REVIEW.md); the specifications describe intended behavior.

The [Fediverse research conclusion](../redesign/ROTT-FEDIVERSE-RESEARCH.md) summarizes the original link-discovery task, retrieval alternatives, recommendation, and operational tradeoffs. The broader application requirements above go beyond that research scope.

Detailed API design proceeds per feature using that review. No full upfront endpoint design is required. Open API work is retained as the checklist in [implementation](implementation.md), without carrying resolved-question bookkeeping into these specifications.
