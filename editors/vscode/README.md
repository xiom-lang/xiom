# XIOM Language Support for VS Code

Syntax highlighting, autocomplete, go-to-definition, diagnostics, and **debugging** for the XIOM programming language.

## Features

- **Syntax highlighting** — keywords, types, operators, contracts, strings, comments
- **Bracket matching** — {}, [], (), ""
- **Line/block comments** — // and /* */
- **LSP integration** — diagnostics, hover, completion, go-to-definition, references, rename (requires xiom-lsp binary)
- **DAP debugging** — breakpoints, step, variables, contract-violation traps (requires xiom-dbg binary + GDB)

## Test the Extension (Development)

1. Build the toolchain from repo root:
   ```powershell
   cargo build --release -p xiom-lsp -p xiom-dbg
   ```
2. Open the `editors/vscode` folder in VS Code.
3. Press **F5** ("Run Extension") — a new Extension Development Host window opens.
4. In the dev host, open the XIOM repo folder and any `.xi` file:
   - Syntax highlighting works immediately.
   - LSP features activate if `target/release/xiom-lsp.exe` exists (auto-detected).
5. Debugging: press **F5** in the dev host on a `.xi` project → select "XIOM Debugger" →
   it launches `xiom-dbg` against your compiled program (default `${workspaceFolder}/a.exe`).
   Compile first with `xiom -g -o a.exe program.xi` for DWARF symbols. GDB must be on PATH.

## Package as .vsix (Distribution)

```bash
npm install -g @vscode/vsce
cd editors/vscode
vsce package
# Produces xiom-0.11.0.vsix — install via:
#   code --install-extension xiom-0.11.0.vsix
```

## Binary Resolution Order

Both `xiom-lsp` and `xiom-dbg` are found automatically:
1. VS Code setting (`xiom.lsp.path` / `xiom.dbg.path`) — explicit override
2. **`PATH`** — the installed XIOM toolchain (standard for end users; release zip's `bin/` on PATH)
3. `<workspace>/target/release/` and `target/debug/` — compiler developers working in the XIOM repo
4. The extension's own directory (bundled binaries)

## Debug Launch Configuration

`.vscode/launch.json` example:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "xiom",
      "request": "launch",
      "name": "Debug XIOM Program",
      "program": "${workspaceFolder}/a.exe",
      "stopOnEntry": true,
      "contractTraps": true
    }
  ]
}
```

## License

MIT OR Apache-2.0
