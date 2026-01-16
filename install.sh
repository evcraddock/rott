#!/bin/bash
# ROTT installer
# Usage: curl -fsSL https://raw.githubusercontent.com/evcraddock/rott/main/install.sh | bash

set -e

REPO="evcraddock/rott"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64) TARGET="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        echo "For Windows, download from: https://github.com/$REPO/releases"
        exit 1
        ;;
esac

# Get latest release version
echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

echo "Installing ROTT $LATEST for $TARGET..."

# Download and extract
URL="https://github.com/$REPO/releases/download/$LATEST/rott-$TARGET.tar.gz"
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

curl -fsSL "$URL" -o "$TEMP_DIR/rott.tar.gz"
tar -xzf "$TEMP_DIR/rott.tar.gz" -C "$TEMP_DIR"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TEMP_DIR/rott" "$INSTALL_DIR/rott"
chmod +x "$INSTALL_DIR/rott"

echo ""
echo "✓ Installed rott to $INSTALL_DIR/rott"

# Check if in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo ""
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add it with:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "Run 'rott --help' to get started, or just 'rott' to launch the TUI."
echo ""
echo "For man pages, clone the repo and run 'sudo make install-man':"
echo "  git clone https://github.com/$REPO.git && cd rott && sudo make install-man"
