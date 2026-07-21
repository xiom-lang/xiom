#!/bin/bash
# ============================================================================
# XIOM Compiler v0.49.5 â€” Cross-Platform Installer (Linux/macOS)
# ============================================================================
set -e

VERSION="0.49.5"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# â”€â”€ ASCII Art â”€â”€
cat << 'EOF'

  |\  /|  |\    /|  |\     /|  |\  /|  v0.49.5
  | \/ |  | \  / |  | \   / |  | \/ |  "Phoenix"
  |    |  |  \/  |  |  \ /  |  |    |  Industrial Compiler
  |    |  |      |  |       |  |    |  Production Release

  ================================================================
    XIOM COMPILER â€” Lightning-fast systems programming language.
    Zero-cost abstractions, contract verification, hot reload.
    Built for games, engines, embedded, and high-performance apps.
  ================================================================

  Version:  v0.49.5 (920/920 (673+247) - Z3, MCP, LSP, Hot Reload, AI)
  Runtime:  Requires CLANG/LLVM for native compilation

EOF

# â”€â”€ Detect platform â”€â”€
case "$(uname -s)" in
  Linux*)  PLATFORM="linux"   ; DEFAULT_INSTALL="$HOME/.local/share/xiom" ;;
  Darwin*) PLATFORM="macos"   ; DEFAULT_INSTALL="$HOME/Library/Application Support/xiom" ;;
  *)       echo "Unsupported platform: $(uname -s)"; exit 1 ;;
esac

# â”€â”€ Choose install directory â”€â”€
echo "  [1/5] Install directory"
read -p "  Location [$DEFAULT_INSTALL]: " XIOM_DIR
XIOM_DIR="${XIOM_DIR:-$DEFAULT_INSTALL}"
XIOM_BIN="$XIOM_DIR/bin"
XIOM_MCP="$XIOM_DIR/mcp"

echo ""
echo "  Installing to $XIOM_DIR..."
mkdir -p "$XIOM_BIN" "$XIOM_DIR/lib" "$XIOM_DIR/runtime" "$XIOM_MCP"

# â”€â”€ Copy binaries â”€â”€
echo "  [2/5] Copying binaries..."
if [ -d "$SCRIPT_DIR/bin" ]; then
  cp "$SCRIPT_DIR/bin/"* "$XIOM_BIN/" 2>/dev/null || true
  echo "    + xiom, xiom-fmt, xiom-doc, xiom-ffigen"
  echo "    + xiom-pkg, xiom-lsp, xiom-mcp"
  echo "    + xiom-dbg, xiom-verify, z3"
fi

# â”€â”€ Copy stdlib â”€â”€
echo "  [3/5] Copying standard library..."
cp -r "$SCRIPT_DIR/lib/"* "$XIOM_DIR/lib/" 2>/dev/null || true
echo "    + Standard library installed"

# â”€â”€ Copy runtime â”€â”€
cp -r "$SCRIPT_DIR/runtime/"* "$XIOM_DIR/runtime/" 2>/dev/null || true
echo "    + Runtime installed"

# â”€â”€ Copy MCP configs â”€â”€
echo "  [4/5] Setting up MCP configurations..."
if [ -d "$SCRIPT_DIR/mcp" ]; then
  cp -r "$SCRIPT_DIR/mcp/"* "$XIOM_MCP/" 2>/dev/null || true
fi

# Create MCP config with actual install path
cat > "$XIOM_MCP/xiom-mcp-config.json" << MCPEOF
{
  "xiom": {
    "command": "$XIOM_BIN/xiom-mcp",
    "args": [],
    "env": {
      "XIOM_HOME": "$XIOM_DIR"
    }
  }
}
MCPEOF
echo "    + MCP configs installed to $XIOM_MCP"

# â”€â”€ AI Configuration â”€â”€
echo ""
echo "  [5/5] AI Configuration (optional â€” press Enter to skip)"
echo "  -------------------------------------------------------"
echo "  XIOM integrates with AI providers for error diagnostics."
echo "  Supported: OpenAI, Anthropic, Ollama, DeepSeek, LiteLLM"
echo ""
read -p "  AI Endpoint [skip]: " AI_ENDPOINT
if [ -n "$AI_ENDPOINT" ]; then
  read -sp "  AI API Key [skip]: " AI_KEY; echo ""
  read -p "  AI Model [gpt-4o]: " AI_MODEL; AI_MODEL="${AI_MODEL:-gpt-4o}"
  read -p "  AI Provider [openai]: " AI_PROVIDER; AI_PROVIDER="${AI_PROVIDER:-openai}"

  cat > "$XIOM_DIR/.xiom_ai_config" << AIEOF
# XIOM AI Configuration
XIOM_AI_PROVIDER=$AI_PROVIDER
XIOM_AI_ENDPOINT=$AI_ENDPOINT
XIOM_AI_MODEL=$AI_MODEL
XIOM_AI_TIMEOUT=30
XIOM_AI_CACHE_DIR=$XIOM_DIR

# MCP Server â€” start with: xiom-mcp
# The MCP server provides AI-assisted diagnostics.
# See $XIOM_MCP/ for integration guides.
AIEOF
  [ -n "$AI_KEY" ] && echo "XIOM_AI_API_KEY=$AI_KEY" >> "$XIOM_DIR/.xiom_ai_config"
  echo "    + AI configuration saved"
fi

# â”€â”€ Create xiom wrapper script â”€â”€
cat > "$XIOM_BIN/xiom" << 'WRAPPER'
#!/bin/bash
XIOM_BIN="__XIOM_BIN__"
XIOM_HOME="__XIOM_HOME__"
case "${1:-}" in
  compile) shift; exec "$XIOM_BIN/xiom" "$@" ;;
  run)     shift; exec "$XIOM_BIN/xiom" --run "$@" ;;
  build)   shift; exec "$XIOM_BIN/xiom" build "$@" ;;
  test)    shift; exec "$XIOM_BIN/xiom" --test "$@" ;;
  fmt)     shift; exec "$XIOM_BIN/xiom-fmt" "$@" ;;
  doc)     shift; exec "$XIOM_BIN/xiom-doc" "$@" ;;
  ffigen)  shift; exec "$XIOM_BIN/xiom-ffigen" "$@" ;;
  pkg)     shift; exec "$XIOM_BIN/xiom-pkg" "$@" ;;
  lsp)     shift; exec "$XIOM_BIN/xiom-lsp" "$@" ;;
  mcp)     shift; exec "$XIOM_BIN/xiom-mcp" "$@" ;;
  verify)  shift; exec "$XIOM_BIN/xiom-verify" "$@" ;;
  dbg)     shift; exec "$XIOM_BIN/xiom-dbg" "$@" ;;
  ai)      shift; exec "$XIOM_BIN/xiom" --ai "$@" ;;
  graph)   shift; exec "$XIOM_BIN/xiom" --graph "$@" ;;
  *)       exec "$XIOM_BIN/xiom" "$@" ;;
esac
WRAPPER

sed -i "s|__XIOM_BIN__|$XIOM_BIN|g" "$XIOM_BIN/xiom"
sed -i "s|__XIOM_HOME__|$XIOM_DIR|g" "$XIOM_BIN/xiom"
chmod +x "$XIOM_BIN/xiom"

# â”€â”€ Add to PATH â”€â”€
echo ""
echo "  PATH Configuration"
echo "  ------------------"
echo "  [S] Shell config  â€” add to ~/.bashrc / ~/.zshrc (recommended)"
echo "  [N] Skip          â€” add manually later"
echo ""
read -p "  Choose [S/n]: " PATH_CHOICE
PATH_CHOICE="${PATH_CHOICE:-S}"

if [ "$PATH_CHOICE" != "n" ] && [ "$PATH_CHOICE" != "N" ]; then
  SHELL_RC=""
  if [ -f "$HOME/.zshrc" ]; then SHELL_RC="$HOME/.zshrc"; fi
  if [ -f "$HOME/.bashrc" ]; then SHELL_RC="$HOME/.bashrc"; fi
  if [ -z "$SHELL_RC" ] && [ -f "$HOME/.profile" ]; then SHELL_RC="$HOME/.profile"; fi

  if [ -n "$SHELL_RC" ]; then
    if ! grep -q "$XIOM_BIN" "$SHELL_RC" 2>/dev/null; then
      echo "" >> "$SHELL_RC"
      echo "# XIOM Compiler v$VERSION" >> "$SHELL_RC"
      echo "export XIOM_HOME=\"$XIOM_DIR\"" >> "$SHELL_RC"
      echo "export PATH=\"$XIOM_BIN:\$PATH\"" >> "$SHELL_RC"
      echo "    + Added to $SHELL_RC"
    else
      echo "    + PATH already configured"
    fi
  fi
fi

# â”€â”€ Create uninstaller â”€â”€
cat > "$XIOM_BIN/uninstall.sh" << UNEOF
#!/bin/bash
echo "XIOM Uninstaller v$VERSION"
echo "This will remove: $XIOM_DIR"
read -p "Continue? [y/N]: " CONFIRM
if [ "\$CONFIRM" != "y" ] && [ "\$CONFIRM" != "Y" ]; then exit 0; fi
rm -rf "$XIOM_DIR"
echo "XIOM removed. Remove from PATH manually if needed."
echo "Check ~/.bashrc or ~/.zshrc for XIOM entries."
UNEOF
chmod +x "$XIOM_BIN/uninstall.sh"

# â”€â”€ Final message â”€â”€
cat << EOF

  =========================================
    XIOM v$VERSION INSTALLED SUCCESSFULLY!
  =========================================

  Location:   $XIOM_DIR
  Binary:     $XIOM_BIN/xiom
  Wrapper:    $XIOM_BIN/xiom
  MCP Config: $XIOM_MCP/xiom-mcp-config.json
EOF

if [ -n "$AI_ENDPOINT" ]; then
  echo "  AI Provider: $AI_PROVIDER  ($AI_ENDPOINT)"
  echo "  AI Config:   $XIOM_DIR/.xiom_ai_config"
  echo ""
fi

cat << EOF

  Quick Start:
    xiom compile hello.xi
    xiom --help

  MCP Integration:
    Copy $XIOM_MCP/xiom-mcp-config.json to your agent's MCP config

  To uninstall: $XIOM_BIN/uninstall.sh
EOF

echo ""
