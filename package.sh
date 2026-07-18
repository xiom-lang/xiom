#!/usr/bin/env bash
# XIOM Release Packager (Linux / macOS)
# Builds all tools in release mode and packages into distributable folder + tar.gz
#
# Usage:
#   ./package.sh
#   ./package.sh 0.47.0

set -euo pipefail

VERSION="${1:-0.46.0}"
ROOT="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$ROOT/release"
PKG_DIR="$RELEASE_DIR/xiom-v$VERSION"
BIN_DIR="$PKG_DIR/bin"
LIB_DIR="$PKG_DIR/lib"
RT_DIR="$PKG_DIR/runtime"

echo ""
echo "  XIOM Release Packager v$VERSION"
echo "  ================================"
echo ""

# Build all tools
TOOLS=("xiomc" "xiom-fmt" "xiom-doc" "xiom-ffigen" "xiom-pkg" "xiom-lsp")
BUILT_OK=()
for tool in "${TOOLS[@]}"; do
    echo "  Building $tool..."
    if cargo build -p "$tool" --release 2>/dev/null; then
        BUILT_OK+=("$tool")
        echo "    [OK] $tool"
    else
        echo "    [SKIP] $tool (crate not found or build failed)"
    fi
done

# Create release directories
mkdir -p "$BIN_DIR" "$LIB_DIR" "$RT_DIR"

# Copy binaries
echo ""
echo "  Packaging release..."
for tool in "${BUILT_OK[@]}"; do
    src="$ROOT/target/release/$tool"
    if [ -f "$src" ]; then
        cp "$src" "$BIN_DIR/$tool"
        echo "    + $tool"
    fi
done

# Copy icon
if [ -f "$ROOT/resource/img/xiom-icon.ico" ]; then
    cp "$ROOT/resource/img/xiom-icon.ico" "$BIN_DIR/xiom-icon.ico"
    echo "    + xiom-icon.ico"
fi

# Copy stdlib + runtime
if [ -d "$ROOT/stdlib" ]; then
    cp -r "$ROOT/stdlib/"* "$LIB_DIR/"
    echo "    + stdlib/ -> lib/"
fi
if [ -f "$ROOT/stdlib/runtime/xiom_runtime.c" ]; then
    cp "$ROOT/stdlib/runtime/xiom_runtime.c" "$RT_DIR/"
    echo "    + xiom_runtime.c -> runtime/"
fi

# Copy documentation (optional)
if [ -d "$ROOT/docs/language/html" ]; then
    mkdir -p "$PKG_DIR/docs"
    cp -r "$ROOT/docs/language/html/"* "$PKG_DIR/docs/"
    echo "    + docs/ (API reference)"
fi

# Create install.sh
cat > "$PKG_DIR/install.sh" << 'INSTALLEOF'
#!/usr/bin/env bash
# XIOM Installer (Linux / macOS)
set -euo pipefail

XIOM_DEFAULT="$HOME/.local/share/xiom"
echo ""
echo "  XIOM Compiler Installer"
echo "  ======================="
echo ""
read -p "  Install directory [$XIOM_DEFAULT]: " XIOM_DIR
XIOM_DIR="${XIOM_DIR:-$XIOM_DEFAULT}"
XIOM_BIN="$XIOM_DIR/bin"

echo ""
echo "  Installing to $XIOM_DIR..."
mkdir -p "$XIOM_BIN"

# Copy binaries
cp "$(dirname "$0")/bin/"* "$XIOM_BIN/" 2>/dev/null || true
chmod +x "$XIOM_BIN/"*
echo "    + Binaries installed"

# Copy stdlib
if [ -d "$(dirname "$0")/lib" ]; then
    cp -r "$(dirname "$0")/lib/"* "$XIOM_DIR/lib/"
    echo "    + Standard library installed"
fi

# Copy runtime
if [ -d "$(dirname "$0")/runtime" ]; then
    cp -r "$(dirname "$0")/runtime/"* "$XIOM_DIR/runtime/"
    echo "    + Runtime installed"
fi

# Copy docs
if [ -d "$(dirname "$0")/docs" ]; then
    cp -r "$(dirname "$0")/docs/"* "$XIOM_DIR/docs/"
    echo "    + Documentation installed"
fi

# Create xiom wrapper
cat > "$XIOM_BIN/xiom" << 'WRAPEOF'
#!/usr/bin/env bash
XIOM_BIN="$(dirname "$(readlink -f "$0")")"
if [ $# -eq 0 ]; then
    exec "$XIOM_BIN/xiomc" --help
fi
case "$1" in
    compile) shift; exec "$XIOM_BIN/xiomc" "$@" ;;
    fmt)     shift; exec "$XIOM_BIN/xiom-fmt" "$@" ;;
    doc)     shift; exec "$XIOM_BIN/xiom-doc" "$@" ;;
    ffigen)  shift; exec "$XIOM_BIN/xiom-ffigen" "$@" ;;
    pkg)     shift; exec "$XIOM_BIN/xiom-pkg" "$@" ;;
    lsp)     shift; exec "$XIOM_BIN/xiom-lsp" "$@" ;;
    *)       exec "$XIOM_BIN/xiomc" "$@" ;;
esac
WRAPEOF
chmod +x "$XIOM_BIN/xiom"

# Add to PATH
echo ""
echo "  PATH options:"
echo "    [U] User shell profile   (default)"
echo "    [S] System-wide /usr/local/bin"
echo "    [N] Skip"
echo ""
read -p "  Choose [U/s/N]: " PATH_TYPE
PATH_TYPE="${PATH_TYPE:-U}"

case "$PATH_TYPE" in
    [Ss])
        sudo ln -sf "$XIOM_BIN/xiom" /usr/local/bin/xiom 2>/dev/null || true
        for tool in xiomc xiom-fmt xiom-doc xiom-ffigen xiom-pkg xiom-lsp; do
            sudo ln -sf "$XIOM_BIN/$tool" "/usr/local/bin/$tool" 2>/dev/null || true
        done
        echo "    + Symlinks created in /usr/local/bin"
        ;;
    [Nn])
        echo "    + Skipped PATH setup"
        ;;
    *)
        PROFILE="$HOME/.bashrc"
        if [ -f "$HOME/.zshrc" ]; then PROFILE="$HOME/.zshrc"; fi
        if [ -f "$HOME/.config/fish/config.fish" ]; then PROFILE="$HOME/.config/fish/config.fish"; fi
        if ! grep -q "$XIOM_BIN" "$PROFILE" 2>/dev/null; then
            echo "export PATH=\"$XIOM_BIN:\$PATH\"" >> "$PROFILE"
        fi
        echo "    + Added to $PROFILE (restart shell or run: source $PROFILE)"
        ;;
esac

# Create uninstaller
cat > "$XIOM_BIN/uninstall.sh" << 'UNEOF'
#!/usr/bin/env bash
echo "XIOM Uninstaller"
echo ""
read -p "This will remove $(dirname "$(dirname "$(readlink -f "$0")")"). Continue? [y/N]: " CONFIRM
if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then exit 0; fi
XIOM_DIR="$(dirname "$(dirname "$(readlink -f "$0")")")"
rm -rf "$XIOM_DIR"
echo "XIOM removed. Remove from PATH manually if needed."
UNEOF
chmod +x "$XIOM_BIN/uninstall.sh"

echo ""
echo "  ========================================="
echo "  XIOM installed successfully!"
echo "  ========================================="
echo ""
echo "  Location:  $XIOM_DIR"
echo "  Binary:    $XIOM_BIN/xiomc"
echo "  Wrapper:   $XIOM_BIN/xiom"
echo ""
echo "  Quick start:"
echo "    xiom compile hello.xi"
echo "    xiom --help"
echo ""
if ! command -v clang &>/dev/null && ! command -v clang-18 &>/dev/null; then
    echo "  NOTE: clang not found. XIOM emits LLVM IR; clang compiles to native."
    echo "  Install: sudo apt install clang-18  (Ubuntu/Debian)"
    echo "           brew install llvm          (macOS)"
fi
echo ""
echo "  To uninstall: $XIOM_BIN/uninstall.sh"
echo ""
INSTALLEOF
chmod +x "$PKG_DIR/install.sh"
echo "    + install.sh"

# Create README
cat > "$PKG_DIR/README.txt" << READMEEOF
XIOM v$VERSION - Portable Release
===================================

Quick install:
  Run: ./install.sh
  This copies XIOM to ~/.local/share/xiom and adds it to PATH.

Manual install:
  1. Copy this entire folder anywhere you like
  2. Add the bin/ folder to your system PATH
  3. Run: xiomc --help

Contents:
  bin/       - xiomc, xiom-fmt, xiom-doc, etc.
  lib/       - Standard library (.xi source files)
  runtime/   - C runtime (xiom_runtime.c)
  install.sh - Installer script

Need dependencies? Install LLVM/clang for native compilation.
READMEEOF
echo "    + README.txt"

# Create tar.gz
TAR_NAME="xiom-v$VERSION-linux-x64.tar.gz"
TAR_PATH="$RELEASE_DIR/$TAR_NAME"
cd "$RELEASE_DIR"
tar -czf "$TAR_NAME" "xiom-v$VERSION"
echo ""
echo "  Release packaged:"
echo "    Folder: $PKG_DIR"
echo "    TAR:    $TAR_PATH"
echo ""
