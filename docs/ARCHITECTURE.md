# ROTT v2 Architecture Design

> Record of Tagged Topics - A local-first knowledge management system

## Overview

ROTT v2 is a complete redesign of the original ROTT application. It transforms from a simple file-based link manager into a multi-platform, local-first application with optional sync across devices.

### Design Principles

1. **Local-first** - Full functionality without any network connection
2. **Self-hosted** - Users run their own sync infrastructure
3. **Progressive complexity** - Start simple (local only), add sync when needed
4. **Platform native** - Each platform gets an idiomatic implementation

---

## Data Model

### Content Types

ROTT manages links with attached notes:

#### Links

Bookmarks with rich metadata:

- Title
- Source URL
- Author(s)
- Created date
- Updated date
- Description
- Tags
- Notes (annotations attached to the link)

#### Notes

Notes are **children of links**, serving as annotations or comments. They cannot exist independently.

- Title (optional)
- Body
- Created date

Notes do not have their own tags—they inherit context from their parent link.

### Ownership Model

- **Single owner** - Each user owns their documents exclusively
- **No sharing** - Documents cannot be shared between users
- **Multi-device** - A single user can access their documents from multiple devices
- **Multi-user** - Multiple users can use the same sync server (documents isolated by user)

---

## User Identity

### Root Document Model

Each user's identity is represented by a **root document** - a single Automerge document that acts as both identity and index.

```
User's Root Document (ID: abc123xyz...)
├── Links collection
├── Notes collection  
└── Settings/preferences
```

**The root document ID is the user's identity.** No server-side accounts needed.

### Multi-Device Flow

**First device setup:**

1. App generates a root document with a random ID
2. User's data is stored in (or referenced from) this document
3. Root document ID is stored locally

**Adding a new device:**

1. User enters root document ID on new device (QR code, manual entry, etc.)
2. New device syncs the root document from sync server
3. New device now has access to all user's data

```
First Device (laptop)              Second Device (phone)
        │                                   │
        │ Creates root doc "abc123"         │
        │ Stores ID locally                 │
        │                                   │
        ▼                                   │
   ┌─────────┐                              │
   │  Sync   │◄─────────────────────────────┤ User enters "abc123"
   │ Server  │                              │
   └─────────┘                              │
        │                                   │
        └──────────────────────────────────►│ Syncs root doc
                                            │ Now has all data
```

### Multiple Users on Same Server

Multiple users can share the same sync server:

- Each user has their own root document ID
- Users only sync documents they know about (their own)
- No server-side access control needed
- Isolation through document ID separation, not security

---

## Platform Architecture

### Interfaces

ROTT provides four interfaces:

| Interface | Technology | Use Case |
|-----------|------------|----------|
| CLI | Rust | Scripting, automation, power users |
| TUI | Rust | Interactive terminal experience |
| Web | TypeScript | Browser access from any device |
| iOS | Swift | Mobile access |

### Implementation Strategy

Each platform uses native implementations rather than shared cross-platform code:

| Platform | Language | Automerge Library |
|----------|----------|-------------------|
| CLI/TUI | Rust | automerge-rs |
| Web | TypeScript | automerge-js |
| iOS | Swift | automerge-swift |

**Rationale:**

- Idiomatic code per platform
- Leverage existing, well-tested Automerge libraries
- Avoid FFI/WASM complexity
- Easier debugging and maintenance

### Rust Workspace Structure

The CLI and TUI share a common core library:

```
rott/
├── crates/
│   ├── rott-core/           # Shared library
│   │   ├── config.rs        # Application configuration
│   │   ├── document.rs      # Automerge document handling
│   │   ├── document_id.rs   # Document ID (automerge-repo compatible)
│   │   ├── identity.rs      # User identity management
│   │   ├── models.rs        # Link, Note data structures
│   │   ├── store.rs         # Unified storage interface
│   │   ├── storage/         # Automerge persistence
│   │   └── sync/            # Sync server client
│   │
│   └── rott-cli/            # CLI and TUI binary
│       ├── commands/        # CLI command handlers
│       └── tui/             # Terminal UI (ratatui)
```

**Boundary:**

- Core contains all business logic, data handling, storage, and sync
- CLI crate contains both CLI commands and TUI presentation

---

## Data Synchronization

### Technology

**Automerge** - A Conflict-free Replicated Data Type (CRDT) library

Benefits:

- Automatic conflict resolution
- Offline-first by design
- Changes merge deterministically
- Sync protocol built-in

### Sync Server

Uses **automerge-repo-sync-server** or equivalent - a simple relay that stores and forwards Automerge documents.

**Characteristics:**

- Stores Automerge documents
- Relays sync messages between devices
- No knowledge of users or application semantics
- Generic - could serve any Automerge-based application

**Deployment:**

- Self-hosted on user's private network
- Accessible only from local network or via VPN (Tailscale, WireGuard)
- No authentication needed - network access is the security boundary

### Sync Behavior

Sync is **opportunistic**, not required:

| Scenario | Behavior |
|----------|----------|
| On private network | Auto-sync with server |
| Off network | Full offline functionality |
| Return to network | Sync catches up automatically |

This aligns with local-first principles - the app works fully offline, sync is a background convenience.

---

## Network Architecture

### Private Network Model

```
┌─────────────────────────────────────────────────────────────────┐
│  User's Private Network (Home, VPN, etc.)                      │
│                                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                        │
│  │   CLI   │  │   TUI   │  │   iOS   │                        │
│  └────┬────┘  └────┬────┘  └────┬────┘                        │
│       │            │            │                              │
│       └────────────┴────────────┘                              │
│                    │                                           │
│                    ▼                                           │
│             ┌────────────┐                                     │
│             │Sync Server │  (on NAS, Raspberry Pi, etc.)      │
│             └────────────┘                                     │
│                                                                │
└─────────────────────────────────────────────────────────────────┘
```

### Remote Access

When away from local network, users connect via VPN:

- **Tailscale** - Easy setup, works on all platforms including iOS
- **WireGuard** - Lightweight, fast
- **Other VPN** - Any solution that provides network access

App behavior when VPN not connected:

- Works fully offline (local-first)
- Syncs when VPN reconnected

---

## Web Architecture

### Challenge

The web app runs in a browser that may not be on the private network. It needs a way to reach the sync server.

### Solution: Web Server as Authenticated Relay

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Browser    │◄───────►│  Web Server  │◄───────►│ Sync Server  │
│  (anywhere)  │  HTTPS  │  (private    │  local  │  (private    │
│              │  + auth │   network)   │         │   network)   │
└──────────────┘         └──────────────┘         └──────────────┘
```

**How it works:**

1. Web server runs on private network (or has VPN access)
2. Web server is exposed to internet (with authentication)
3. User authenticates to web server
4. Web server relays Automerge sync messages to/from sync server
5. Sync server never exposed to internet

### Web Server Responsibilities

- Serve the web application (HTML/JS/CSS)
- Authenticate users
- Relay WebSocket traffic to sync server

### Web Authentication

Since web users access from the internet, authentication is required:

**Options (choose based on needs):**

| Method | Complexity | UX |
|--------|------------|-----|
| Password | Low | Enter each session |
| Passkey (WebAuthn) | Medium | Biometric/PIN |
| Magic link (email) | Medium | Click link in email |

**Flow:**

1. User authenticates to web server
2. User provides their root document ID (first time only, stored in browser)
3. Web server relays sync traffic for that user's documents
4. Browser stores data locally in IndexedDB (local-first)

### Web Data Storage

- **IndexedDB** - Automerge documents cached locally
- **LocalStorage** - Root document ID, preferences
- Browser works offline after initial sync, syncs when connected

---

## Component Dependencies

### Deployment Scenarios

| Use Case | Components Required |
|----------|---------------------|
| Single device, no sync | App only (no servers) |
| Multi-device native | App + Sync server (private network) |
| Web access | App + Sync server + Web server |

### Dependency Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         User                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
   ┌─────────┐            ┌─────────┐            ┌─────────┐
   │   CLI   │            │   TUI   │            │   iOS   │
   └────┬────┘            └────┬────┘            └────┬────┘
        │                      │                      │
        └──────────┬───────────┘                      │
                   │                                  │
                   ▼                                  │
            ┌─────────────┐                           │
            │  rott-core  │                           │
            │   (Rust)    │                           │
            └──────┬──────┘                           │
                   │                                  │
                   └──────────────┬──────────────────┘
                                  │
                                  │ Private network / VPN
                                  ▼
                         ┌───────────────┐
                         │  Sync Server  │ ◄── Optional (only for sync)
                         │  (Private)    │
                         └───────────────┘
                                  ▲
                                  │ Private network
                                  │
   ┌─────────┐         ┌──────────────┐
   │   Web   │◄───────►│  Web Server  │ ◄── Optional (only for web)
   │   App   │ Internet│  (Relay)     │
   └─────────┘  + Auth └──────────────┘
```

---

## Local Storage

### Storage Format

Each platform stores Automerge documents locally:

| Platform | Storage Mechanism |
|----------|-------------------|
| CLI/TUI | Filesystem (Automerge binary) |
| Web | IndexedDB |
| iOS | Filesystem or local database |

### Data Location

| Platform | Default Location |
|----------|------------------|
| CLI/TUI (Linux) | `~/.local/share/rott/` |
| CLI/TUI (macOS) | `~/Library/Application Support/rott/` |
| Web | Browser IndexedDB |
| iOS | App sandbox |

---

## Migration Path

### From Current ROTT

The current ROTT application uses markdown files with YAML frontmatter. Migration considerations:

1. **Import tool** - Convert existing markdown files to Automerge documents
2. **Preserve metadata** - Map frontmatter fields to new data model
3. **One-time migration** - Run once, then use new system exclusively

```bash
rott migrate /path/to/markdown/files
```

### Backward Compatibility

The v2 architecture is a clean break. The old and new systems are not compatible. Users must migrate their data.

---

## Security Model

### Trust Boundaries

| Component | Trust Level | Access |
|-----------|-------------|--------|
| User's devices | Fully trusted | Full access to data |
| Sync server | Trusted | Stores data, on private network |
| Web server | Trusted | Relays data, authenticates web users |
| Private network | Trusted | Access boundary for sync |
| Internet | Untrusted | Only web server exposed (with auth) |

### Threat Model

**Protected against:**

- Random internet attackers (sync server not exposed)
- Unauthorized web access (authentication required)

**Not protected against:**

- Attackers on your private network
- Compromised devices
- Physical access to sync server

**Assumption:** The private network is trusted. This is appropriate for personal/home use.

### Data Privacy

- Data is **not encrypted** at rest or in transit within the private network
- TLS should be used for web server connections from internet
- For higher security needs, consider:
  - VPN for all access (no web server exposure)
  - Full-disk encryption on sync server
  - Future: Add encryption layer to Automerge documents

---

## Future Considerations

Items explicitly deferred from v2:

1. **End-to-end encryption** - Encrypt documents client-side before sync
2. **Document sharing** - Share specific documents with other users
3. **Collaborative editing** - Multiple users editing same document
4. **File attachments** - Currently text-only, binary support could be added
5. **Offline web (PWA)** - Service worker for full offline web support
6. **Public sync servers** - Multi-tenant hosted sync with user isolation

---

## Open Questions

Items to resolve during detailed planning:

1. ~~**Automerge document structure**~~ - **Resolved:** Single root document per user
2. ~~**Root document ID format**~~ - **Resolved:** Base58-encoded with checksum (bs58)
3. **Web authentication method** - Password, passkey, magic link?
4. **Web server stack** - Language, framework choices
5. **iOS specifics** - SwiftUI vs UIKit, data storage approach
6. **Sync conflict UX** - How to surface merge information to users (if at all)
7. ~~**Device linking UX**~~ - **Resolved:** Manual entry via `rott init --join <id>`

---

## Glossary

| Term | Definition |
|------|------------|
| Automerge | A CRDT library for building local-first applications |
| CRDT | Conflict-free Replicated Data Type - data structures that merge automatically |
| FFI | Foreign Function Interface - calling code across language boundaries |
| IndexedDB | Browser-based database for persistent storage |
| Local-first | Applications that work offline, treating the server as optional |
| Magic Link | Authentication via email link, no password required |
| Passkey | WebAuthn credential using biometrics or device PIN |
| Root Document | The user's primary Automerge document that acts as identity and index |
| Sync Server | Server that relays Automerge documents between devices |
| Tailscale | VPN service that creates a private network across devices |
| WASM | WebAssembly - binary format for running code in browsers |
| WireGuard | Fast, modern VPN protocol |
