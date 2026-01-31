#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/hoqqun/dtd-viewer.git"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="dtd-viewer"

need_cmd() {
    if ! command -v "$1" &>/dev/null; then
        echo "Error: '$1' is required but not found." >&2
        exit 1
    fi
}

need_cmd git
need_cmd cargo

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Cloning dtd-viewer..."
git clone --depth 1 "$REPO" "$TMPDIR/dtd-viewer"

echo "Building (release)..."
cargo build --release --manifest-path "$TMPDIR/dtd-viewer/Cargo.toml"

echo "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
install -Dm755 "$TMPDIR/dtd-viewer/target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

echo "Done. Run 'dtd-viewer --help' to get started."
