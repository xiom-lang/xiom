# XIOM Language Support for VS Code

Syntax highlighting, autocomplete, go-to-definition, and diagnostics for the XIOM programming language.

## Features

- **Syntax highlighting** — keywords, types, operators, contracts, strings, comments
- **Bracket matching** — {}, [], (), ""
- **Line/block comments** — // and /* */
- **LSP integration** — diagnostics, hover, completion, go-to-definition (requires xiom-lsp binary)

## LSP Setup

```bash
cargo build -p xiom-lsp
```

Then set `xiom.lsp.path` in VS Code settings to the path of the compiled binary.

## License

MIT OR Apache-2.0
