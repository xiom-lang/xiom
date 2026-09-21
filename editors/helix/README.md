<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM for Helix

LSP diagnostics/completion/hover work today via `xiom-lsp`. Native highlighting
needs a tree-sitter grammar (planned).

## Install

Append the contents of `languages.toml` to your Helix config
(`~/.config/helix/languages.toml`). Requires `xiom-lsp` on PATH.

## Debugging

Helix has built-in DAP. The `[language.debugger]` section in `languages.toml`
wires `xiom-dbg`. Compile with symbols first: `xiom -g -o a.exe main.xi`,
then `:debug-start` in Helix.
