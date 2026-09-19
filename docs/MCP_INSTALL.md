<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM MCP Server -- Installation Guide

> **What is this?** `xiom-mcp` is a Model Context Protocol server that gives ANY AI coding agent
> deep XIOM compiler integration: compile + structured diagnostics, error explanations, contract
> queries, syntax checking, formatting, safety audits, and a language cheatsheet.
>
> **LLM compatibility:** MCP is a client-side protocol. The LLM never talks to the server directly --
> your agent does. Any MCP-capable agent works regardless of the LLM behind it (Claude, GPT,
> Gemini, DeepSeek, Qwen, local models). No API keys needed by the server.

---

## 1. Get the Binary

### Option A: From a release package (recommended for users)

Download the latest release zip. It ships **all nine CLI tools** --
`xiom`, `xiom-pkg`, `xiom-fmt`, `xiom-doc`, `xiom-lsp`, `xiom-dbg`,
`xiom-mcp`, `xiom-verify`, `xiom-ffigen` -- plus the pinned **Z3 solver**
(`bin/z3.exe` / `bin/z3`, license `LICENSE-Z3`) used by `xiom --verify`.
The MCP binary is at `bin/xiom-mcp.exe` (Windows) or `bin/xiom-mcp`
(Linux/macOS); run it with `--version` to confirm the build.

### Option B: Build from source (for contributors)

```bash
git clone https://github.com/XIOM-lang/XIOM.git 
cd xiom
cargo build --release -p xiom-mcp
# Binary: target/release/xiom-mcp(.exe)
```

**Important:** The tools `compile_and_analyze`, `get_contract_signature`, and `check_xiom_syntax`
link the compiler directly (library mode -- no subprocess). The tools `format_xiom_code` and
`audit_safety_sandbox` spawn `xiom-fmt` / `xiom`, so keep all binaries in the same directory
or on `PATH`.

---

## 2. The 7 Tools

| Tool | What it does | Needs file on disk? |
|------|-------------|---------------------|
| `xiom_cheatsheet` | Canonical XIOM patterns: functions, structs, enums, contracts, FFI, generics, ownership, stdlib | No |
| `check_xiom_syntax` | Fast parse+check of source snippet | No (takes source string) |
| `compile_and_analyze` | Full compile with structured JSON diagnostics | Yes |
| `explain_error_code` | Error code reference (T001, P001, L001, E001...) | No |
| `get_contract_signature` | Query requires/ensures/invariants of a function/type | Yes |
| `format_xiom_code` | Canonical formatting | No (takes source string) |
| `audit_safety_sandbox` | Unsafe-block audit with severity scoring for CI/CD | Yes |

**Recommended agent workflow:** `xiom_cheatsheet` (learn patterns) -> write code ->
`check_xiom_syntax` (fast validation) -> `compile_and_analyze` (full check) ->
`audit_safety_sandbox` (before commit).

---

## 3. Client Configurations

### Kilo CLI / Kilo Code (project `kilo.json`)

```jsonc
{
  "$schema": "https://app.kilo.ai/config.json",
  "mcp": {
    "xiom": {
      "type": "local",
      "command": ["C:\\path\\to\\xiom-mcp.exe"],
      "enabled": true,
      "timeout": 30000
    }
  },
  "permission": {
    // Auto-allow safe read-only tools; ask for the rest
    "xiom_xiom_cheatsheet": "allow",
    "xiom_check_xiom_syntax": "allow",
    "xiom_explain_error_code": "allow",
    "xiom_*": "ask"
  }
}
```

Place at project root (`./kilo.json`) or globally (`~/.config/kilo/kilo.json`).

### Claude Desktop (`claude_desktop_config.json`)

Windows: `%APPDATA%\Claude\claude_desktop_config.json`
macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": []
    }
  }
}
```

### Claude Code (`.mcp.json` at project root)

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": []
    }
  }
}
```

Or via CLI: `claude mcp add xiom -- C:\path\to\xiom-mcp.exe`

### Cursor (`.cursor/mcp.json` at project root or `~/.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": []
    }
  }
}
```

### Windsurf (`~/.codeium/windsurf/mcp_config.json`)

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": []
    }
  }
}
```

### Cline / Roo Code (VS Code extension MCP settings)

Open the extension's MCP settings and add:

```json
{
  "mcpServers": {
    "xiom": {
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": [],
      "alwaysAllow": ["xiom_cheatsheet", "check_xiom_syntax", "explain_error_code"]
    }
  }
}
```

### VS Code GitHub Copilot (agent mode, `.vscode/mcp.json`)

```json
{
  "servers": {
    "xiom": {
      "type": "stdio",
      "command": "C:\\path\\to\\xiom-mcp.exe",
      "args": []
    }
  }
}
```

---

## 4. Verify Installation

From a terminal, pipe an initialize request:

```powershell
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | & "C:\path\to\xiom-mcp.exe"
```

Expected: two JSON lines -- `serverInfo.name: "xiom-mcp"` and a `tools` array with 7 entries.

In your agent, ask: *"Use the xiom cheatsheet tool to show me how to write a XIOM function with contracts."*
The agent should call `xiom_cheatsheet {section: "contracts"}` and get canonical patterns back.

---

## 5. Distribution Model

| Channel | How users get it |
|---------|------------------|
| **Release zip** | `bin/xiom-mcp.exe` ships alongside `xiom.exe` -- one download, everything included |
| **Source build** | `cargo build --release -p xiom-mcp` |
| **Future: registry** | `xiom pkg install xiom-mcp` (planned, registry.xiom-lang.com) |

The MCP server is **bundled with the compiler toolchain** -- not a separate install. If you have
XIOM, you have the MCP server. Agents just need the JSON config pointing at the binary.

### Why not auto-install into agents?

Each agent has its own config location and security model (permissions, allowlists). We ship
copy-paste configs (section 3) instead of writing into agent configs automatically -- that would
be invasive and fragile across agent updates.

---

## 6. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Agent shows no xiom tools | Check the binary path in config is absolute and exists |
| `format_xiom_code` fails | Put `xiom-fmt(.exe)` in the same dir as `xiom-mcp` or on PATH |
| `audit_safety_sandbox` fails | Put `xiom(.exe)` in the same dir or on PATH |
| Tools error "File not found" | Paths are relative to the agent's working directory -- use absolute paths |
| "Path traversal rejected" | The server blocks `..` in paths by design (security) |
| Server exits immediately | Run the verify command (section 4) to see raw errors |
