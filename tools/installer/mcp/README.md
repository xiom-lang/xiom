<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM MCP Server -- IDE Integration Guide

The XIOM MCP server provides AI-assisted compiler diagnostics, SMT-based
contract verification, and code formatting to any MCP-compatible IDE or agent.

## Quick Setup

1. Copy the appropriate config file below to your IDE's MCP config directory
2. Replace `XIOM_BIN` and `XIOM_HOME` with your actual install paths
3. Restart your IDE or agent

## Supported IDEs & Agents

| IDE / Agent | Config File | Target Path |
|------------|-------------|-------------|
| **Kilo Code** | `kilo.json` | `.kilocode/mcp.json` |
| **Claude Code** | `claude-code.json` | Run: `claude mcp add xiom --config <path>` |
| **Cursor** | `cursor.json` | `.cursor/mcp.json` |
| **Windsurf** | `windsurf.json` | `.windsurf/mcp.json` |
| **Continue.dev** | `continue.json` | `~/.continue/config.json` (under `mcpServers`) |
| **Cline (VS Code)** | `cline.json` | VS Code settings -> Cline -> MCP Servers |
| **GitHub Copilot** | `github-copilot.json` | `.github/copilot-instructions.md` or MCP config |
| **Aider** | `aider.json` | `.aider.conf.yml` (MCP section) |
| **OpenAI Codex** | `codex.json` | `~/.codex/mcp.json` |
| **Google Antigravity** | `antigravity.json` | `.agent-config/mcp.json` |
| **Trae (ByteDance)** | `trae.json` | `.trae/mcp.json` |
| **VS Code Copilot** | `vscode-copilot.json` | `.vscode/mcp.json` |

## Manual Setup (any MCP-compatible tool)

Add to your tool's MCP servers configuration:

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\Users\\YOU\\AppData\\Local\\xiom\\bin\\xiom-mcp.exe",
      "args": [],
      "env": {
        "XIOM_HOME": "C:\\Users\\YOU\\AppData\\Local\\xiom"
      }
    }
  }
}
```

**Linux/macOS:**
```json
{
  "mcpServers": {
    "xiom": {
      "command": "/home/you/.local/share/xiom/bin/xiom-mcp",
      "args": [],
      "env": {
        "XIOM_HOME": "/home/you/.local/share/xiom"
      }
    }
  }
}
```

## Available MCP Tools

| Tool | Description |
|------|-------------|
| `compile_and_analyze` | Compile .xi file -> structured diagnostics with fix suggestions |
| `compile_and_fix` | Inline compile + AI diagnose -- one-shot error analysis |
| `verify_contracts` | SMT-based contract verification with Z3 counterexamples |
| `format_code` | Format XIOM source code |

## AI Configuration

To enable AI-powered diagnostics, create `~/.xiom_ai_config` (or `%XIOM_HOME%\.xiom_ai_config` on Windows):

```ini
XIOM_AI_PROVIDER=openai
XIOM_AI_ENDPOINT=https://api.openai.com/v1
XIOM_AI_MODEL=gpt-4o
XIOM_AI_API_KEY=sk-...
XIOM_AI_TIMEOUT=30
```

Supported providers: `openai`, `anthropic`, `ollama`, `deepseek`, `litellm`,
`groq`, `together`, `fireworks`, `openrouter`.

## Testing the MCP Server

```bash
# Start the MCP server
xiom mcp

# Test tool invocation (from MCP client)
# The server responds to JSON-RPC method calls
```

## Troubleshooting

- **MCP server not found**: Ensure `xiom-mcp.exe` is in your PATH or use absolute path
- **AI diagnostics not working**: Check `.xiom_ai_config` exists and has valid API key
- **Contract verification fails**: Ensure `z3.exe` is in `XIOM_HOME/bin/`
- **Permission denied on Linux/macOS**: Run `chmod +x ~/.local/share/xiom/bin/xiom-mcp`
