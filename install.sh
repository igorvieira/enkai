#!/bin/bash
set -e

# Saki (Murasaki) installation script
# Usage: curl -fsSL https://raw.githubusercontent.com/igorvieira/murasaki_rs/main/install.sh | bash

REPO="igorvieira/murasaki_rs"
INSTALL_DIR="${HOME}/.config/murasaki_rs"

echo "Installing Saki (Murasaki)..."
echo ""

# Check for required commands
if ! command -v git >/dev/null 2>&1; then
    echo "Error: git is required but not installed"
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo is required but not installed"
    echo "Install Rust from: https://rustup.rs/"
    exit 1
fi

# Clone or update repository
if [ -d "$INSTALL_DIR" ]; then
    echo "Updating existing installation at $INSTALL_DIR..."
    cd "$INSTALL_DIR"
    git pull origin main
else
    echo "Cloning repository to $INSTALL_DIR..."
    git clone "https://github.com/${REPO}.git" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

echo ""
echo "Building and installing saki..."
cargo install --path .

echo ""
echo "Success! Saki has been installed."
echo ""

# Check if command is accessible
if command -v saki >/dev/null 2>&1; then
    echo "saki is ready to use!"
    saki --help
    echo ""
    echo "Run 'saki' in a repository with git conflicts to start."
else
    echo "Installation complete, but 'saki' command not found in PATH."
    echo ""
    echo "Make sure ~/.cargo/bin is in your PATH:"
    echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo ""
    echo "Add this line to your shell configuration file (~/.bashrc or ~/.zshrc)"
    echo "Then reload your shell or run:"
    echo "  source ~/.bashrc  # for bash"
    echo "  source ~/.zshrc   # for zsh"
fi
