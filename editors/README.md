# XIOM Editor Integrations

XIOM tooling is **protocol-based** — `xiom-lsp` speaks the Language Server Protocol and
`xiom-dbg` speaks the Debug Adapter Protocol. Any editor with LSP/DAP support integrates
with a small config; only VS Code ships a full extension.

**Prerequisite for all editors:** the XIOM toolchain `bin/` directory on `PATH`
(so `xiom-lsp` and `xiom-dbg` resolve). Verify: `xiom-lsp --help`.

## Support Matrix

| Editor | Highlighting | LSP (diagnostics, hover, completion, refs, rename) | Debugging (DAP) | How |
|--------|--------------|-----|-----|-----|
| **VS Code** | ✅ TextMate | ✅ | ✅ | [vscode/](./vscode/) — full extension (.vsix) |
| **Neovim** | ⚠ via LSP semantic info¹ | ✅ | ✅ nvim-dap | [neovim/](./neovim/) — Lua config |
| **JetBrains** (IDEA, CLion, RustRover) | ⚠ plugin textmate bundle | ✅ LSP4IJ | ✅ LSP4IJ 0.8+ | [jetbrains/](./jetbrains/) — LSP4IJ setup |
| **Helix** | ⚠ needs tree-sitter¹ | ✅ | ✅ built-in | [helix/](./helix/) — languages.toml |
| **Sublime Text** | ✅ TextMate² | ✅ LSP package | — | [sublime/](./sublime/) — LSP settings |
| **Emacs** | basic | ✅ eglot | ✅ dape | [emacs/](./emacs/) — elisp snippet |
| **Zed** | ⚠ needs tree-sitter¹ | ✅ (extension API) | — | planned |

¹ Native highlighting in Neovim/Helix/Zed requires a **tree-sitter grammar** — planned (tracked in ROADMAP 5d follow-ups). Until then these editors still get full LSP diagnostics/completion; Neovim can also use the TextMate grammar via plugins.
² Sublime consumes the same TextMate grammar as VS Code (`vscode/syntaxes/xiom.tmLanguage.json`).

## Quick sanity test (any editor)

Open a `.xi` file containing `fn main() -> Int { return true; }` — you should see
`T001: return type mismatch: expected Int, found Bool` from the LSP within a second.
