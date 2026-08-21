#!/usr/bin/env bash
# ============================================================================
# XIOM Compiler Installer -- macOS & Linux
# ============================================================================
# Installs the XIOM toolchain: xiom, xiom fmt, xiom doc (and aux tools)
#
# Two modes:
#   Source build:  ./install.sh                  (auto-installs deps first)
#   Pre-built:     ./install.sh /path/to/release  (skips build)
#
# Environment:
#   XIOM_INSTALL_DIR   Override install directory (default: ~/.local/xiom)
#   XIOM_SKIP_DEPS     Set to 1 to skip dependency auto-install
# ============================================================================

set -euo pipefail

XIOM_VERSION="0.46.0"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_PATH="${1:-}"

# -- Install directory ---------------------------------------------------
if [ -n "${XIOM_INSTALL_DIR:-}" ]; then
    INSTALL_DIR="$XIOM_INSTALL_DIR"
elif [ "$(uname -s)" = "Darwin" ]; then
    INSTALL_DIR="$HOME/.local/xiom"
else
    INSTALL_DIR="$HOME/.local/xiom"
fi
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
RUNTIME_DIR="$INSTALL_DIR/runtime"

# -- Colors --------------------------------------------------------------
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; MAGENTA='\033[0;35m'; NC='\033[0m'

# -- Banner --------------------------------------------------------------
clear 2>/dev/null || true
echo ""
echo -e "  ${MAGENTA} #####+ ##+  ##+##+ ######+ ###+   ###+${NC}"
echo -e "  ${MAGENTA}##+==##++##+##++##|##+===##+####+ ####|${NC}"
echo -e "  ${MAGENTA}#######| +###++ ##|##|   ##|##+####+##|${NC}"
echo -e "  ${MAGENTA}##+==##| ##+##+ ##|##|   ##|##|+##++##|${NC}"
echo -e "  ${MAGENTA}##|  ##|##++ ##+##|+######++##| +=+ ##|${NC}"
echo -e "  ${MAGENTA}+=+  +=++=+  +=++=+ +=====+ +=+     +=+${NC}"
echo ""
echo -e "  ${CYAN}XIOM Compiler v${XIOM_VERSION}${NC}"
echo -e "  ${CYAN}Safe, Verified, Precise -- Systems Programming${NC}"
echo ""

# -- Step 0: Auto-install dependencies -----------------------------------
if [ -z "$BINARY_PATH" ] && [ "${XIOM_SKIP_DEPS:-0}" != "1" ]; then
    DEPS_SCRIPT="$SCRIPT_DIR/install_deps.sh"
    if [ -f "$DEPS_SCRIPT" ]; then
        echo -e "${CYAN}Auto-installing missing dependencies...${NC}"
        bash "$DEPS_SCRIPT"
    fi
fi

# -- Step 1: Build or use pre-built --------------------------------------
if [ -n "$BINARY_PATH" ]; then
    echo -e "${CYAN}Installing pre-built binaries from: $BINARY_PATH${NC}"
    RELEASE_DIR="$BINARY_PATH"
    if [ ! -d "$BINARY_PATH" ]; then
        echo -e "${RED}ERROR: Build path does not exist: $BINARY_PATH${NC}"
        exit 1
    fi
else
    echo -e "${CYAN}Building XIOM toolchain (release mode)...${NC}"
    cd "$SCRIPT_DIR"

    TOOLS=("xiom" "xiom-fmt" "xiom-doc" "xiom-ffigen" "xiom-pkg" "xiom-lsp")
    for tool in "${TOOLS[@]}"; do
        echo "  Building $tool..."
        cargo build -p "$tool" --release 2>/dev/null || {
            echo -e "${RED}ERROR: Failed to build $tool${NC}"
            exit 1
        }
    done
    RELEASE_DIR="$SCRIPT_DIR/target/release"
fi

# -- Step 2: Install -----------------------------------------------------
echo ""
echo -e "${CYAN}Installing to $INSTALL_DIR...${NC}"
mkdir -p "$BIN_DIR" "$LIB_DIR" "$RUNTIME_DIR"

# Binaries
for exe in xiom xiom-fmt xiom-doc xiom-ffigen xiom-pkg xiom-lsp; do
    if [ -f "$RELEASE_DIR/$exe" ]; then
        cp "$RELEASE_DIR/$exe" "$BIN_DIR/"
        echo -e "  ${GREEN}[OK]${NC} $exe"
    elif [ -f "$RELEASE_DIR/${exe}.exe" ]; then
        cp "$RELEASE_DIR/${exe}.exe" "$BIN_DIR/$exe"
        echo -e "  ${GREEN}[OK]${NC} $exe"
    fi
done

# Wrapper script (like xiom.bat but for Unix)
cat > "$BIN_DIR/xiom" << 'WRAPPER'
#!/usr/bin/env bash
# XIOM toolchain dispatcher
BIN_DIR="$(cd "$(dirname "$0")" && pwd)"
case "${1:-}" in
    compile) shift; exec "$BIN_DIR/xiom" "$@" ;;
    fmt)     shift; exec "$BIN_DIR/xiom-fmt" "$@" ;;
    doc)     shift; exec "$BIN_DIR/xiom-doc" "$@" ;;
    ffigen)  shift; exec "$BIN_DIR/xiom-ffigen" "$@" ;;
    pkg)     shift; exec "$BIN_DIR/xiom-pkg" "$@" ;;
    lsp)     shift; exec "$BIN_DIR/xiom-lsp" "$@" ;;
    *)       exec "$BIN_DIR/xiom" "$@" ;;
esac
WRAPPER
chmod +x "$BIN_DIR/xiom"

# Stdlib + runtime
if [ -d "$SCRIPT_DIR/stdlib" ]; then
    cp -r "$SCRIPT_DIR/stdlib" "$LIB_DIR/"
    echo -e "  ${GREEN}[OK]${NC} stdlib"
fi
if [ -d "$SCRIPT_DIR/stdlib/runtime" ]; then
    cp -r "$SCRIPT_DIR/stdlib/runtime"/* "$RUNTIME_DIR/"
    echo -e "  ${GREEN}[OK]${NC} runtime"
fi

# Icon (if present)
if [ -f "$SCRIPT_DIR/resource/img/xiom-icon.ico" ]; then
    cp "$SCRIPT_DIR/resource/img/xiom-icon.ico" "$BIN_DIR/"
fi

# -- Step 3: PATH --------------------------------------------------------
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
        echo "export PATH=\"$BIN_DIR:\$PATH\"  # XIOM" >> "$SHELL_RC"
        echo -e "  ${GREEN}[OK]${NC} Added to $SHELL_RC"
    else
        echo -e "  ${GREEN}[OK]${NC} Already in $SHELL_RC"
    fi
else
    echo -e "  ${YELLOW}[WARN]${NC} Add this to your shell config manually:"
    echo "     export PATH=\"$BIN_DIR:\$PATH\""
fi

# -- Step 4: Verify ------------------------------------------------------
echo ""
echo -e "${CYAN}Verifying installation...${NC}"

export PATH="$BIN_DIR:$PATH"

if [ -x "$BIN_DIR/xiom" ]; then
    "$BIN_DIR/xiom" --version 2>/dev/null || true
    echo -e "  ${GREEN}[OK]${NC} xiom is ready"
else
    echo -e "  ${RED}[FAIL]${NC} xiom not found in $BIN_DIR"
fi

# -- Done ----------------------------------------------------------------
echo ""
echo -e "  ${MAGENTA}========================================${NC}"
echo -e "  ${MAGENTA}XIOM installed successfully!${NC}"
echo -e "  ${MAGENTA}========================================${NC}"
echo ""
echo -e "  Binary:   ${GREEN}$BIN_DIR/xiom${NC}"
echo -e "  Usage:    ${GREEN}xiom compile file.xi${NC}"
echo -e "            ${GREEN}xiom fmt file.xi${NC}"
echo -e "            ${GREEN}xiom doc .${NC}"
echo ""
echo -e "  ${YELLOW}Run 'source $SHELL_RC' or restart your terminal to use 'xiom'.${NC}"
echo ""
