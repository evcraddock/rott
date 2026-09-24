# ROTT: Unified Reader and Saved-Link Collection

Date: September 23, 2026  
Status: Working design draft. No redesign implementation has started.

This repository document incorporates the discussion draft from `~/Vault/01-Projects/rott/ROTT-DESIGN.md` and the subsequent ActivityPub client brainstorming for task `task-d922c229`.

## Purpose

The new ROTT will combine a reader for multiple kinds of feeds and accounts with a personal collection of saved links. Users discover accounts through Slug-compatible websites, choose which accounts to subscribe to, browse incoming items, and save selected links with tags and notes.

The application will retain the ROTT name. The direction discussed is a fresh TypeScript application, rather than a requirement to reproduce or directly port the existing Rust application. Its interface and architecture can differ substantially from the current version.

## Working decisions

- The new ROTT is a client for a configurable ActivityPub actor hosted by a Slug-compatible website, such as `@erik@erikvancraddock.com`.
- The configured actor is the user's federated identity and owns the accounts followed for ROTT.
- ROTT should connect to the Slug-compatible website in a manner similar to the existing `slug` CLI.
- The Slug server keeps the actor's ActivityPub signing keys, performs federation, receives inbox deliveries, and exposes authenticated client APIs. ROTT does not store the actor's private key or operate its own public ActivityPub inbox.
- Slug websites provide account catalogs, while ROTT owns the user's subscription choices.
- Fetched posts and feed content are disposable local cache data.
- Subscription choices, read state, saved links, tags, and notes are durable personal state synchronized with Automerge.
- A discovered link is not automatically saved. The user reviews discovered links before adding them to the durable collection.
- This is currently a research and design effort. It does not authorize implementation or following accounts yet.

## Existing projects and context

### Current ROTT

[evcraddock/rott](https://github.com/evcraddock/rott), "Record of Tagged Topics," is a Rust application used daily. Its core models and storage code already support saved links, metadata, tags, and attached notes. It provides a CLI and terminal interface and uses Automerge for local storage, with a sync server for exchanging changes between devices.

The essential behavior to preserve is local-first ownership of the saved collection: links can be saved and organized offline, remain available on the machine, and sync to other machines after connectivity returns.

### Slugkit and Slug-compatible websites

Slugkit is written in TypeScript and runs on Node.js. Its source, contact, and account models provide context for discovering accounts. Accounts can be associated with sources or contacts and include information such as URL, kind, and protocol.

The existing website at [erikvancraddock.com](https://erikvancraddock.com) is a concrete catalog provider. During the original discussion, the command below passed all reported checks:

```sh
slug --site erikvancraddock.com doctor
```

The checks covered configuration, API reachability, API metadata, package metadata, OpenAPI, and authentication. The site reported API version 1.0.0 and OpenAPI 3.1.0. Its [API schema](https://erikvancraddock.com/api/v1/openapi.json) includes sources, contacts, and accounts. This verifies the CLI's compatibility checks; it is not an exhaustive test of every API operation.

The current `slug` CLI also exposes commands for listing, following, and unfollowing ActivityPub accounts. Its login flow opens the Slug site's `/cli/auth` page, asks the user to paste the generated API key, verifies the key through `/auth/check`, stores the API base URL and key, and sends the key as a bearer token on later requests.

## Intended user experience

1. Configure ROTT with a Slug-compatible website and select an ActivityPub actor hosted by that website.
2. Authenticate ROTT to act as that actor through the website's client API.
3. Browse the website's catalog of sources, contacts, and accounts.
4. Choose specific accounts for the configured actor to follow.
5. Retrieve incoming posts for followed accounts and browse them in a reader.
6. Track which items have been read.
7. Extract external links from incoming posts and place them in a review workflow.
8. Save selected links for reading later or another purpose, assigning tags and optionally notes.
9. Use the saved collection offline and sync durable personal state between machines when connected.

RSS and ActivityPub accounts are intended inputs. Bluesky is a possible additional input. Other kinds of accounts may be supported where access is available. Facebook, X, and Instagram are outside the intended initial scope; the discussion did not establish an integration approach for them.

## ActivityPub client model

ROTT is not intended to become an independently hosted ActivityPub server. It is a client for an actor already hosted by a Slug-compatible website.

A representative configuration would identify:

- the Slug site's API base URL;
- the selected local actor, such as `@erik@erikvancraddock.com`;
- an authenticated client credential or token reference;
- local cache and Automerge document locations.

The Slug site is responsible for:

- resolving and following remote actors;
- signing ActivityPub requests with the configured actor's private key;
- receiving and validating inbox deliveries;
- maintaining server-side federation state;
- exposing authenticated client operations for follows and incoming posts.

ROTT is responsible for:

- site and actor configuration;
- subscription intent and reader preferences;
- retrieving incoming items through the client API;
- local caching and read-state presentation;
- extracting external links;
- reviewing and saving selected links;
- synchronizing durable personal state through Automerge.

ActivityPub standardizes server-to-server federation but does not define a complete general-purpose client API. The Slug API must therefore define the client-facing contract ROTT needs. Existing follow-management endpoints are useful groundwork, but timeline or inbox-reading capabilities still need to be verified or designed.

## Authentication and connection

The initial connection model should remain compatible with the existing `slug` CLI unless research identifies a reason to change it. Today that means browser-assisted creation of an API key followed by bearer-token API access.

A future OAuth 2.1 Authorization Code flow with PKCE may provide a stronger application-oriented login experience, scoped access, token rotation, and revocation. That is a suggestion rather than a current decision. Any authentication redesign should be implemented at the shared Slug client boundary so both ROTT and the CLI can use it.

Secrets must remain device-local and should be stored in the operating system credential store. Credentials must not be stored in Automerge or synchronized as ordinary application state.

## Shared Slug client library

There is not currently a supported reusable npm package for the complete Slug client connection flow.

Current packages include:

- `@evcraddock/slug-cli`, which implements connection and API-key authentication internally but exposes only the `slug` executable;
- `@evcraddock/slug-core`, which provides shared types, constants, and helpers rather than an HTTP client;
- `@evcraddock/slug-auth`, `@evcraddock/slug-api`, and `@evcraddock/slug-federation`, which are primarily server-side building blocks.

The proposed direction is to extract the reusable client behavior into a package such as `@evcraddock/slug-client`. Both the `slug` CLI and the new ROTT application could consume it.

A shared client package could own:

- Slug site discovery and API URL normalization;
- API metadata and compatibility checks;
- authentication and credential abstractions;
- typed HTTP requests and consistent error handling;
- actor selection;
- catalog operations;
- following and unfollowing operations;
- incoming-item or timeline operations once their API contract exists.

Whether creating this package is a hard prerequisite for ROTT remains open, but avoiding a second independent implementation of Slug connection and authentication is preferred.

## Catalogs and subscriptions

The Slug website supplies the master catalog of accounts the user might want to follow. It does not determine which accounts ROTT actually follows.

ROTT owns the user's subscription choices. A source might list several accounts, and the user can choose a subset of those accounts. Merely appearing in the website's catalog must not create a subscription.

Subscription choices will be stored in Automerge and sync between the user's machines. The design should accommodate catalogs from multiple Slug-compatible websites, although the initial scope and details of connecting them remain open.

The catalog remains authoritative for the account information it publishes; ROTT remains authoritative for the user's decision to subscribe. How ROTT responds to catalog edits, deleted accounts, or an unavailable catalog needs further design.

The relationship between local subscription intent and the configured ActivityPub actor's server-side following collection needs an explicit reconciliation model. ROTT may need to distinguish follows it manages from follows created by another client using the same actor.

## Data ownership and storage

The agreed distinction is between disposable fetched content and durable personal state.

| Data | Intended storage | Behavior across machines |
| --- | --- | --- |
| Source, contact, and account catalog | Slug website; local caching details undecided | Retrieved from the configured catalog provider |
| Selected subscriptions | Automerge | Syncs between machines |
| Configured Slug site and actor identity | Automerge or non-secret configuration; exact split undecided | May sync if it contains no credentials |
| Authentication credentials | OS credential store | Device-local and never stored in Automerge |
| Fetched posts and feed content | Local disposable cache | Each machine may have a different cache |
| Feed-fetch bookkeeping | Local cache or local database | Belongs to that machine's fetching process |
| Per-item read/unread state | Automerge | Syncs independently of cached content |
| Explicitly saved links, tags, and notes | Automerge | Preserved locally and synced between machines |

An item appearing in a feed does not automatically become a saved link. Saving it creates a durable entry in the personal collection. Clearing or expiring feed content must not remove saved links or their annotations.

"Only saved links are preserved" refers to preserving the link collection and content records: Automerge also holds subscription choices and lightweight reading state for unsaved items.

The exact database or cache technology is not yet chosen. SQLite was discussed as the kind of local storage a reader such as Newsboat uses, not as a finalized implementation requirement.

## Incoming items and link discovery

The initial conceptual pipeline is:

1. The configured actor follows a remote ActivityPub account.
2. The Slug server receives activities for that actor and makes relevant posts available through its authenticated client API.
3. ROTT retrieves new or changed items and stores them in a local disposable cache.
4. ROTT extracts external links from each post while retaining the source actor and post identity.
5. ROTT compares candidate links with the local review queue and saved-link collection.
6. The user reviews candidates, optionally edits metadata or assigns tags and notes, and explicitly saves selected links.

Research still needs to define:

- which ActivityPub object and activity types count as posts;
- whether boosts, replies, quote posts, and pinned posts are included;
- which links are considered external rather than actor, hashtag, mention, attachment, or navigation links;
- URL normalization and canonicalization rules;
- whether multiple posts sharing one URL become one candidate with multiple sources;
- how edits update pending candidates;
- how deleted posts affect unread items, reviewed candidates, and already saved links;
- how pagination, retention, backfill, and rate limits work in the Slug API;
- whether the server supplies an aggregated timeline or ROTT combines account streams itself.

## Feed fetching and reading state

Each machine may fetch subscribed accounts independently. Its cache can therefore differ from another machine's cache. A machine that was offline may miss older items that are no longer available from the upstream source or Slug server.

Readers distinguish several kinds of state:

- Fetch metadata, such as cursors, ETags, and Last-Modified values when supported, helps avoid downloading unchanged content.
- Item identifiers allow repeated fetches to recognize an existing item rather than create duplicates.
- Read/unread state records the user's interaction with an item, independently of whether it was fetched.

The new ROTT will keep fetched content locally while syncing lightweight read/unread state through Automerge. If another machine fetches the same item, it can use the synced state to display it as already read without receiving that item's content through Automerge.

This requires a consistent identity for each item across machines. The exact identity scheme remains open and will need to account for ActivityPub object IDs, account identity, protocol differences, and missing or unreliable upstream IDs. An item's identity is not necessarily the same thing as the URL of a link shared in that item.

A shared feed-fetching service was discussed as an alternative for identical histories across machines. It is not part of the current agreed direction: independent, disposable local caches are acceptable. The Slug server may nevertheless retain enough inbox history to act as the common upstream source for each ROTT client.

## Local-first behavior and sync

The application should support browsing locally cached items and accessing, saving, tagging, and annotating locally available links without connectivity. Fetching new remote content and exchanging changes with other machines require connectivity.

Automerge is the intended mechanism for durable personal state. Moving the application to TypeScript does not itself require replacing Automerge. Its [TypeScript tutorial](https://automerge.org/docs/tutorial/) and [repository documentation](https://automerge.org/docs/reference/repositories/) describe local storage and network synchronization support.

Reusing the existing ROTT sync server remains a question to verify. Using Automerge in both applications does not by itself establish compatibility with the deployed server, its transport, or its document layout.

Saved links currently mean saved URL records and their metadata, tags, and notes. Downloading full article bodies or assets for permanent offline reading has not been decided.

## Coexistence with the Rust version

The existing Rust ROTT is used daily and must remain usable while the new application is explored.

The proposed separation is:

- A distinct development command, such as `rott-next`, while retaining ROTT as the product name.
- Separate configuration and data directories.
- Separate Automerge documents and sync identity for the new application during development.

These are isolation choices, not finalized names or paths. The new application should not write into the current Rust application's data or assume it can change its schema. Any future import or migration of the existing saved collection will be an explicit design decision.

## Potential reuse

The Rust ROTT provides existing concepts for saved links, tags, notes, URL duplicate detection, and local-first synchronization. Its core data model is useful design input even if the new application is substantially different.

Slugkit provides the existing catalog API, source/contact/account concepts, API-key login flow, and ActivityPub follow management. Publishing a federated website and operating a reader have different requirements, so incoming-item client APIs still need focused design.

The CLI compatibility check confirms the example site's API is discoverable and usable by the configured Slug CLI. It does not settle whether ROTT should call that API directly or use the proposed shared client library. Invoking the CLI as a subprocess is not the preferred application architecture.

## Approaches to evaluate during research

Even with the ActivityPub client direction established, the research should compare where retrieval and aggregation occur:

1. **Slug-hosted actor timeline API:** The Slug server receives federation deliveries and exposes one authenticated, paginated timeline for the configured actor. This best matches the client model but requires a new stable API and server retention policy.
2. **Per-account data exposed by the Slug server:** ROTT asks the server for posts associated with each followed actor and combines them locally. This allows local control but complicates pagination, ordering, and rate limiting.
3. **Direct public polling as fallback:** ROTT polls public outboxes, platform APIs, or feeds when server timeline support is unavailable. This may improve compatibility but weakens the configured-actor model, often loses private or followers-only content, and adds protocol-specific behavior to the client.

A true ROTT-hosted ActivityPub actor and inbox is not the current direction because it would require ROTT to operate a publicly reachable federated service and hold signing keys.

## Open design questions

- Is `@evcraddock/slug-client` a prerequisite for ROTT, and which repository owns it?
- What exact API identifies and selects the local ActivityPub actor on a multi-actor Slug site?
- Does the current API key represent the whole site administrator or a specific actor, and what narrower scopes will ROTT require?
- Should the Slug server expose an aggregated actor timeline, per-followed-account streams, or both?
- How far back can a new ROTT installation retrieve inbox history?
- How are catalog connections configured and authenticated, and which non-secret connection details sync?
- What account information is cached or copied into a subscription so it remains useful offline?
- How are account edits, deletions, and duplicates across catalogs handled?
- How does ROTT reconcile Automerge subscription intent with follows changed by other clients?
- What stable keys identify subscriptions, incoming items, candidate links, and saved links?
- How should conflicting read/unread changes from offline devices resolve?
- How long should cached content and synced reading-state records be retained?
- How should multiple items that share the same link relate to a single saved entry?
- How should edits and deletions propagate to cached items and pending candidates?
- Can the current sync server support the new application, and how should documents be organized?
- Is full article archiving needed in addition to saved link metadata?
- Should existing Rust ROTT data eventually be imported, and what compatibility guarantees would be needed?

These questions are intentionally left open. The discussion establishes product behavior and data boundaries without authorizing implementation or committing to a detailed architecture.

## Potential implementation follow-ups

These are candidate follow-up tasks, not implementation authorized by this design task:

1. Specify the Slug client API for actor selection, timeline retrieval, pagination, edits, deletions, and retention.
2. Decide whether to extract and publish `@evcraddock/slug-client`, then refactor `slug` to use it.
3. Define subscription, incoming-item, candidate-link, and saved-link identity schemes.
4. Define Automerge schemas for subscriptions and read state without storing fetched post bodies.
5. Design the local cache and fetch bookkeeping model.
6. Prototype ActivityPub post and external-link extraction against representative Slug-hosted actors.
7. Design the review queue and save-to-collection workflow.
8. Verify whether the existing sync server can support the redesign's Automerge documents.
9. Define migration and coexistence boundaries for the Rust application.
