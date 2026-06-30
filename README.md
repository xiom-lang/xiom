# AXIOM Programming Language

**Safe · Verified · Precise** — A systems programming language with first-class contracts.

## Quick Start

### Install

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1
```

Close and reopen your terminal. Then:

```powershell
axiom --help
```

### Your First Program

Create `hello.ax`:
```axiom
fn main() -> Int {
  return 42;
}
```

Compile and run:
```powershell
axiom compile --run hello.ax
# → exit code: 42
```

## Toolchain

| Command | Description |
|---------|-------------|
| `axiom compile file.ax` | Compile AXIOM source to native binary |
| `axiom fmt file.ax` | Canonical formatting (--check for CI) |
| `axiom doc file.ax` | Generate Markdown documentation |
| `axiom ffigen spec.axiom-bind` | Generate C FFI bindings |
| `axiom pkg --list --root dir` | Package manifest inspection |
| `axiom lsp` | Language server for editor integration |
| `axiom verify file.ax` | Generate SMT-LIB for contract verification |

## Targets

| Target | Flag |
|--------|------|
| Windows x86_64 | default |
| WebAssembly | `--target wasm` |
| ARM aarch64 | `--target arm` |
| RISC-V riscv64 | `--target riscv` |

## Ecosystem

| Library | Purpose |
|---------|---------|
| `axiom-http` | HTTP client (libcurl FFI) |
| `axiom-crypto` | Cryptographic hashing (OpenSSL FFI) |
| `axiom-sql` | SQL database (SQLite FFI) |

## Build from Source

```powershell
cargo build --release
cargo test  # 208 tests
```

## License

MIT OR Apache-2.0
