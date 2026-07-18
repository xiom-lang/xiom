# XIOM for Sublime Text 4

LSP + TextMate highlighting. Requires `xiom-lsp` on PATH and the
[LSP package](https://packagecontrol.io/packages/LSP).

## Setup

1. **Install the LSP package**: `Ctrl+Shift+P` → "Package Control: Install Package" → `LSP`.
2. **Register the server**: `Preferences → Package Settings → LSP → Settings`, merge:
   the contents of `LSP.sublime-settings` from this folder.
3. **Syntax highlighting**: Sublime reads TextMate grammars. Copy
   `../vscode/syntaxes/xiom.tmLanguage.json` into
   `Packages/User/` (via `Preferences → Browse Packages`) and Sublime will offer
   "XIOM" as a syntax for `.xi` files (View → Syntax → XIOM).

Open a `.xi` file — diagnostics/hover/completion active.
