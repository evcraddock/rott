# Changelog

All notable changes to ROTT will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.5.1] - 2026-02-07

### Fixed
- Always fetch metadata regardless of `-q` flag — quiet mode should only suppress output, not skip metadata fetching

## [2.5.0] - 2026-02-07

### Changed
- **Removed SQLite projection layer** — all queries now served directly from the in-memory Automerge document, dramatically improving startup and write performance
- **Skip metadata fetch in quiet mode** — `rott link create -q` no longer makes HTTP requests, enabling near-instant link creation (e.g., from newsboat bookmarks)
- Search uses in-memory substring matching instead of SQLite FTS5
- Simplified save path: no more full database rebuild on every operation

### Removed
- `rusqlite` dependency (including bundled C SQLite compilation)
- `SqliteProjection`, `schema.rs`, and `projection.rs`
- `sqlite_path()` from configuration
- SQLite references from documentation

### Added
- `get_link_by_url()` on `RottDocument` for duplicate detection
- `search_links()` on `RottDocument` for in-memory search
- `get_tags_with_counts()`, `link_count()`, `note_count()` on `RottDocument`
- Sync server setup guide (`docs/SYNC_SERVER_SETUP.md`)
- Root document ID explainer (`docs/IDENTITY.md`)
- Troubleshooting guide (`docs/TROUBLESHOOTING.md`)
- Contributing guide (`CONTRIBUTING.md`)
- This changelog

### Changed
- Updated `docs/ARCHITECTURE.md` to reflect actual implementation
- Updated `docs/plans/` with implementation status

## [2.3.0] - 2026-01-15

### Added
- Device setup wizard in TUI for first-run experience
- Settings panel in TUI to view root document ID and sync status
- Global `--config` CLI flag to specify custom config file
- Improved error handling with descriptive messages and recovery hints

### Fixed
- TUI now respects `--config` CLI flag
- Config override properly applied when opening store after wizard

## [2.2.0] - 2026-01-11

### Added
- `gg` and `G` vim keys in TUI for jumping to first/last item
- Links now sorted by date ascending (oldest first) by default

### Fixed
- `--quiet` flag now suppresses all output on `link create`

## [2.1.1] - 2026-01-08

### Fixed
- Join flow no longer creates divergent Automerge documents
- Fixed data sync issues when joining existing identity

## [2.1.0] - 2026-01-08

### Added
- First device setup flow with interactive prompts
- `rott init` now shows guided setup for new users

### Fixed
- Test isolation improvements for CI reliability

## [2.0.1] - 2026-01-08

### Fixed
- Merge external changes before saving to prevent data loss during concurrent edits
- Updated README for v2 with accurate installation instructions

### Added
- Installation script (`install.sh`)

## [2.0.0] - 2026-01-08

Complete rewrite of ROTT as a local-first application with Automerge-based sync.

### Added
- **Local-first architecture** using Automerge CRDTs
- **Multi-device sync** via automerge-repo-sync-server
- **CLI application** (`rott`) with full CRUD operations
- **TUI application** (`rott tui`) with vim-style navigation
- **Root document identity** model (no accounts needed)
- **SQLite projection** for fast queries alongside Automerge source of truth
- Link management with title, URL, description, author, tags
- Notes as annotations attached to links
- Tag-based organization and filtering
- Favorites filter with configurable tag
- WebSocket-based persistent sync connection
- Duplicate URL prevention
- Metadata fetching from URLs (title, description, author)

### Changed
- Complete architectural redesign from file-based to CRDT-based storage
- Notes are now children of links (not independent documents)
- Combined CLI and TUI into single binary

### Migration
- This is a breaking change from ROTT v1
- No automatic migration from v1 markdown files (planned for future release)

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 2.3.0 | 2026-01-15 | TUI setup wizard, settings panel |
| 2.2.0 | 2026-01-11 | Vim navigation improvements |
| 2.1.1 | 2026-01-08 | Join flow fix |
| 2.1.0 | 2026-01-08 | First device setup flow |
| 2.0.1 | 2026-01-08 | Data loss prevention fix |
| 2.0.0 | 2026-01-08 | Initial v2 release |

[Unreleased]: https://github.com/evcraddock/rott/compare/v2.3.0...HEAD
[2.3.0]: https://github.com/evcraddock/rott/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/evcraddock/rott/compare/v2.1.1...v2.2.0
[2.1.1]: https://github.com/evcraddock/rott/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/evcraddock/rott/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/evcraddock/rott/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/evcraddock/rott/releases/tag/v2.0.0
