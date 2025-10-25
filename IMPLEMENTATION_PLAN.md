# Implementation Plan: CLI Link Creation

## Overview

Add a CLI command to create new links from the command line: `rott create link {URL} --tags {csv,list,of,tags}`

## Current State Analysis

- **Link Model**: Already supports all necessary fields (title, source, tags, created, etc.)
- **LinkService**: Has methods for loading/deleting/moving links, but no `create_link` or `save_link` method
- **Main Entry Point**: Currently only runs the TUI application
- **Config**: Stores `links_path` where markdown files are saved

## Implementation Stages

### Stage 1: Add CLI Argument Parsing

**Goal**: Add clap dependency and parse CLI arguments

**Success Criteria**:

- Binary accepts both TUI mode (no args) and CLI mode (`create link`)
- Arguments parsed: URL and optional --tags flag
- Tests pass

**Implementation**:

- Add `clap` with derive feature to Cargo.toml
- Create CLI struct with subcommands (CreateCommand)
- Update main.rs to check for CLI args before launching TUI
- Add unit tests for argument parsing

**Status**: ✅ Complete

---

### Stage 2: Add Link Creation Logic

**Goal**: Implement link creation and file writing in LinkService

**Success Criteria**:

- Can create a Link from minimal data (URL + tags)
- Can write Link to markdown file with proper frontmatter
- File saved to correct location from config
- Tests verify file creation and content format

**Implementation**:

- Add `create_link` method to LinkService
- Add `save_link_to_file` method to write markdown with YAML frontmatter
- Generate filename from URL (slugify title or use timestamp)
- Use serde_yaml to serialize frontmatter
- Add comprehensive tests for file creation

**Tests**:

- Test creating link with URL only
- Test creating link with URL and tags
- Test file naming logic
- Test frontmatter serialization
- Test file is created in correct directory

**Status**: ✅ Complete

---

### Stage 3: Fetch URL Metadata (Optional Enhancement)

**Goal**: Automatically fetch page title, description, and author from URL

**Success Criteria**:

- HTTP request fetches page content
- HTML parsing extracts title, description, and author meta tags
- Graceful fallback if fetch fails (use URL as title)
- Tests verify metadata extraction

**Implementation**:

- Add `reqwest` and `scraper` dependencies
- Create `fetch_url_metadata` function
- Extract title from `<title>` tag
- Extract description from `<meta name="description">` tag or `<meta property="og:description">`
- Extract author from multiple possible sources (priority order):
  - `<meta name="author">` tag
  - `<meta property="article:author">` tag (Open Graph)
  - `<meta name="twitter:creator">` tag (Twitter Cards)
  - JSON-LD structured data (`@type: Person` or `Article.author`)
  - `rel="author"` link tag
- Handle errors gracefully (timeout, network failure, invalid HTML)
- Make this feature optional/async

**Tests**:

- Test successful metadata fetch with all fields
- Test author extraction from different meta tag types
- Test fallback on network failure
- Test fallback on invalid HTML
- Test timeout handling
- Test partial metadata (e.g., title but no author)

**Status**: ✅ Complete

---

### Stage 4: CLI Integration and User Experience

**Goal**: Wire everything together and polish UX

**Success Criteria**:

- `rott create link <URL>` creates a basic link
- `rott create link <URL> --tags tag1,tag2,tag3` creates link with tags
- Success/error messages printed to user
- Created file path shown to user
- Integration tests pass

**Implementation**:

- Create `create_link_command` handler function
- Call LinkService methods from CLI handler
- Print user-friendly success/error messages
- Show path to created file
- Add integration test for full flow

**Tests**:

- Integration test: full CLI command execution
- Test error handling (invalid URL, filesystem errors)
- Test success message output

**Status**: ✅ Complete

---

## Implementation Complete! 🎉

All 4 stages have been successfully implemented and tested.

## Technical Decisions

### Dependencies to Add

```toml
clap = { version = "4.5", features = ["derive"] }
reqwest = { version = "0.12", features = ["blocking"], optional = true }
scraper = { version = "0.20", optional = true }
slug = "0.1"
```

### File Naming Strategy

Options:

1. **Timestamp-based**: `YYYY-MM-DD-HHMMSS.md` (simple, always unique)
2. **Title-based**: `slugified-title.md` (readable, may conflict)
3. **Hash-based**: First 8 chars of URL hash (unique, not readable)

**Decision**: Use timestamp-based naming for Stage 2, consider title-based in Stage 3 when we have metadata.

### CLI Structure

```rust
#[derive(Parser)]
#[command(name = "rott")]
#[command(about = "Brain ROTT - Link management system")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        #[command(subcommand)]
        resource: CreateCommands,
    },
}

#[derive(Subcommand)]
enum CreateCommands {
    Link {
        url: String,
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
}
```

### Markdown File Format

```markdown
---
title: Page Title (or URL if no metadata)
source: https://example.com
author:
  - Author Name (from meta tags)
published: null
created: 2025-10-24
description: Page description from meta tags
tags:
  - tag1
  - tag2
---
```

Note: Author field will be populated from scraped metadata when available, otherwise empty array.

## Testing Strategy

1. **Unit Tests**: Each new method in LinkService
2. **Integration Tests**: Full CLI command execution
3. **Manual Testing**: Run CLI commands and verify files created correctly

## Acceptance Criteria

- [x] `rott` with no args launches TUI (existing behavior preserved)
- [x] `rott create link https://example.com` creates a markdown file
- [x] `rott create link https://example.com --tags rust,cli` creates file with tags
- [x] Created files have proper YAML frontmatter
- [x] Created files are saved to `links_path` from config
- [x] Error messages are clear and helpful
- [x] All tests pass (34 tests passing)
- [x] No regressions in TUI functionality
- [x] Metadata scraping works (title, description, author)
- [x] Graceful fallback when metadata fetch fails
