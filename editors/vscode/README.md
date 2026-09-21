<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Language Support for VS Code

Syntax highlighting, autocomplete, go-to-definition, diagnostics, and **debugging** for the XIOM programming language.

## Features

- **Syntax highlighting** -- keywords, types, operators, contracts, strings, comments
- **Bracket matching** -- {}, [], (), ""
- **Line/block comments** -- // and /* */
- **LSP integration** -- diagnostics, hover, completion, go-to-definition, references, rename (requires xiom-lsp binary)
- **DAP debugging** -- breakpoints, step, variables, contract-violation traps (requires xiom-dbg binary + GDB)

## Test the Extension (Development)

1. Build the toolchain from repo root:
   ```powershell
   cargo build --release -p xiom-lsp -p xiom-dbg
   ```
2. Open the `editors/vscode` folder in VS Code.
3. Press **F5** ("Run Extension") -- a new Extension Development Host window opens.
4. In the dev host, open the XIOM repo folder and any `.xi` file:
   - Syntax highlighting works immediately.
   - LSP features activate if `target/release/xiom-lsp.exe` exists (auto-detected).
5. Debugging: press **F5** in the dev host on a `.xi` project -> select "XIOM Debugger" ->
   it launches `xiom-dbg` against your compiled program (default `${workspaceFolder}/a.exe`).
   Compile first with `xiom -g -o a.exe program.xi` for DWARF symbols. GDB must be on PATH.

## Package as .vsix (Distribution)

The extension ships as ONE universal package -- no platform binaries are
bundled, so the same VSIX works on Windows, Linux and macOS:

```bash
cd editors/vscode
npx --yes @vscode/vsce@3 package --out xiom-vscode-0.12.0.vsix
code --install-extension xiom-vscode-0.12.0.vsix
```

Release publishing is automated: on a `v*` tag, `release.yml` attaches the
VSIX (plus a SHA256SUMS entry) to the GitHub Release and publishes to the
VS Code Marketplace (`VSCE_PAT`) and Open VSX (`OVSX_TOKEN`, covers
VSCodium/Cursor/Gitpod) when those secrets are configured.

## First-Run Toolchain Check

On activation the extension verifies `xiom`, `xiom-lsp` and `xiom-dbg`. When
any is missing it shows a notification with an install button
(<https://xiom-lang.org/install>) and registers `XIOM: Recheck toolchain`.
It never installs anything silently -- the official installer is the single
install path. **Minimum toolchain: v0.61.0** (the LSP/DAP protocol this
extension speaks).

## Binary Resolution Order

`xiom-lsp` and `xiom-dbg` are resolved at runtime in this order:
1. VS Code setting (`xiom.lsp.path` / `xiom.debugAdapterPath`) -- explicit override
2. **`PATH`** -- the installed XIOM toolchain (standard for end users; release zip's `bin/` on PATH)
3. `<workspace>/target/release/` and `target/debug/` -- compiler developers working in the XIOM repo

The old `xiom.dbg.path` setting is still honored as a deprecated alias.

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
