# XIOM for Neovim (0.10+)

Full LSP + debugging via native `vim.lsp` and `nvim-dap`. Requires `xiom-lsp`
and `xiom-dbg` on PATH (XIOM release `bin/`).

## Install

Copy `xiom.lua` into your config (e.g. `~/.config/nvim/lua/plugins/xiom.lua`
or paste into `init.lua`).

## What you get

- Filetype detection for `.xi`
- Diagnostics (type errors, borrow errors) as you edit
- Hover (`K`), completion, go-to-definition (`gd`), references (`grr`), rename (`grn`)
- Debugging with breakpoints/step/locals via nvim-dap (`:DapToggleBreakpoint`, `:DapContinue`)

## Notes

- Native tree-sitter highlighting is planned; until then LSP provides diagnostics
  and you can enable basic keyword highlighting with `vim.cmd('runtime! syntax/c.vim')`
  as a rough fallback, or use a TextMate-grammar plugin.
- Debug target defaults to `a.exe`/`a.out` in the workspace root -- compile first:
  `xiom -g -o a.exe main.xi`.
