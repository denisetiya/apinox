#!/bin/bash
set -euo pipefail

# Apinox Installer
# Usage: curl -sSL https://apinox.denisetiya.site/install.sh | bash

REPO="denisetiya/apinox"
INSTALL_DIR="${APINOX_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${APINOX_VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}▸${NC} $1"; }
ok()    { echo -e "${GREEN}✓${NC} $1"; }
warn()  { echo -e "${YELLOW}!${NC} $1"; }
err()   { echo -e "${RED}✗${NC} $1"; exit 1; }

# Detect OS and arch
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) err "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "Unsupported architecture: $(uname -m)" ;;
    esac

    if [ "$os" = "windows" ]; then
        PLATFORM="${os}-${arch}.exe"
        BINARY="apinox.exe"
    else
        PLATFORM="${os}-${arch}"
        BINARY="apinox"
    fi
}

# Get latest version from GitHub
get_version() {
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -z "$VERSION" ]; then
            # Fallback: use tag from API
            VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
        fi
        [ -z "$VERSION" ] && err "Could not determine latest version"
    fi
}

# Download binary
download() {
    local url="https://github.com/${REPO}/releases/download/${VERSION}/apinox-${PLATFORM}"
    local tmp_dir
    tmp_dir=$(mktemp -d)

    info "Downloading apinox ${VERSION} for ${PLATFORM}..."
    if ! curl -sL "$url" -o "${tmp_dir}/${BINARY}"; then
        err "Download failed: $url"
    fi

    chmod +x "${tmp_dir}/${BINARY}"

    # Install
    mkdir -p "$INSTALL_DIR"
    mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    rm -rf "$tmp_dir"

    ok "Installed to ${INSTALL_DIR}/${BINARY}"
}

# Add to PATH if needed
setup_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            local shell_profile
            if [ -f "$HOME/.bashrc" ]; then
                shell_profile="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_profile="$HOME/.bash_profile"
            elif [ -f "$HOME/.zshrc" ]; then
                shell_profile="$HOME/.zshrc"
            fi

            if [ -n "${shell_profile:-}" ]; then
                echo "" >> "$shell_profile"
                echo "# Apinox" >> "$shell_profile"
                echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$shell_profile"
                warn "Added ${INSTALL_DIR} to PATH in ${shell_profile}"
                warn "Run: source ${shell_profile}"
            else
                warn "Add ${INSTALL_DIR} to your PATH"
            fi
            ;;
    esac
}

# Verify installation
verify() {
    if "${INSTALL_DIR}/${BINARY}" --version >/dev/null 2>&1; then
        local version
        version=$("${INSTALL_DIR}/${BINARY}" --version 2>&1)
        ok "Installed: ${version}"
    else
        warn "Binary installed but could not verify version"
    fi
}

main() {
    echo ""
    echo -e "${BLUE}  ╔══════════════════════════════════╗${NC}"
    echo -e "${BLUE}  ║       Apinox Installer           ║${NC}"
    echo -e "${BLUE}  ╚══════════════════════════════════╝${NC}"
    echo ""

    detect_platform
    get_version
    download
    setup_path
    verify

    echo ""
    echo -e "${GREEN}  Done!${NC} Run ${BLUE}apinox --help${NC} to get started."
    echo ""
}

main "$@"
