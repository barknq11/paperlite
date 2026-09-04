#!/usr/bin/env bash
# PaperLite Linux Launcher
set -e

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
cd "$SCRIPT_DIR"

# Source Cargo environment if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Check for Rust/Cargo
if ! command -v cargo &>/dev/null; then
    echo ""
    echo "  Error: Rust/Cargo is not installed."
    echo ""
    echo "  Run this first to install dependencies:"
    echo ""
    echo "    ./setup.sh"
    echo ""
    exit 1
fi

if [ ! -f "target/release/paperlite" ]; then
    echo "Building PaperLite (Release)..."
    cargo build --release
fi

exec "$SCRIPT_DIR/target/release/paperlite" "$@"
