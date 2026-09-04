#!/usr/bin/env bash
# PaperLite - Linux Setup Script
# Installs Rust toolchain and swaybg if missing.
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[✓]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }

# --- Detect package manager ---
detect_pkg_manager() {
    if command -v pacman &>/dev/null; then
        echo "pacman"
    elif command -v dnf &>/dev/null; then
        echo "dnf"
    elif command -v apt &>/dev/null; then
        echo "apt"
    elif command -v zypper &>/dev/null; then
        echo "zypper"
    else
        echo "unknown"
    fi
}

# --- Install Rust ---
install_rust() {
    if command -v cargo &>/dev/null; then
        info "Rust/Cargo already installed: $(cargo --version)"
        return
    fi

    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
        if command -v cargo &>/dev/null; then
            info "Rust/Cargo already installed: $(cargo --version)"
            return
        fi
    fi

    warn "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    info "Rust installed: $(cargo --version)"
}

# --- Install swaybg ---
install_swaybg() {
    if command -v swaybg &>/dev/null; then
        info "swaybg already installed"
        return
    fi

    warn "swaybg not found. Attempting to install..."
    PKG=$(detect_pkg_manager)
    case "$PKG" in
        pacman)
            sudo pacman -S --noconfirm swaybg
            ;;
        dnf)
            sudo dnf install -y swaybg
            ;;
        apt)
            sudo apt update && sudo apt install -y swaybg
            ;;
        zypper)
            sudo zypper install -y swaybg
            ;;
        *)
            error "Could not detect package manager."
            error "Please install 'swaybg' manually for your distro."
            return 1
            ;;
    esac
    info "swaybg installed"
}

# --- Main ---
echo ""
echo "  PaperLite - Linux Setup"
echo "  ======================="
echo ""

install_rust
install_swaybg

echo ""
info "Setup complete! Run ./run.sh to launch PaperLite."
echo ""
