# AXIOM Programming Language

**Safe · Verified · Precise** — A systems programming language with first-class contracts.

[![Tests](https://img.shields.io/badge/tests-234%20passed-brightgreen)]()

AXIOM is a compiled, statically typed, memory-safe systems programming language. It compiles to native code via LLVM and supports x86_64, ARM, RISC-V, and WebAssembly. The compiler is self-hosted — it compiles itself.

## Quick Start

### Download (Pre-built)

| Version | Platform | Download |
|---------|----------|----------|
| v0.12.0 | Windows x64 | [axiom-v0.12.0-windows-x64.zip]() |
| v0.12.0 | Linux x64 | [axiom-v0.12.0-linux-x64.tar.gz]() |
| v0.12.0 | macOS (ARM) | [axiom-v0.12.0-macos-arm64.tar.gz]() |

```powershell
# Windows: run install.bat, restart terminal
axiom --help
axiom compile hello.ax
```

### Build from Source

```powershell
# Prerequisites
#   Rust: https://rustup.rs
#   LLVM/clang: https://github.com/llvm/llvm-project/releases
#     OR Visual Studio with "Desktop development with C++"

# Clone and build
git clone https://github.com/NgonArt_STUDIO/AXIOM.git
cd AXIOM
cargo build --release -p axiomc

# Install to PATH
powershell -ExecutionPolicy Bypass -File install.ps1
# OR: copy target\release\axiomc.exe to a directory in PATH

# Verify
axiomc --version
axiomc --help
```

### Building the Self-Hosted Compiler

The AXIOM compiler can compile itself. The Rust compiler is the bootstrap.

```powershell
# Step 1: Compile the self-hosted compiler with Rust
cargo run -p axiomc -- -o axiomc.exe selfhost/axiomc_v10.ax

# Step 2: The resulting axiomc.exe IS the AXIOM compiler
.\axiomc.exe --help

# Step 3: Use it to compile AXIOM code
.\axiomc.exe examples/demo_float.ax --run
```

## Toolchain

| Command | Description |
|---------|-------------|
| `axiomc` | Compiler — compiles .ax to native binary |
| `axiom fmt` | Canonical formatter |
| `axiom doc` | Documentation generator |
| `axiom lsp` | Language server |
| `axiom pkg` | Package manager |
| `axiom ffigen` | FFI binding generator |
| `axiom verify` | Contract verification (SMT-LIB) |

### Compiler Flags

```
axiomc [OPTIONS] <source.ax>

OPTIONS:
  --help              Show help
  --version           Print version
  -o <output>         Output binary (default: a.exe)
  --run               Compile and run
  --emit-ir           Print LLVM IR (no compilation)
  --target <target>   native (default), wasm, arm, riscv
  --no-contracts      Disable contract runtime checks
  --diagnostics=json  JSON error output
  --dump-contracts    Print contract index
  --verify            Generate SMT-LIB verification
```

### Targets

| Target | Flag | Triple |
|--------|------|--------|
| Windows x86_64 | default | `x86_64-pc-windows-msvc` |
| WebAssembly | `--target wasm` | `wasm32-unknown-unknown` |
| ARM aarch64 | `--target arm` | `aarch64-unknown-linux-gnu` |
| RISC-V | `--target riscv` | `riscv64gc-unknown-linux-gnu` |

## Language Features

| Feature | Status |
|---------|--------|
| Ownership / Borrow Checker | ✅ Lexical scope: &T, &mut T, move semantics |
| Contracts (requires/ensures/invariant) | ✅ Runtime guards with @llvm.trap |
| derive (Eq, Clone, Display, Hash, Ord) | ✅ Compiler-generated implementations |
| Generics with inline constraints | ✅ Monomorphisation ([T: Ord]) |
| Error handling (Result, Option, ?) | ✅ |
| Module system (module/use/pub) | ✅ |
| Async / Channels | ✅ Parsed + checked (codegen deferred) |
| C FFI (extern) | ✅ Zero-cost interop |
| Match / Pattern matching | ✅ Exhaustion checking |
| Structs + Enums | ✅ With derive support |
| Interfaces (structural) | ✅ |

## Platform Build Matrix

### Pre-built Binaries

| Platform | Rust compiler | Selfhost |
|----------|--------------|----------|
| Windows x64 | ✅ | ✅ |
| macOS ARM | ✅ (via CI) | ⏳ |
| Linux x64 | ✅ (via CI) | ⏳ |

### Build from Source Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| **Rust** | 1.75+ | `rustup` recommended |
| **LLVM/clang** | 15+ | For native compilation |
| **Windows SDK** | 10.0+ | Included with Visual Studio |
| **WebAssembly** | — | Built-in via LLVM |

```powershell
# Install Rust
winget install Rustlang.Rustup
# OR: https://rustup.rs

# Install clang/LLVM
winget install LLVM.LLVM
# OR: https://github.com/llvm/llvm-project/releases

# Verify
rustc --version
clang --version
```

### Cross-Compilation (from any host)

```powershell
# Build for Windows
cargo build --release -p axiomc --target x86_64-pc-windows-msvc

# Build for Linux
cargo build --release -p axiomc --target x86_64-unknown-linux-gnu

# Build for macOS ARM
cargo build --release -p axiomc --target aarch64-apple-darwin
```

## Version History

See [RELEASES.md](RELEASES.md) for full version history with changelog, test counts, and download links.

| Version | Date | Tests | Milestone |
|---------|------|-------|-----------|
| v0.12.0 | 2026-07-01 | 234 | Production self-hosted compiler |
| v0.11.0 | 2026-07-01 | 213 | Full body IR, if/else/while |
| v0.10.0 | 2026-06-30 | 208 | Self-hosting bootstrap |
| v0.9.0 | 2026-06-30 | 197 | E2E pipeline validation |
| v0.5.0 | 2026-06-30 | 125 | Self-hosting structural proof |
| v0.2.5 | 2026-06-30 | 109 | Full language surface |
| v0.1.0 | 2026-06-30 | 36 | First working pipeline |

## Ecosystem

| Library | Purpose | Status |
|---------|---------|--------|
| `axiom-http` | HTTP client (libcurl) | 🚧 |
| `axiom-crypto` | Cryptography (OpenSSL) | 🚧 |
| `axiom-sql` | SQL database (SQLite) | 🚧 |

## Project Structure

```
AXIOM/
├── crates/           # Rust bootstrap compiler (12 crates)
│   ├── axiomc/       #   Compiler CLI
│   ├── axiom-ast/    #   AST definitions
│   ├── axiom-lexer/  #   Tokenizer
│   ├── axiom-parser/ #   Recursive descent parser
│   ├── axiom-check/  #   Type checker + borrow checker
│   ├── axiom-codegen/#   LLVM IR emitter
│   └── ...           #   fmt, doc, lsp, pkg, ffigen, verify
├── selfhost/         # AXIOM self-hosted compiler
│   └── axiomc_v10.ax #   Main compiler source
├── stdlib/           # Standard library
│   ├── axiom/        #   core, io, collections, string, math, ffi, async
│   └── runtime/      #   C runtime (axiom_runtime.c)
├── examples/         # Example programs (21)
├── ecosystem/        # Ecosystem libraries
│   ├── axiom-http/   #   HTTP (libcurl)
│   ├── axiom-crypto/ #   Cryptography (OpenSSL)
│   └── axiom-sql/    #   SQL (SQLite)
├── editors/vscode/   # VS Code extension
├── playground/       # WASM playground
├── dist/             # Distribution packages
├── specs/            # Language specification + build strategy
└── docs/             # Documentation
```

## License

MIT OR Apache-2.0

Copyright (c) 2026 Eleftherios Notas
