# ROTT v2 Implementation Plans

This directory contains detailed implementation plans for each component of ROTT v2.

## Plan Overview

| Plan | Technology | Purpose |
|------|------------|---------|
| [Rust CLI/TUI](RUST_CLI_TUI.md) | Rust | Core library + command-line and terminal interfaces |
| [Sync Server](SYNC_SERVER.md) | Docker/Node.js | Automerge document relay (uses existing software) |
| [Web Server](WEB_SERVER.md) | Node.js/Go | Authentication and WebSocket relay for web access |
| [Web App](WEB_APP.md) | TypeScript | Browser-based interface |
| [iOS App](IOS_APP.md) | Swift | Native iOS application |

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│  User's Private Network                                        │
│                                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                        │
│  │   CLI   │  │   TUI   │  │   iOS   │                        │
│  └────┬────┘  └────┬────┘  └────┬────┘                        │
│       │            │            │                              │
│       └────────────┴────────────┘                              │
│                    │                                           │
│                    ▼                                           │
│             ┌────────────┐                                     │
│             │Sync Server │  (automerge-repo-sync-server)      │
│             └────────────┘                                     │
│                    ▲                                           │
│                    │                                           │
│             ┌────────────┐                                     │
│             │ Web Server │  (auth + relay)                    │
│             └────────────┘                                     │
│                    ▲                                           │
└────────────────────┼───────────────────────────────────────────┘
                     │ HTTPS (internet)
              ┌──────┴──────┐
              │   Web App   │  (browser, anywhere)
              └─────────────┘
```

## Key Simplifications

This architecture is intentionally simple:

| Aspect | Approach |
|--------|----------|
| **Encryption** | None - trust the private network |
| **User Identity** | Root document ID (no accounts) |
| **Sync Server** | Use existing automerge-repo-sync-server |
| **Network Security** | Private network + VPN for remote access |
| **Web Auth** | Simple password or passkey (web server only) |

## Recommended Implementation Order

### Phase A: Local-First Foundation

**Goal:** Working local-only application

```
RUST_CLI_TUI.md - Phases 1-4
(Setup, Storage, CLI, TUI)
```

**Duration:** ~5-8 weeks

**Deliverables:**
- Rust core library with Automerge
- Working CLI application
- Working TUI application
- Local storage (no sync yet)

---

### Phase B: Sync Infrastructure

**Goal:** Multi-device sync on private network

```
SYNC_SERVER.md - All phases (mostly deployment)
        +
RUST_CLI_TUI.md - Phases 5-6
(Sync Client, Device Setup)
```

**Duration:** ~2-3 weeks (sync server is existing software)

**Deliverables:**
- Sync server running on private network
- CLI/TUI can sync between devices
- Root document ID sharing for device setup

---

### Phase C: iOS Application

**Goal:** Native mobile access

```
IOS_APP.md - All phases
```

**Duration:** ~13-14 weeks

**Deliverables:**
- Native iOS application
- Syncs with other devices on network
- App Store ready

---

### Phase D: Web Access

**Goal:** Browser access from anywhere

```
WEB_SERVER.md - All phases
        +
WEB_APP.md - All phases
```

**Duration:** ~12-17 weeks

**Deliverables:**
- Web server with authentication
- Web application
- Access from any browser with internet

---

### Phase E: Migration and Polish

**Goal:** Production readiness

```
RUST_CLI_TUI.md - Phase 7
(Migration from v1)
```

**Duration:** ~1-2 weeks

**Deliverables:**
- Migration tool for existing ROTT users
- Documentation
- Polish

---

## Parallel Development Options

If multiple developers are available:

```
Timeline ──────────────────────────────────────────────────►

Developer 1:
├── Phase A (Rust) ──────────┼── Phase B (Sync) ──┤

Developer 2:
                                   ├── Phase C (iOS) ─────────────────┤

Developer 3:
                                   ├── Phase D (Web) ─────────────────┤
```

---

## Total Estimated Timeline

| Scenario | Duration |
|----------|----------|
| Single developer, sequential | 8-12 months |
| Two developers, some parallel | 5-8 months |
| Full team, maximum parallel | 3-4 months |

---

## Minimum Viable Product (MVP)

**MVP 1: Local-only CLI/TUI**
- RUST_CLI_TUI.md Phases 1-4
- ~5-8 weeks
- No server required

**MVP 2: Add multi-device sync**
- + SYNC_SERVER.md (deployment)
- + RUST_CLI_TUI.md Phases 5-6
- ~2-3 additional weeks

**MVP 3: Add iOS**
- + IOS_APP.md all phases
- ~13-14 additional weeks

**MVP 4: Add web**
- + WEB_SERVER.md + WEB_APP.md
- ~12-17 additional weeks

---

## Dependencies Between Plans

```
                    ┌─────────────┐
                    │ Rust Core   │
                    │ (Phases 1-4)│
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
        ┌──────────┐             ┌──────────┐
        │   CLI    │             │   TUI    │
        └──────────┘             └──────────┘
              │
              │ needs for sync
              ▼
       ┌─────────────┐
       │ Sync Server │ (existing software, just deploy)
       └──────┬──────┘
              │
    ┌─────────┴─────────┐
    │                   │
    ▼                   ▼
┌──────────┐      ┌──────────┐
│ iOS App  │      │Web Server│
└──────────┘      └────┬─────┘
                       │
                       ▼
                  ┌──────────┐
                  │ Web App  │
                  └──────────┘
```

---

## Getting Started

1. Read [ARCHITECTURE.md](../ARCHITECTURE.md) for overall design
2. Start with [RUST_CLI_TUI.md](RUST_CLI_TUI.md) Phase 1
3. Deploy sync server when ready for multi-device
4. Build iOS and/or Web based on priorities
