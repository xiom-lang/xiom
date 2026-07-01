#!/usr/bin/env bash
# ============================================================================
# AXIOM Compiler Installer — macOS & Linux
# ============================================================================
# Installs the AXIOM toolchain: axiomc, axiom fmt, axiom doc (and aux tools)
#
# Two modes:
#   Source build:  ./install.sh                  (auto-installs deps first)
#   Pre-built:     ./install.sh /path/to/release  (skips build)
#
# Environment:
#   AXIOM_INSTALL_DIR   Override install directory (default: ~/.local/axiom)
#   AXIOM_SKIP_DEPS     Set to 1 to skip dependency auto-install
# ============================================================================

set -euo pipefail

AXIOM_VERSION="0.20.0"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_PATH="${1:-}"

# ── Install directory ───────────────────────────────────────────────────
if [ -n "${AXIOM_INSTALL_DIR:-}" ]; then
    INSTALL_DIR="$AXIOM_INSTALL_DIR"
elif [ "$(uname -s)" = "Darwin" ]; then
    INSTALL_DIR="$HOME/.local/axiom"
else
    INSTALL_DIR="$HOME/.local/axiom"
fi
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
RUNTIME_DIR="$INSTALL_DIR/runtime"

# ── Colors ──────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; MAGENTA='\033[0;35m'; NC='\033[0m'

# ── Banner ──────────────────────────────────────────────────────────────
clear 2>/dev/null || true
echo ""
echo -e "  ${MAGENTA} █████╗ ██╗  ██╗██╗ ██████╗ ███╗   ███╗${NC}"
echo -e "  ${MAGENTA}██╔══██╗╚██╗██╔╝██║██╔═══██╗████╗ ████║${NC}"
echo -e "  ${MAGENTA}███████║ ╚███╔╝ ██║██║   ██║██╔████╔██║${NC}"
echo -e "  ${MAGENTA}██╔══██║ ██╔██╗ ██║██║   ██║██║╚██╔╝██║${NC}"
echo -e "  ${MAGENTA}██║  ██║██╔╝ ██╗██║╚██████╔╝██║ ╚═╝ ██║${NC}"
echo -e "  ${MAGENTA}╚═╝  ╚═╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝     ╚═╝${NC}"
echo ""
echo -e "  ${CYAN}AXIOM Compiler v${AXIOM_VERSION}${NC}"
echo -e "  ${CYAN}Safe, Verified, Precise — Systems Programming${NC}"
echo ""

# ── Step 0: Auto-install dependencies ───────────────────────────────────
if [ -z "$BINARY_PATH" ] && [ "${AXIOM_SKIP_DEPS:-0}" != "1" ]; then
    DEPS_SCRIPT="$SCRIPT_DIR/install_deps.sh"
    if [ -f "$DEPS_SCRIPT" ]; then
        echo -e "${CYAN}Auto-installing missing dependencies...${NC}"
        bash "$DEPS_SCRIPT"
    fi
fi

# ── Step 1: Build or use pre-built ──────────────────────────────────────
if [ -n "$BINARY_PATH" ]; then
    echo -e "${CYAN}Installing pre-built binaries from: $BINARY_PATH${NC}"
    RELEASE_DIR="$BINARY_PATH"
    if [ ! -d "$BINARY_PATH" ]; then
        echo -e "${RED}ERROR: Build path does not exist: $BINARY_PATH${NC}"
        exit 1
    fi
else
    echo -e "${CYAN}Building AXIOM toolchain (release mode)...${NC}"
    cd "$SCRIPT_DIR"

    TOOLS=("axiomc" "axiom-fmt" "axiom-doc" "axiom-ffigen" "axiom-pkg" "axiom-lsp")
    for tool in "${TOOLS[@]}"; do
        echo "  Building $tool..."
        cargo build -p "$tool" --release 2>/dev/null || {
            echo -e "${RED}ERROR: Failed to build $tool${NC}"
            exit 1
        }
    done
    RELEASE_DIR="$SCRIPT_DIR/target/release"
fi

# ── Step 2: Install ─────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}Installing to $INSTALL_DIR...${NC}"
mkdir -p "$BIN_DIR" "$LIB_DIR" "$RUNTIME_DIR"

# Binaries
for exe in axiomc axiom-fmt axiom-doc axiom-ffigen axiom-pkg axiom-lsp; do
    if [ -f "$RELEASE_DIR/$exe" ]; then
        cp "$RELEASE_DIR/$exe" "$BIN_DIR/"
        echo -e "  ${GREEN}✓${NC} $exe"
    elif [ -f "$RELEASE_DIR/${exe}.exe" ]; then
        cp "$RELEASE_DIR/${exe}.exe" "$BIN_DIR/$exe"
        echo -e "  ${GREEN}✓${NC} $exe"
    fi
done

# Wrapper script (like axiom.bat but for Unix)
cat > "$BIN_DIR/axiom" << 'WRAPPER'
#!/usr/bin/env bash
# AXIOM toolchain dispatcher
BIN_DIR="$(cd "$(dirname "$0")" && pwd)"
case "${1:-}" in
    compile) shift; exec "$BIN_DIR/axiomc" "$@" ;;
    fmt)     shift; exec "$BIN_DIR/axiom-fmt" "$@" ;;
    doc)     shift; exec "$BIN_DIR/axiom-doc" "$@" ;;
    ffigen)  shift; exec "$BIN_DIR/axiom-ffigen" "$@" ;;
    pkg)     shift; exec "$BIN_DIR/axiom-pkg" "$@" ;;
    lsp)     shift; exec "$BIN_DIR/axiom-lsp" "$@" ;;
    *)       exec "$BIN_DIR/axiomc" "$@" ;;
esac
WRAPPER
chmod +x "$BIN_DIR/axiom"

# Stdlib + runtime
if [ -d "$SCRIPT_DIR/stdlib" ]; then
    cp -r "$SCRIPT_DIR/stdlib" "$LIB_DIR/"
    echo -e "  ${GREEN}✓${NC} stdlib"
fi
if [ -d "$SCRIPT_DIR/stdlib/runtime" ]; then
    cp -r "$SCRIPT_DIR/stdlib/runtime"/* "$RUNTIME_DIR/"
    echo -e "  ${GREEN}✓${NC} runtime"
fi

# Icon (if present)
if [ -f "$SCRIPT_DIR/resource/img/axiom-icon.ico" ]; then
    cp "$SCRIPT_DIR/resource/img/axiom-icon.ico" "$BIN_DIR/"
fi

# ── Step 3: PATH ────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}Configuring PATH...${NC}"

SHELL_RC=""
case "${SHELL##*/}" in
    bash) SHELL_RC="$HOME/.bashrc" ;;
    zsh)  SHELL_RC="$HOME/.zshrc" ;;
    fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
esac

if [ -n "$SHELL_RC" ]; then
    if ! grep -q "$BIN_DIR" "$SHELL_RC" 2>/dev/null; then
        echo "export PATH=\"$BIN_DIR:\$PATH\"  # AXIOM" >> "$SHELL_RC"
        echo -e "  ${GREEN}✓${NC} Added to $SHELL_RC"
    else
        echo -e "  ${GREEN}✓${NC} Already in $SHELL_RC"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Add this to your shell config manually:"
    echo "     export PATH=\"$BIN_DIR:\$PATH\""
fi

# ── Step 4: Verify ──────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}Verifying installation...${NC}"

export PATH="$BIN_DIR:$PATH"

if [ -x "$BIN_DIR/axiomc" ]; then
    "$BIN_DIR/axiomc" --version 2>/dev/null || true
    echo -e "  ${GREEN}✓${NC} axiomc is ready"
else
    echo -e "  ${RED}✗${NC} axiomc not found in $BIN_DIR"
fi

# ── Done ────────────────────────────────────────────────────────────────
echo ""
echo -e "  ${MAGENTA}========================================${NC}"
echo -e "  ${MAGENTA}AXIOM installed successfully!${NC}"
echo -e "  ${MAGENTA}========================================${NC}"
echo ""
echo -e "  Binary:   ${GREEN}$BIN_DIR/axiomc${NC}"
echo -e "  Usage:    ${GREEN}axiom compile file.ax${NC}"
echo -e "            ${GREEN}axiom fmt file.ax${NC}"
echo -e "            ${GREEN}axiom doc .${NC}"
echo ""
echo -e "  ${YELLOW}Run 'source $SHELL_RC' or restart your terminal to use 'axiom'.${NC}"
echo ""
