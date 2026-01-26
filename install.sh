#!/bin/bash
set -e

# Saki (Murasaki) installation script
# Usage: curl -fsSL https://raw.githubusercontent.com/igorvieira/murasaki_rs/main/install.sh | bash

VERSION="${VERSION:-0.1.1}"
REPO="igorvieira/murasaki_rs"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="saki"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)
        OS_TYPE="linux"
        ;;
    Darwin*)
        OS_TYPE="macos"
        ;;
    *)
        echo "Unsupported operating system: ${OS}"
        exit 1
        ;;
esac

case "${ARCH}" in
    x86_64)
        ARCH_TYPE="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo "Unsupported architecture: ${ARCH}"
        exit 1
        ;;
esac

# Construct download URL for the release binary
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/saki-${OS_TYPE}-${ARCH_TYPE}"

echo "Installing Saki (Murasaki)..."
echo "Version: ${VERSION}"
echo "OS: ${OS_TYPE}"
echo "Architecture: ${ARCH_TYPE}"
echo ""

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download binary
echo "Downloading from ${DOWNLOAD_URL}..."
if command -v curl > /dev/null 2>&1; then
    if ! curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${BINARY_NAME}"; then
        echo "Error: Failed to download binary. Make sure version ${VERSION} is released."
        exit 1
    fi
elif command -v wget > /dev/null 2>&1; then
    if ! wget -q "${DOWNLOAD_URL}" -O "${TMP_DIR}/${BINARY_NAME}"; then
        echo "Error: Failed to download binary. Make sure version ${VERSION} is released."
        exit 1
    fi
else
    echo "Error: Neither curl nor wget is available. Please install one of them."
    exit 1
fi

# Make binary executable
chmod +x "${TMP_DIR}/${BINARY_NAME}"

# Install binary
if [ ! -w "$INSTALL_DIR" ]; then
    echo "Warning: $INSTALL_DIR is not writable. Installing to ~/.local/bin instead."
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
else
    sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
fi

echo ""
echo "Success! Saki has been installed to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""

# Check if command is accessible
if command -v ${BINARY_NAME} > /dev/null 2>&1; then
    echo "${BINARY_NAME} is ready to use!"
    ${BINARY_NAME} --version
    echo ""
    echo "Run 'saki' in a repository with git conflicts to start."
elif [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
    echo "Note: ${INSTALL_DIR} might not be in your PATH."
    echo ""
    echo "Add the following line to your shell configuration file:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then reload your shell or run:"
    echo "  source ~/.bashrc  # for bash"
    echo "  source ~/.zshrc   # for zsh"
else
    echo "Warning: ${BINARY_NAME} command not found in PATH."
    echo "You may need to restart your terminal or add ${INSTALL_DIR} to your PATH."
fi
