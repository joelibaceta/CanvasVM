#!/bin/bash
set -e

# Canvas VM Installation Script
# This script installs the Canvas VM CLI tools (piet, pietc)

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BOLD='\033[1m'
RESET='\033[0m'

echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  Canvas VM - Piet Language Runtime Installer${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}[ERROR]${RESET} Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo -e "${BLUE}[INFO]${RESET} Detected system: $OS-$ARCH"
echo ""

# Check for required tools
if ! command -v curl &> /dev/null; then
    echo -e "${RED}[ERROR]${RESET} curl is required but not installed"
    exit 1
fi

if ! command -v tar &> /dev/null; then
    echo -e "${RED}[ERROR]${RESET} tar is required but not installed"
    exit 1
fi

# GitHub repository info
REPO="joelibaceta/CanvasVM"
LATEST_URL="https://api.github.com/repos/$REPO/releases/latest"

echo -e "${BLUE}[INFO]${RESET} Fetching latest release..."

# Get latest release info
RELEASE_INFO=$(curl -s "$LATEST_URL")
VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}[ERROR]${RESET} Failed to fetch release information"
    echo "   Please check https://github.com/$REPO/releases"
    exit 1
fi

echo -e "${GREEN}[OK]${RESET} Latest version: $VERSION"
echo ""

# Construct download URL based on OS and arch
BINARY_NAME="canvas-vm-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

echo -e "${BLUE}[INFO]${RESET} Downloading Canvas VM..."
echo "   URL: $DOWNLOAD_URL"

# Create temporary directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download the release
if ! curl -L -o "$TMP_DIR/$BINARY_NAME" "$DOWNLOAD_URL" 2>/dev/null; then
    echo ""
    echo -e "${YELLOW}[WARN]${RESET} Failed to download binary"
    echo "   Pre-built binaries might not be available for $OS-$ARCH"
    echo ""
    echo -e "${BLUE}[INFO]${RESET} Building from source instead..."
    echo ""
    
    # Check for Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${BLUE}[INFO]${RESET} Installing Rust toolchain..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    echo -e "${BLUE}[INFO]${RESET} Cloning repository..."
    git clone https://github.com/$REPO.git "$TMP_DIR/canvas-vm"
    cd "$TMP_DIR/canvas-vm"
    
    echo -e "${BLUE}[INFO]${RESET} Building Canvas VM (this may take a few minutes)..."
    cargo build --release
    
    # Install directory
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    
    echo -e "${BLUE}[INFO]${RESET} Installing binaries to $INSTALL_DIR..."
    cp target/release/canvas-vm "$INSTALL_DIR/" 2>/dev/null || true
    cp target/release/piet "$INSTALL_DIR/" 2>/dev/null || true
    cp target/release/pietc "$INSTALL_DIR/" 2>/dev/null || true
else
    echo -e "${GREEN}[OK]${RESET} Downloaded successfully"
    echo ""
    echo -e "${BLUE}[INFO]${RESET} Extracting..."
    
    # Extract the tarball
    tar -xzf "$TMP_DIR/$BINARY_NAME" -C "$TMP_DIR"
    
    # Install directory
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    
    echo -e "${BLUE}[INFO]${RESET} Installing to $INSTALL_DIR..."
    cp "$TMP_DIR/canvas-vm" "$INSTALL_DIR/" 2>/dev/null || true
    cp "$TMP_DIR/piet" "$INSTALL_DIR/" 2>/dev/null || true
    cp "$TMP_DIR/pietc" "$INSTALL_DIR/" 2>/dev/null || true
fi

# Make binaries executable
chmod +x "$INSTALL_DIR/canvas-vm" 2>/dev/null || true
chmod +x "$INSTALL_DIR/piet" 2>/dev/null || true
chmod +x "$INSTALL_DIR/pietc" 2>/dev/null || true

echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}${BOLD}[SUCCESS]${RESET} Canvas VM installed successfully!"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    # Detect shell and add to PATH automatically
    SHELL_NAME=$(basename "$SHELL")
    SHELL_CONFIG=""
    
    case "$SHELL_NAME" in
        bash)
            SHELL_CONFIG="$HOME/.bashrc"
            # On macOS, bash uses .bash_profile
            if [[ "$OS" == "darwin" ]] && [[ -f "$HOME/.bash_profile" ]]; then
                SHELL_CONFIG="$HOME/.bash_profile"
            fi
            ;;
        zsh)
            SHELL_CONFIG="$HOME/.zshrc"
            ;;
        fish)
            SHELL_CONFIG="$HOME/.config/fish/config.fish"
            ;;
        *)
            SHELL_CONFIG="$HOME/.profile"
            ;;
    esac
    
    # Check if PATH export already exists in config file
    if [[ -f "$SHELL_CONFIG" ]] && grep -q "$INSTALL_DIR" "$SHELL_CONFIG" 2>/dev/null; then
        echo -e "${GREEN}[OK]${RESET} Installation directory already configured in PATH"
    else
        # Add to shell config
        echo "" >> "$SHELL_CONFIG"
        echo "# Canvas VM" >> "$SHELL_CONFIG"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$SHELL_CONFIG"
        echo -e "${GREEN}[OK]${RESET} Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
        echo ""
        echo -e "${YELLOW}[ACTION]${RESET} Run this to use Canvas VM now:"
        echo "   source $SHELL_CONFIG"
    fi
    echo ""
else
    echo -e "${GREEN}[OK]${RESET} Installation directory is already in PATH"
    echo ""
fi

echo -e "${BOLD}Quick start:${RESET}"
echo "   piet run program.png       # Run a Piet program"
echo "   pietc compile program.png  # Compile to bytecode"
echo "   piet --help                # Show all commands"
echo ""
echo -e "${BLUE}Documentation:${RESET} https://github.com/$REPO"
echo ""
