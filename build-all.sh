#!/bin/bash
set -euo pipefail

# Apinox — Build all platforms (run on Linux)
# Produces binaries for Linux x86_64, Linux ARM64, Windows x86_64
# macOS must be built on macOS (see .github/workflows/release.yml)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASES_DIR="${SCRIPT_DIR}/releases"
VERSION="${1:-dev}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}▸${NC} $1"; }
ok()   { echo -e "${GREEN}✓${NC} $1"; }

mkdir -p "$RELEASES_DIR"

build_target() {
    local target=$1
    local name=$2
    local ext=${3:-}

    info "Building ${name}..."
    cargo build --release --target "$target" 2>&1 | tail -1

    local src="target/${target}/release/apinox${ext}"
    local dst="${RELEASES_DIR}/apinox-${name}${ext}"

    cp "$src" "$dst"
    chmod +x "$dst"
    ok "${name}: $(ls -lh "$dst" | awk '{print $5}')"
}

echo ""
info "Apinox Build All ($VERSION)"
echo ""

# Linux x86_64
build_target "x86_64-unknown-linux-gnu" "linux-x86_64"

# Linux ARM64
build_target "aarch64-unknown-linux-gnu" "linux-aarch64"

# Windows x86_64
build_target "x86_64-pc-windows-gnu" "windows-x86_64" ".exe"

echo ""
ok "All builds complete!"
echo ""
ls -lh "$RELEASES_DIR"
echo ""
echo "  macOS builds require a macOS machine or CI/CD."
echo "  See: .github/workflows/release.yml"
