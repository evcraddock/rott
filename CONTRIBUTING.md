# Contributing to ROTT

Thank you for your interest in contributing to ROTT! This guide will help you get started.

## Getting Started

### Prerequisites

- **Rust** (stable toolchain) - Install via [rustup](https://rustup.rs/)
- **Docker** (optional) - For running the sync server locally
- **Git** - For version control

### Clone and Build

```bash
git clone https://github.com/evcraddock/rott.git
cd rott

# Build all crates
make build
# or: cargo build

# Run tests
make test
# or: cargo test

# Run linters
make lint
# or: cargo fmt --check && cargo clippy
```

### Development Environment

Start a local sync server for testing:

```bash
# Start sync server via Docker
make svc-start

# Or use the full dev environment
make dev

# Stop when done
make svc-stop
# or: make dev-stop
```

The sync server runs at `ws://localhost:3030`.

---

## Project Structure

```
rott/
├── crates/
│   ├── rott-core/           # Core library (business logic, storage, sync)
│   │   ├── config.rs        # Application configuration
│   │   ├── document.rs      # Automerge document handling
│   │   ├── identity.rs      # User identity management
│   │   ├── models.rs        # Link, Note data structures
│   │   ├── store.rs         # Unified storage interface
│   │   ├── storage/         # Persistence layer
│   │   └── sync/            # Sync client
│   │
│   └── rott-cli/            # CLI and TUI application
│       ├── commands/        # CLI command handlers
│       └── tui/             # Terminal UI (ratatui)
│
├── docs/                    # Documentation
│   ├── ARCHITECTURE.md      # System architecture
│   └── plans/               # Implementation plans
│
└── hack/                    # Development scripts
```

### Key Components

| Component | Purpose |
|-----------|---------|
| `rott-core` | All business logic, data models, storage, and sync |
| `rott-cli` | CLI commands and TUI presentation |
| `Store` | Main entry point for data operations |
| `RottDocument` | Automerge document wrapper |
| `SyncClient` | WebSocket sync with automerge-repo servers |

---

## Development Workflow

### 1. Find or Create an Issue

- Check [existing issues](https://github.com/evcraddock/rott/issues)
- Comment on an issue you'd like to work on
- Or create a new issue describing your proposed change

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or: git checkout -b fix/bug-description
```

### 3. Make Changes

- Write code following the existing style
- Add tests for new functionality
- Update documentation if needed

### 4. Run Checks

Before committing:

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Or all at once
make lint && make test
```

### 5. Commit

Write clear commit messages:

```
feat: add tag filtering to link list

- Add --tag flag to `rott link list`
- Filter links by one or more tags
- Update help text and tests
```

Use conventional commit prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

### 6. Push and Create PR

```bash
git push origin your-branch-name
```

Then create a pull request on GitHub.

---

## Code Guidelines

### Style

- Run `cargo fmt` before committing
- Follow Clippy suggestions
- Match the existing code style

### Architecture Principles

- **Core contains all logic** - CLI/TUI should be thin wrappers
- **Automerge is the source of truth** - SQLite is a read projection
- **Fail fast with clear errors** - Use descriptive error messages
- **Test your changes** - Add tests for new functionality

### Error Handling

```rust
// Good: Descriptive error with context
.context("Failed to read config file")?

// Good: Typed errors with recovery hints
StorageError::PermissionDenied { path, source }

// Avoid: Silent failures or generic errors
.ok()
.expect("something went wrong")
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let link = Link::new("https://example.com");
        
        // Act
        let result = link.add_tag("rust");
        
        // Assert
        assert!(link.tags.contains(&"rust".to_string()));
    }
}
```

---

## Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p rott-core

# Specific test
cargo test test_link_new

# With output
cargo test -- --nocapture
```

---

## Documentation

### Code Documentation

- Add doc comments to public items
- Include examples where helpful

```rust
/// Create a new link from a URL.
///
/// # Example
///
/// ```
/// let link = Link::new("https://example.com");
/// assert_eq!(link.url, "https://example.com");
/// ```
pub fn new(url: impl Into<String>) -> Self {
```

### User Documentation

- Update docs/ when adding user-facing features
- Keep README.md current
- Add troubleshooting entries for new error conditions

---

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] Commit messages are clear

### PR Description

Include:
- What the change does
- Why it's needed
- Any breaking changes
- How to test it

### Review Process

1. Maintainers will review your PR
2. Address any feedback
3. Once approved, it will be merged

---

## Architecture Overview

For detailed architecture, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

### Data Flow

```
User Action → CLI/TUI → Store → Automerge Document
                              ↓
                        SQLite Projection (for queries)
                              ↓
                        Sync Client → Sync Server
```

### Key Patterns

1. **Local-first**: All operations work offline
2. **CRDT-based**: Automerge handles conflict resolution
3. **Projection**: SQLite mirrors Automerge for fast queries
4. **Thin CLI**: Business logic lives in rott-core

---

## Getting Help

- Open an issue for questions
- Check existing documentation
- Look at similar code in the project for examples

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
