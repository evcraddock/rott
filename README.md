# ROTT

**R**ecord **O**f **T**agged **T**opics — A local-first links and notes manager with real-time sync.

## Overview

ROTT helps you save, organize, and access links with tags and notes. It features a terminal UI for browsing your collection and a CLI for scripting and automation.

**Key features:**

- **Local-first**: Your data is stored locally using Automerge CRDTs
- **Real-time sync**: Optional sync across devices via WebSocket
- **TUI interface**: Three-pane layout with filters, items, and detail view
- **CLI commands**: Full CLI for scripting and automation
- **Metadata scraping**: Automatically fetches title, description, and author from URLs
- **Notes**: Attach notes to any link

## Installation

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/evcraddock/rott/main/install.sh | bash
```

### Download Binary

Pre-built binaries are available on the [releases page](https://github.com/evcraddock/rott/releases):

- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

### Build from Source

```bash
git clone https://github.com/evcraddock/rott.git
cd rott
cargo install --path crates/rott-cli
```

## Usage

### TUI Interface

Launch the TUI by running `rott` with no arguments:

```bash
rott
```

The interface has three panes:

| Pane | Description |
|------|-------------|
| **Filters** | Browse by All, Favorites, Untagged, or specific tags |
| **Items** | List of links matching the selected filter |
| **Detail** | Full details of the selected link including notes |

#### Keyboard Shortcuts

**Navigation:**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Move to left pane |
| `l` / `→` | Move to right pane |
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |

**Actions:**

| Key | Action |
|-----|--------|
| `Enter` | Open link in browser |
| `Space` | Toggle favorite tag |
| `a` | Add new link |
| `e` | Edit selected link |
| `t` | Edit tags |
| `n` | Add note to link |
| `d` | Delete selected item |
| `u` | Copy URL to clipboard |
| `/` | Search |
| `:` | Command mode |
| `?` | Show help |
| `Ctrl+s` | Force sync |
| `q` | Quit |

### First-Time Setup

On first run, ROTT will prompt you to set up your identity:

```bash
$ rott init

Welcome to ROTT!

No existing identity found. Is this your first device?

  [1] Yes, create new identity
  [2] No, I have an existing root document ID

>
```

Your root document ID is your identity for syncing across devices. You can view it anytime:

```bash
$ rott device show

Root document ID: 3PkFS4K4KKTeCm2iiN9XVxHRRFdN
Automerge URL:    automerge:3PkFS4K4KKTeCm2iiN9XVxHRRFdN

Use this ID to set up ROTT on another device:
  rott init --join 3PkFS4K4KKTeCm2iiN9XVxHRRFdN
```

**Non-interactive setup (for scripting):**

```bash
# Create new identity
rott init --new

# Join existing identity
rott init --join <root-document-id>
```

### CLI Commands

```bash
# Add a link
rott link create https://example.com --tag rust --tag programming

# List all links
rott link list

# List links by tag
rott link list --tag rust

# Show link details
rott link show <id>

# Search links
rott link search "search query"

# Edit a link (opens in $EDITOR)
rott link edit <id>

# Delete a link
rott link delete <id>

# Add a note to a link
rott link note add <link-id> "Note content"

# List all tags
rott tags

# Show sync status
rott status

# Force sync
rott sync

# Show configuration
rott config show
```

## Configuration

Configuration file location: `~/.config/rott/config.toml`

```toml
# Data directory (default: ~/.local/share/rott)
data_dir = "/path/to/data"

# Sync server URL (optional)
sync_url = "wss://sync.example.com"

# Enable sync (default: false)
sync_enabled = true

# Tag used for Favorites filter (optional)
favorite_tag = "favorite"
```

### Environment Variables

Environment variables override config file values:

| Variable | Description |
|----------|-------------|
| `ROTT_DATA_DIR` | Data directory path |
| `ROTT_SYNC_URL` | Sync server URL |
| `ROTT_SYNC_ENABLED` | Enable sync (`true` or `1`) |

## Data Storage

ROTT uses a local-first architecture:

- **Automerge document**: Primary data store using CRDTs for conflict-free sync
- All queries are served directly from the in-memory Automerge document

Data is stored in the data directory (default `~/.local/share/rott`):

```
~/.local/share/rott/
├── document.automerge   # Automerge document
├── root_doc_id          # Document identity
└── sync_state.json      # Sync state
```

## Sync

ROTT supports real-time sync using the Automerge sync protocol over WebSocket. To enable sync:

1. Set `sync_url` to your sync server address
2. Set `sync_enabled = true`

When sync is enabled, changes are automatically synchronized in real-time. The sync protocol handles conflicts automatically using Automerge's CRDT merge semantics.

## License

MIT License - see [LICENSE](LICENSE) for details.
