#!/usr/bin/env bash
# PaperLite Linux Launcher
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "target/release/paperlite" ]; then
    echo "Building PaperLite (Release)..."
    cargo build --release
fi

exec "$SCRIPT_DIR/target/release/paperlite" "$@"
