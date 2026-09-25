# Site and API integration

[Specification index](README.md)

## Ownership and contract

SlugKit is the protocol-independent website/content API contract used by the Slug CLI and ROTT. The existing schema is the source of truth. A compatible website need not host an ActivityPub actor or implement social features.

**API additions and extensions belong in the SlugKit project.** SlugKit must define compatible optional semantic operations and schemas. Website backends implement the corresponding storage and, where enabled, federation/signing/delivery. ROTT's Rust core consumes the contract and reconciles results. Language-specific server packages or the CLI's implementation are not required client dependencies.

Social requests identify the relevant account and item without carrying ActivityPub wire messages or signing details. Reuse the existing follow contract where applicable. No separate API namespace, endpoint names, action field names, or route renames are selected. A site-owned post slug, an external post identity and a following-relationship ID are distinct concepts.

The API vocabulary must use **Share** for the semantic action and **Shares** for its records. An ActivityPub backend maps a plain reshare to Announce. Plain Share, quote, reply, and link publication remain distinct operations.

Current schema/code coverage and deployed evidence are in the [API gap review](../redesign/ROTT-API-GAP-REVIEW.md). Missing required behavior must be added per feature; the presence of an actor or an API key does not supply missing operations. Existing Follow/Undo and Accept/Reject code requires integration validation, not an assumed replacement contract.

## Connection and identity

On Mac, connect with a website address and API key. Initially support one connected website/catalog and one account configuration. Explicitly distinguish website/API identity, the configured social actor where applicable, and catalog accounts. Preserve the possibility of future configuration switching; multi-account UI and combining catalogs are outside initial scope.

Non-secret website address and account name/identity must be stored in Automerge for optional sync. Each device authenticates separately; keys need not be identical. Credentials must remain in that device's platform secure credential store and must not enter Automerge. No platform credential-cloud sync is selected.

Extend the existing `GET /auth/check` contract with account identity and an optional ActivityPub actor address. Do not add a separate `whoami` route. Exact response fields and handling of multiple accounts or site-wide keys are SlugKit implementation work. The extension is planned, not deployed functionality established by this specification.

The initial full-access API-key policy is accepted based on user confirmation. Narrower permissions are deferred and are not an initial blocker. Maintain compatibility with the CLI's API-key/bearer contract. A future OAuth flow, including Authorization Code with PKCE, is an unselected possibility; no new OAuth/browser flow or authentication scheme is required now.

## Rust client responsibilities

The core must provide platform-independent site discovery/API URL normalization, metadata and compatibility checks, credential abstractions, typed HTTP operations and errors, identity handling, catalogs, follow management, and incoming/action operations as their contracts become available. Platform adapters supply OS-specific credential and browser interactions where needed.

Keep Rust types and requests/responses aligned with the shared schema and validate authentication compatibility. No generator, exact module organization, or signatures are selected. The CLI may evolve independently against the same contract.

## Read-only doctor

ROTT must provide a doctor diagnostic that checks:

- API compatibility and supported operations against the shared contract and explicit website capability reporting.
- Credentials and access to the configured account.
- Configured ActivityPub actor-profile reachability where applicable.

The website must report supported capabilities such as incoming reading, follow, Like and Share. An actor URL alone is insufficient. Unsupported capabilities must be clearly reported without treating the whole application as unusable or requiring every site to expose an actor.

Doctor must not create test posts, follows, likes, or other mutations. Health or metadata endpoints are candidate report locations only. Exact location, fields, versioning, schema mapping and unsupported/error responses remain SlugKit contract work listed in [implementation](implementation.md).
