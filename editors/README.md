<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Editor Integrations

XIOM tooling is **protocol-based** -- `xiom-lsp` speaks the Language Server Protocol and
`xiom-dbg` speaks the Debug Adapter Protocol. Any editor with LSP/DAP support integrates
with a small config; only VS Code ships a full extension.

**Prerequisite for all editors:** the XIOM toolchain `bin/` directory on `PATH`
(so `xiom-lsp` and `xiom-dbg` resolve). Verify: `xiom --version` and `xiom-lsp --help`.

## Support Matrix

| Editor | Highlighting | LSP (diagnostics, hover, completion, refs, rename) | Debugging (DAP) | How |
|--------|--------------|-----|-----|-----|
| **VS Code** | [OK] TextMate | [OK] | [OK] | [vscode/](./vscode/) -- universal .vsix (Marketplace + Open VSX) |
| **Neovim** | [WARN] via LSP semantic info1 | [OK] | [OK] nvim-dap | [neovim/](./neovim/) -- Lua config |
| **JetBrains** (IDEA, CLion, RustRover) | [WARN] plugin textmate bundle | [OK] LSP4IJ | [OK] LSP4IJ 0.8+ | [jetbrains/](./jetbrains/) -- LSP4IJ setup |
| **Helix** | [WARN] needs tree-sitter1 | [OK] | [OK] built-in | [helix/](./helix/) -- languages.toml |
| **Sublime Text** | [OK] TextMate2 | [OK] LSP package | -- | [sublime/](./sublime/) -- LSP settings |
| **Emacs** | basic | [OK] eglot | [OK] dape | [emacs/](./emacs/) -- elisp snippet |
| **Zed** | [WARN] needs tree-sitter1 | [OK] (extension API) | -- | planned |

## Distribution policy

- **VS Code** is the only editor with a full extension, and it ships as ONE
  universal VSIX: no platform binaries are bundled. `xiom-lsp` and `xiom-dbg`
  are resolved at runtime from the `xiom.lsp.path` / `xiom.debugAdapterPath`
  settings, then `PATH` (the toolchain installer puts `bin/` on `PATH`), then
  a workspace `target/release|debug` build. Minimum toolchain: the version
  the README of the extension documents (current release line).
- **Publishing** is part of `release.yml` on tag builds: the `.vsix` plus a
  `SHA256SUMS` entry ride on the GitHub Release; `vsce publish` (VS Code
  Marketplace) and `ovsx publish` (Open VSX, covers VSCodium/Cursor/Gitpod)
  run only when `VSCE_PAT` / `OVSX_TOKEN` repository secrets are present.
- **First run**: the extension verifies `xiom`, `xiom-lsp` and `xiom-dbg` on
  activation and, when something is missing, shows a notification linking to
  <https://xiom-lang.org/install> plus a `XIOM: Recheck toolchain` command.
  It never installs the toolchain silently -- the official installer is the
  single install path.
- **Other editors stay config-only** in this directory (Emacs, Helix,
  Neovim, Sublime, JetBrains via LSP4IJ) -- no per-ecosystem publishing for
  now. **Visual Studio** (separate marketplace) is deferred; see ROADMAP
  M13.11.


1 Native highlighting in Neovim/Helix/Zed requires a **tree-sitter grammar** -- planned (tracked in ROADMAP 5d follow-ups). Until then these editors still get full LSP diagnostics/completion; Neovim can also use the TextMate grammar via plugins.
2 Sublime consumes the same TextMate grammar as VS Code (`vscode/syntaxes/xiom.tmLanguage.json`).

## Quick sanity test (any editor)

Open a `.xi` file containing `fn main() -> Int { return true; }` -- you should see
`T001: return type mismatch: expected Int, found Bool` from the LSP within a second.
