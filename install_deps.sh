#!/usr/bin/env bash
# ============================================================================
# AXIOM Dependency Auto-Installer — macOS & Linux
# ============================================================================
# Detects OS/distro, checks for required build/runtime dependencies,
# and auto-installs any that are missing.
#
# Dependencies: Rust (rustc/cargo), LLVM (clang), C build tools (cc/linker)
#
# Usage:
#   chmod +x install_deps.sh
#   ./install_deps.sh
# ============================================================================

set -euo pipefail

# ── Colors ──────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; GRAY='\033[0;90m'; MAGENTA='\033[0;35m'; NC='\033[0m'

ok()    { echo -e "    ${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "   ${YELLOW}[WARN]${NC} $1"; }
fail()  { echo -e "   ${RED}[FAIL]${NC} $1"; }
info()  { echo -e "   ${GRAY}[INFO]${NC} $1"; }
header(){ echo -e "\n  ${CYAN}$1${NC}"; }

installed=0; skipped=0; failed=0

# ── Banner ──────────────────────────────────────────────────────────────
clear 2>/dev/null || true
echo ""
echo -e "  ${MAGENTA}AXIOM Dependency Installer (macOS / Linux)${NC}"
echo -e "  ${MAGENTA}------------------------------------------${NC}"
echo ""

# ── OS Detection ────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
    Darwin)  OS_TYPE="macos" ;;
    Linux)   OS_TYPE="linux" ;;
    *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

if [ "$OS_TYPE" = "linux" ]; then
    # Detect distro
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO="${ID:-unknown}"
    elif command -v apt-get >/dev/null 2>&1; then
        DISTRO="debian"
    elif command -v dnf >/dev/null 2>&1; then
        DISTRO="fedora"
    elif command -v pacman >/dev/null 2>&1; then
        DISTRO="arch"
    else
        DISTRO="unknown"
    fi
else
    DISTRO="macos"
fi

info "OS:      $OS_TYPE"
info "Distro:  $DISTRO"
info "Package managers: $(command -v brew >/dev/null 2>&1 && echo -n 'brew ' || true)$(command -v apt-get >/dev/null 2>&1 && echo -n 'apt ' || true)$(command -v dnf >/dev/null 2>&1 && echo -n 'dnf ' || true)$(command -v pacman >/dev/null 2>&1 && echo -n 'pacman ' || true)$(command -v apk >/dev/null 2>&1 && echo -n 'apk ' || true)"
echo ""

# ── Helper: detect sudo ─────────────────────────────────────────────────
if command -v sudo >/dev/null 2>&1 && [ "$(id -u)" -ne 0 ]; then
    SUDO="sudo"
else
    SUDO=""
fi

# ── Helper: install via detected package manager ─────────────────────────
install_pkg() {
    local name="$1"
    local brew_pkg="${2:-$1}"
    local apt_pkg="${3:-$1}"
    local dnf_pkg="${4:-$1}"
    local pacman_pkg="${5:-$1}"

    case "$OS_TYPE/$DISTRO" in
        macos/*)
            if command -v brew >/dev/null 2>&1; then
                info "Installing $name via Homebrew..."
                brew install "$brew_pkg" && return 0
            fi
            ;;
        linux/debian|linux/ubuntu|linux/linuxmint|linux/pop|linux/elementary)
            if command -v apt-get >/dev/null 2>&1; then
                info "Installing $name via apt..."
                $SUDO apt-get update -qq 2>/dev/null || true
                $SUDO apt-get install -y -qq "$apt_pkg" 2>&1 && return 0
            fi
            ;;
        linux/fedora|linux/rhel|linux/centos|linux/rocky|linux/almalinux)
            if command -v dnf >/dev/null 2>&1; then
                info "Installing $name via dnf..."
                $SUDO dnf install -y "$dnf_pkg" 2>&1 && return 0
            fi
            ;;
        linux/arch|linux/manjaro|linux/endeavouros)
            if command -v pacman >/dev/null 2>&1; then
                info "Installing $name via pacman..."
                $SUDO pacman -S --noconfirm "$pacman_pkg" 2>&1 && return 0
            fi
            ;;
        linux/alpine)
            if command -v apk >/dev/null 2>&1; then
                info "Installing $name via apk..."
                $SUDO apk add "$apt_pkg" 2>&1 && return 0
            fi
            ;;
    esac
    return 1
}

# ========================================================================
# 1. Rust (rustc + cargo)
# ========================================================================
header "1. Rust (rustc + cargo)"

if command -v rustc >/dev/null 2>&1; then
    ok "already installed — $(rustc --version 2>/dev/null)"
    ((skipped++))
else
    info "Rust not found — installing via rustup..."
    if command -v rustup >/dev/null 2>&1; then
        rustup toolchain install stable 2>&1
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1
    fi
    # Source cargo env
    if [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"

    if command -v rustc >/dev/null 2>&1; then
        ok "Rust installed — $(rustc --version 2>/dev/null)"
        ((installed++))
    else
        fail "Rust installation failed. Install manually: https://rustup.rs"
        ((failed++))
    fi
fi

# ========================================================================
# 2. C build tools (cc / build-essential / Xcode CLT)
# ========================================================================
header "2. C Build Tools (cc + linker)"

if command -v cc >/dev/null 2>&1; then
    ok "cc already available — $(cc --version 2>/dev/null | head -1)"
    ((skipped++))
else
    info "C compiler not found — installing build tools..."
    case "$OS_TYPE" in
        macos)
            # Xcode Command Line Tools
            if xcode-select -p >/dev/null 2>&1; then
                ok "Xcode CLT already installed"
            else
                info "Installing Xcode Command Line Tools..."
                xcode-select --install 2>/dev/null || true
                info "If GUI prompt appeared, complete it and re-run this script."
            fi
            ;;
        linux)
            install_pkg "build tools" "" "build-essential" "gcc gcc-c++ make" "base-devel" && ((installed++)) || {
                warn "Could not auto-install C build tools. Please install gcc/clang + make manually."
                ((failed++))
            }
            ;;
    esac
fi

# ========================================================================
# 3. LLVM / clang
# ========================================================================
header "3. LLVM / clang (required to compile IR to native binary)"

if command -v clang >/dev/null 2>&1; then
    ok "already installed — $(clang --version 2>/dev/null | head -1)"
    ((skipped++))
else
    info "clang not found — installing..."
    installed_clang=false

    # Try package manager
    case "$OS_TYPE/$DISTRO" in
        macos/*)
            install_pkg "llvm" "llvm" "" "" "" && installed_clang=true
            ;;
        linux/debian|linux/ubuntu|linux/linuxmint|linux/pop)
            install_pkg "clang" "" "clang lld" "" "" && installed_clang=true
            ;;
        linux/fedora|linux/rhel|linux/centos)
            install_pkg "clang" "" "" "clang lld" "" && installed_clang=true
            ;;
        linux/arch|linux/manjaro)
            install_pkg "clang" "" "" "" "clang lld" && installed_clang=true
            ;;
        linux/alpine)
            install_pkg "clang" "" "clang lld" "" "" && installed_clang=true
            ;;
        *)
            warn "Unknown distro — attempting rustup + manual LLVM"
            ;;
    esac

    # Fallback: download pre-built LLVM binary
    if ! $installed_clang && ! command -v clang >/dev/null 2>&1; then
        LLVM_VER="19.1.0"
        info "Downloading LLVM $LLVM_VER pre-built binary..."
        case "$(uname -m)" in
            x86_64|amd64)  ARCH="x86_64" ;;
            aarch64|arm64) ARCH="aarch64" ;;
            *)             ARCH="x86_64" ;;
        esac
        case "$OS_TYPE" in
            macos)   LLVM_TAR="clang+llvm-${LLVM_VER}-${ARCH}-apple-darwin.tar.xz" ;;
            linux)   LLVM_TAR="clang+llvm-${LLVM_VER}-${ARCH}-linux-gnu-ubuntu-24.04.tar.xz" ;;
        esac
        LLVM_URL="https://github.com/llvm/llvm-project/releases/download/llvmorg-${LLVM_VER}/${LLVM_TAR}"
        LLVM_DIR="$HOME/.local/llvm-${LLVM_VER}"

        mkdir -p "$LLVM_DIR"
        curl -L "$LLVM_URL" -o "/tmp/${LLVM_TAR}" --progress-bar
        tar -xf "/tmp/${LLVM_TAR}" -C "$LLVM_DIR" --strip-components=1
        rm -f "/tmp/${LLVM_TAR}"

        # Add to PATH for this session and profile
        export PATH="$LLVM_DIR/bin:$PATH"
        if ! grep -q "llvm-${LLVM_VER}" "$HOME/.bashrc" 2>/dev/null; then
            echo "export PATH=\"$LLVM_DIR/bin:\$PATH\"" >> "$HOME/.bashrc"
        fi
        if [ -f "$HOME/.zshrc" ] && ! grep -q "llvm-${LLVM_VER}" "$HOME/.zshrc" 2>/dev/null; then
            echo "export PATH=\"$LLVM_DIR/bin:\$PATH\"" >> "$HOME/.zshrc"
        fi

        if command -v clang >/dev/null 2>&1; then
            ok "LLVM installed to $LLVM_DIR"
            ((installed++))
            installed_clang=true
        fi
    fi

    if $installed_clang || command -v clang >/dev/null 2>&1; then
        ((installed++))
    else
        fail "Could not install LLVM/clang."
        info "  Manual install: https://github.com/llvm/llvm-project/releases"
        info "  NOTE: Without clang, the compiler emits .ll IR files but cannot link native binaries."
        info "  The compiler itself (axiomc) does not require clang to compile AXIOM source to IR."
        ((failed++))
    fi
fi

# ========================================================================
# 4. Git (optional)
# ========================================================================
header "4. Git (optional — for package manager)"

if command -v git >/dev/null 2>&1; then
    ok "already installed — $(git --version 2>/dev/null)"
    ((skipped++))
else
    info "Git not found — optional, only needed for 'axiom pkg install'"
    install_pkg "git" && ((installed++)) || {
        warn "Could not auto-install Git (non-critical)."
    }
fi

# ========================================================================
# Summary
# ========================================================================
echo ""
echo -e "  ${MAGENTA}========================================${NC}"
echo -e "  ${MAGENTA}Dependency installation complete${NC}"
echo -e "  ${MAGENTA}========================================${NC}"
echo ""
echo -e "  ${GREEN}Installed : $installed${NC}"
echo -e "  ${GRAY}Skipped   : $skipped  (already present)${NC}"
if [ "$failed" -gt 0 ]; then
    echo -e "  ${RED}Failed    : $failed${NC}"
else
    echo -e "  ${GRAY}Failed    : 0${NC}"
fi
echo ""

if [ "$installed" -gt 0 ]; then
    echo -e "  ${YELLOW}Restart your terminal or run 'source ~/.bashrc' for PATH changes.${NC}"
    echo -e "  ${YELLOW}Then run: ./install.sh${NC}"
elif [ "$failed" -eq 0 ]; then
    echo -e "  ${GREEN}All dependencies present. Ready to install AXIOM:${NC}"
    echo -e "  ${CYAN}Run: ./install.sh${NC}"
else
    echo -e "  ${RED}Some dependencies could not be installed automatically.${NC}"
    echo -e "  ${GRAY}Install them manually, then run: ./install.sh${NC}"
fi

echo ""
exit 0
