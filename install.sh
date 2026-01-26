#!/bin/bash
set -e

# Murasaki installation script
# Usage: curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/murasaki_rs/main/install.sh | sh

REPO="YOUR_USERNAME/murasaki_rs"
INSTALL_DIR="${HOME}/.local/bin"
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
# This assumes you'll create releases with binaries named like: saki-linux-x86_64, saki-macos-aarch64, etc.
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/saki-${OS_TYPE}-${ARCH_TYPE}"

echo "Installing Murasaki..."
echo "OS: ${OS_TYPE}"
echo "Architecture: ${ARCH_TYPE}"

# Create install directory if it doesn't exist
mkdir -p "${INSTALL_DIR}"

# Download binary
echo "Downloading from ${DOWNLOAD_URL}..."
if command -v curl > /dev/null 2>&1; then
    curl -sSL "${DOWNLOAD_URL}" -o "${INSTALL_DIR}/${BINARY_NAME}"
elif command -v wget > /dev/null 2>&1; then
    wget -q "${DOWNLOAD_URL}" -O "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Error: Neither curl nor wget is available. Please install one of them."
    exit 1
fi

# Make binary executable
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "✓ Murasaki has been installed to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""

# Check if install directory is in PATH
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        echo "✓ ${INSTALL_DIR} is already in your PATH"
        echo ""
        echo "You can now use saki by running: murasaki_rs"
        ;;
    *)
        echo "⚠ ${INSTALL_DIR} is not in your PATH"
        echo ""
        echo "Add the following line to your shell configuration file:"
        echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
        echo ""
        echo "Then reload your shell or run:"
        echo "  source ~/.bashrc  # for bash"
        echo "  source ~/.zshrc   # for zsh"
        ;;
esac

echo ""
echo "Get started with: murasaki_rs --help"
