# XIOM for JetBrains IDEs (IntelliJ IDEA, CLion, RustRover, PyCharm...)

Integration via **LSP4IJ** (free, Red Hat) — no custom plugin needed.
Requires `xiom-lsp` (and `xiom-dbg` for debugging) on PATH.

## Setup (5 minutes)

1. **Install LSP4IJ**: Settings → Plugins → Marketplace → search "LSP4IJ" → Install.
2. **Register the file type**: Settings → Editor → File Types → "Recognized File Types" →
   add pattern `*.xi` to *Textmate* or create a "XIOM" type. For highlighting, add the
   TextMate bundle: Settings → Editor → TextMate Bundles → `+` → select the
   `editors/vscode` directory from the XIOM repo/release (it reads the grammar inside).
3. **Add the language server**: View → Tool Windows → LSP Consoles → `+` New Language Server:
   - **Server → Command**: `xiom-lsp`
   - **Mappings → File name patterns**: `*.xi`, language id `xiom`
4. Open a `.xi` file — diagnostics, hover, completion, go-to-definition, references,
   and rename now work.

## Debugging (LSP4IJ 0.8+ / DAP plugin)

LSP4IJ ships a Debug Adapter Protocol client:
1. Run → Edit Configurations → `+` → "Debug Adapter Protocol".
2. **Command**: `xiom-dbg`
3. **Parameters (launch)**:
   ```json
   { "program": "$PROJECT_DIR$/a.exe", "stopOnEntry": true, "contractTraps": true }
   ```
4. Compile with symbols first: `xiomc -g -o a.exe main.xi`, set breakpoints, Debug.

## Native plugin?

A dedicated JetBrains plugin (custom highlighting via lexer, structure view, run
configurations) is deferred — LSP4IJ covers the productive workflow. If demand grows,
the plugin skeleton belongs in this folder.
