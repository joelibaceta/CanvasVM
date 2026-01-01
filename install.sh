#!/bin/sh
# Canvas VM Installer Script

set -e

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64) PLATFORM="linux-x86_64" ;;
            aarch64|arm64) PLATFORM="linux-aarch64" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64) PLATFORM="macos-x86_64" ;;
            arm64) PLATFORM="macos-aarch64" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows-x86_64"
        EXT=".exe"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

BINARY_NAME="canvas-vm${EXT}"
DOWNLOAD_URL="https://github.com/joelibaceta/CanvasVM/releases/latest/download/canvas-vm-${PLATFORM}${EXT}"
INSTALL_DIR="${HOME}/.local/bin"

echo "Installing Canvas VM for ${PLATFORM}..."

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download binary
echo "Downloading from $DOWNLOAD_URL..."
if command -v curl > /dev/null; then
    curl -L "$DOWNLOAD_URL" -o "${INSTALL_DIR}/${BINARY_NAME}"
elif command -v wget > /dev/null; then
    wget "$DOWNLOAD_URL" -O "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Error: curl or wget required"
    exit 1
fi

# Make executable (Unix)
if [ "$OS" != "MINGW*" ] && [ "$OS" != "MSYS*" ]; then
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
fi

# Check if in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo ""
    echo "Add to PATH:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Add this to your ~/.bashrc or ~/.zshrc"
fi

echo "Canvas VM installed to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""
echo "Try it:"
echo "   canvas-vm --help"
