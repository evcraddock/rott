# Implementation Plan: Rust Core, CLI, and TUI

## Overview

This plan covers the Rust implementation of ROTT v2, including:

- **rott-core** - Shared library with all business logic
- **rott-cli** - Command-line interface
- **rott-tui** - Terminal user interface

## Goals

1. Build a local-first knowledge management system
2. Support links and text notes with tagging
3. Work fully offline with optional sync
4. Provide both CLI and TUI interfaces sharing common logic

## Prerequisites

- Rust toolchain (stable)
- Understanding of Automerge library
- Familiarity with ratatui (for TUI)
- Familiarity with clap (for CLI)

## Dependencies on Other Plans

| Dependency | Required For | Blocking? |
|------------|--------------|-----------|
| Sync Server | Multi-device sync | No - local-only works without it |

---

## Phase 1: Project Setup and Core Data Model

### Objective

Establish the workspace structure and define the core data model using Automerge.

### Tasks

1. **Create Rust workspace**
   - Set up Cargo workspace with three crates
   - Configure shared dependencies
   - Set up development tooling (clippy, rustfmt, test configuration)

2. **Define data model**
   - Link structure (title, source, author, dates, description, tags)
   - Note structure (title, body, dates, tags)
   - Tag as a first-class concept

3. **Automerge document structure**
   - Design root document schema (contains all user's links and notes)
   - Define how links and notes are represented in Automerge
   - Handle schema versioning for future changes

4. **Basic CRUD operations in core**
   - Create link/note
   - Read link/note by ID
   - Update link/note fields
   - Delete link/note
   - List all links/notes
   - Filter by tag

### Deliverables

- Workspace compiles with three crates
- Core library can create, read, update, delete links and notes
- Data persists in Automerge documents
- Unit tests for all CRUD operations

### Success Criteria

- `cargo build` succeeds for all crates
- `cargo test` passes with >80% coverage on core
- Can round-trip a link and note through create/read cycle

---

## Phase 2: Local Storage

### Objective

Persist Automerge documents to local filesystem.

### Tasks

1. **Storage abstraction**
   - Define storage trait (interface) for persistence
   - Allow future implementations (filesystem, SQLite)

2. **Filesystem storage implementation**
   - Save Automerge documents to files
   - Load documents on startup
   - Handle document naming/organization
   - Atomic writes (write to temp, then rename)

3. **Root document management**
   - Generate root document ID on first run
   - Store root document ID in config
   - Load root document on startup

4. **Configuration**
   - Config file location (~/.config/rott/)
   - Data directory location (~/.local/share/rott/)
   - Support environment variable overrides

5. **Error handling**
   - Storage errors (disk full, permissions)
   - Corrupt document recovery
   - Missing data directory (auto-create)

### Deliverables

- Documents persist across application restarts
- Root document ID generated and stored
- Configuration file support

### Success Criteria

- Create document, close app, reopen app, document exists
- Root document ID persists across restarts
- Config file changes take effect

---

## Phase 3: CLI Implementation

### Objective

Build a command-line interface for all core operations.

### Tasks

1. **Command structure**
   - `rott link create <url> [--tag <tag>...]`
   - `rott link list [--tag <tag>]`
   - `rott link show <id>`
   - `rott link edit <id> [--title <title>] [--tag <tag>...]`
   - `rott link delete <id>`
   - `rott note create <title> [--tag <tag>...]`
   - `rott note list [--tag <tag>]`
   - `rott note show <id>`
   - `rott note edit <id>`
   - `rott note delete <id>`
   - `rott tag list`
   - `rott config show`
   - `rott config set <key> <value>`
   - `rott status` (show root doc ID, sync status)

2. **Output formatting**
   - Human-readable default output
   - JSON output option (--json flag)
   - Quiet mode for scripting (--quiet)

3. **Interactive editing**
   - Open $EDITOR for note body editing
   - Confirmation prompts for destructive actions

4. **URL metadata fetching**
   - Fetch title, description, author from URL
   - Timeout handling
   - Offline graceful degradation

### Deliverables

- Full CLI for all CRUD operations
- JSON output for scripting integration
- Man page or --help documentation

### Success Criteria

- All commands work as documented
- JSON output parses correctly
- Shell completion works (if implemented)

---

## Phase 4: TUI Implementation

### Objective

Build an interactive terminal interface using ratatui.

### Tasks

1. **Layout design**
   - Two-pane layout (tags/topics on left, items on right)
   - Status bar with keyboard shortcuts
   - Popup dialogs for editing

2. **Navigation**
   - Vim-style keybindings (h/j/k/l)
   - Arrow key support
   - Tab to switch panes
   - Search/filter within lists

3. **CRUD in TUI**
   - View link/note details
   - Create new link/note (popup form)
   - Edit existing item (popup or external editor)
   - Delete with confirmation
   - Edit tags inline

4. **Link-specific features**
   - Open link in browser (Enter)
   - Fetch/refresh metadata

5. **Visual feedback**
   - Loading indicators
   - Success/error messages
   - Highlight active pane

### Deliverables

- Interactive TUI with full CRUD support
- Keyboard-driven navigation
- Visual consistency with original ROTT

### Success Criteria

- Can perform all operations without leaving TUI
- Responsive to keyboard input
- No visual glitches or layout issues

---

## Phase 5: Sync Client

### Objective

Connect to sync server and synchronize documents.

### Tasks

1. **Sync server configuration**
   - Configure sync server URL in config
   - `rott config set sync.url ws://192.168.1.x:3030`
   - Enable/disable sync

2. **Sync server connection**
   - Establish WebSocket connection
   - Handle connection errors and reconnection
   - Graceful degradation when offline

3. **Automerge sync protocol**
   - Use automerge-repo sync protocol
   - Implement sync message exchange
   - Handle sync state persistence

4. **Root document sync**
   - Sync root document by ID
   - All data flows through root document

5. **Offline support**
   - Queue changes when offline
   - Sync when connection restored
   - Visual indicator of sync status in TUI

6. **CLI sync commands**
   - `rott sync` - trigger manual sync
   - `rott sync status` - show sync state
   - `rott device show` - show root document ID for sharing

### Deliverables

- Sync with remote server works
- Changes propagate between devices
- Offline changes sync when back online

### Success Criteria

- Create document on device A, appears on device B
- Edit on both devices offline, changes merge correctly
- Network interruption doesn't cause data loss

---

## Phase 6: Device Setup Flow

### Objective

Implement the flow for setting up new devices.

### Tasks

1. **First device setup**
   - Generate root document ID
   - Create initial root document structure
   - Display root document ID to user
   - Prompt user to save it somewhere

2. **New device setup**
   - `rott init` - first time setup
   - `rott init --join <root-doc-id>` - join existing
   - Prompt for root document ID if joining
   - Sync root document from server

3. **Device information**
   - `rott device show` - display root document ID
   - QR code generation (optional, for easy mobile setup)

4. **TUI integration**
   - First-run wizard
   - Show root document ID in settings
   - Copy to clipboard option

### Deliverables

- Complete device setup flow
- Easy sharing of root document ID
- Works for both first device and additional devices

### Success Criteria

- Can set up first device and see root doc ID
- Can set up second device by entering root doc ID
- Both devices sync correctly

---

## Phase 7: Migration and Polish

### Objective

Migrate from old ROTT format and polish the application.

### Tasks

1. **Migration from v1**
   - Import markdown files with YAML frontmatter
   - Map old fields to new data model
   - Preserve tags and metadata
   - `rott migrate <directory>` command

2. **Performance optimization**
   - Profile and optimize hot paths
   - Lazy loading for large document sets
   - Efficient tag indexing

3. **Error handling improvements**
   - User-friendly error messages
   - Suggested remediation steps
   - Logging for debugging

4. **Documentation**
   - README with installation and usage
   - Man pages for CLI
   - Architecture documentation updates

5. **Testing**
   - Integration tests for CLI
   - TUI testing (if feasible)
   - Sync integration tests (with mock server)

### Deliverables

- Migration tool for existing ROTT users
- Polished, documented application
- Comprehensive test suite

### Success Criteria

- Existing ROTT data migrates without loss
- No known critical bugs
- Documentation sufficient for new users

---

## Technical Decisions

### Automerge Document Structure

**Recommended approach:** Single root document per user containing all links and notes.

```
Root Document {
  links: Map<ID, Link>,
  notes: Map<ID, Note>,
  tags: Set<String>,  // optional: for quick tag listing
}
```

Rationale:
- Simple sync (one document to sync)
- User identity = root document ID
- Automerge handles concurrent edits within document

### Storage Format

- Automerge binary format for documents
- TOML for configuration
- Root document ID stored in config file

### Sync Protocol

- Use automerge-repo compatible WebSocket protocol
- Connect to standard automerge-repo-sync-server

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Automerge learning curve | Delays | Prototype early, document patterns |
| Large document performance | Slow UI | Profile early, consider document splitting |
| Network edge cases | Data issues | Extensive offline testing |

---

## Estimated Timeline

| Phase | Estimated Duration |
|-------|-------------------|
| Phase 1: Setup and Data Model | 1-2 weeks |
| Phase 2: Local Storage | 1 week |
| Phase 3: CLI | 1-2 weeks |
| Phase 4: TUI | 2-3 weeks |
| Phase 5: Sync Client | 1-2 weeks |
| Phase 6: Device Setup | 1 week |
| Phase 7: Migration and Polish | 1-2 weeks |

**Total: 8-13 weeks**

---

## Open Questions to Resolve

1. Automerge document structure - confirm single root doc approach
2. Root document ID format - raw UUID or encoded?
3. Sync server URL - how to discover/configure easily?
4. QR code for device setup - worth the dependency?
