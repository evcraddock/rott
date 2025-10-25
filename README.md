# Brain ROTT

Record of Tagged Topics - A terminal-based tool to organize and navigate your knowledge by tags.

## Overview

Brain ROTT is a terminal UI application that helps you organize and access content by topics (tags). It scans your configured directory for markdown files with front matter, extracts tags, and presents them in a navigable interface.

## Features

- Terminal-based interface using Ratatui
- Navigate between topic and content panes
- Open links directly in your browser
- Delete unwanted links
- Keyboard shortcuts for navigation

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo installed
- Linux, macOS or Windows

### Install from source

1. Clone the repository:

   ```bash
   git clone https://github.com/evcraddock/rott.git
   cd rott
   ```

2. Build and install:

   ```bash
   cargo build --release
   ```

3. Copy the binary to your system (optional):

   ```bash
   # Using the provided script (requires sudo)
   ./hack/deploy.sh
   
   # Or manually
   sudo cp target/release/rott /usr/local/bin/rott
   ```

### Configuration

Create a configuration file at `~/.config/rott/config.yaml`:

```yaml
links_path: "/path/to/your/markdown/files"
default_topic: "default-tag-to-select"
```

Alternatively, you can set configuration using environment variables:

```bash
export APP_LINKS_PATH="/path/to/your/markdown/files"
export APP_DEFAULT_TOPIC="default-tag-to-select"
```

## Usage

### Starting the application

```bash
rott
```

### Keyboard shortcuts

- `q`: Quit the application
- `Tab` or `l`: Move to right pane (Pages)
- `Shift+Tab` or `h`: Move to left pane (Topics)
- `↑` or `k`: Move selection up
- `↓` or `j`: Move selection down
- `Enter`: Open selected link in browser
- `Delete`: Remove selected link
- `r`: Refresh content

## Updating

To update to the latest version:

1. Pull the latest changes:

   ```bash
   git pull
   ```

2. Rebuild and reinstall:

   ```bash
   cargo build --release
   ./hack/deploy.sh  # Or copy manually to /usr/local/bin
   ```

## Content Format

Brain ROTT expects markdown files with front matter that includes:

- `title`: Title of the content
- `tags`: List of tags to categorize the content
- `source`: (Optional) URL to original content
- `author`: (Optional) Content author(s)
- `published`: (Optional) Publication date
- `description`: (Optional) Brief description

Example markdown file:

```markdown
---
title: "Example Article"
tags: ["rust", "programming", "tutorial"]
source: "https://example.com/article"
author: ["Jane Doe"]
published: 2023-01-15
description: "A tutorial about Rust programming"
---

Content of the article here...
```

## License

See the [LICENSE](LICENSE) file for details.
