#!/bin/bash
# DevHub Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/moinsen-dev/devhub/main/install/install.sh | bash

set -euo pipefail

REPO="moinsen-dev/devhub"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin) os="apple-darwin" ;;
        Linux) os="unknown-linux-gnu" ;;
        *) error "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${arch}-${os}"
}

# Get latest release version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name":' \
        | sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    echo ""
    echo "  ____             _   _       _     "
    echo " |  _ \\  _____   _| | | |_   _| |__  "
    echo " | | | |/ _ \\ \\ / / |_| | | | | '_ \\ "
    echo " | |_| |  __/\\ V /|  _  | |_| | |_) |"
    echo " |____/ \\___| \\_/ |_| |_|\\__,_|_.__/ "
    echo ""
    echo " Multi-project development environment manager"
    echo ""

    # Detect platform
    PLATFORM=$(detect_platform)
    info "Detected platform: $PLATFORM"

    # Get version
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
    fi
    info "Installing version: $VERSION"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/devhub-${PLATFORM}.tar.gz"
    info "Downloading from: $DOWNLOAD_URL"

    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    curl -fsSL "$DOWNLOAD_URL" | tar xz -C "$TMP_DIR"

    # Install
    mv "$TMP_DIR/devhub" "$INSTALL_DIR/devhub"
    chmod +x "$INSTALL_DIR/devhub"

    success "Installed devhub to $INSTALL_DIR/devhub"

    # Check if in PATH
    if ! command -v devhub &> /dev/null; then
        warn "Add $INSTALL_DIR to your PATH:"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
        echo "  Add this to your ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish"
    fi

    # Shell completions hint
    echo ""
    info "To enable shell completions:"
    echo ""
    echo "  # Bash"
    echo "  devhub completions bash > ~/.local/share/bash-completion/completions/devhub"
    echo ""
    echo "  # Zsh"
    echo "  devhub completions zsh > ~/.zfunc/_devhub"
    echo ""
    echo "  # Fish"
    echo "  devhub completions fish > ~/.config/fish/completions/devhub.fish"
    echo ""

    # macOS LaunchAgent hint
    if [ "$(uname -s)" = "Darwin" ]; then
        info "To start devhub daemon automatically on login (macOS):"
        echo ""
        echo "  # Download and install LaunchAgent"
        echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install/com.moinsen.devhub.plist \\"
        echo "    -o ~/Library/LaunchAgents/com.moinsen.devhub.plist"
        echo ""
        echo "  # Update path if needed, then load it"
        echo "  launchctl load ~/Library/LaunchAgents/com.moinsen.devhub.plist"
        echo ""
    fi

    success "Installation complete! Run 'devhub --help' to get started."
}

main "$@"
