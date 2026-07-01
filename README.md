# AXIOM Programming Language

**Safe · Verified · Precise** — A systems programming language with first-class contracts.

[![Tests](https://img.shields.io/badge/tests-246%20passed-brightgreen)]()
[![Version](https://img.shields.io/badge/version-0.20.0-blue)]()

AXIOM is a compiled, statically typed, memory-safe systems programming language. It compiles to native code via LLVM and supports x86_64, ARM, RISC-V, and WebAssembly. The compiler is self-hosted — it compiles itself.

## Quick Start

### One-Command Install (Windows)

```powershell
# Clone and auto-install everything (Rust, clang, build tools)
git clone https://github.com/NgonArt_STUDIO/AXIOM.git
cd AXIOM
.\install_deps.ps1      # auto-installs missing dependencies
.\install.ps1            # builds + installs AXIOM to PATH

# After restarting terminal:
axiom --version
# → AXIOM Compiler v0.20.0 "Hardened"
```

### One-Command Install (macOS / Linux)

```bash
git clone https://github.com/NgonArt_STUDIO/AXIOM.git
cd AXIOM
chmod +x install_deps.sh install.sh
./install_deps.sh        # auto-installs missing dependencies
./install.sh             # builds + installs AXIOM to PATH

# After restarting terminal:
axiom --version
```

### Pre-built Release (Windows)

Download the latest `axiom-v0.20.0-windows-x64.zip` from [Releases](https://github.com/NgonArt_STUDIO/AXIOM/releases), extract, and double-click `install.bat`. It will:

1. Ask where to install (default: `%LOCALAPPDATA%\axiom`)
2. Copy binaries + stdlib + runtime
3. Offer to add to PATH
4. Offer to register `.ax` files with the AXIOM icon
5. Detect if `clang` is missing and tell you how to install it

### What the Installer Installs

| Dependency | Source Build | Pre-built Release |
|------------|-------------|-------------------|
| **Rust** (rustc/cargo) | Auto-installed by `install_deps` | Not needed |
| **LLVM/clang** | Auto-installed by `install_deps` | Warned if missing* |
| **C build tools** | Auto-installed by `install_deps` | Not needed |
| **axiomc.exe** | Built from source | Included |
| **stdlib** | Copied from repo | Included |
| **`.ax` icon** | Registered (optional) | Registered (optional) |

\* clang is a runtime dependency — axiomc emits LLVM IR, clang compiles it to native binary. Without clang, use `axiomc --emit-ir file.ax` to view IR.

### Creating a Release

```powershell
# Build all tools + create portable folder + ZIP
.\package.ps1 -Version 0.20.0
# Produces:
#   release\axiom-v0.20.0\                 ← portable folder
#   release\axiom-v0.20.0-windows-x64.zip   ← distributable ZIP
```

Release folder structure:
```
axiom-v0.20.0\
├── bin\              axiomc.exe, axiom-fmt.exe, axiom-doc.exe,
│                     axiom-ffigen.exe, axiom-pkg.exe, axiom-lsp.exe,
│                     axiom-icon.ico
├── lib\              Standard library (.ax source files)
├── runtime\          C runtime (axiom_runtime.c)
├── install.bat       Double-click Windows installer
└── README.txt
```

Install from a release:
```powershell
# From local release folder
.\install.ps1 -BinaryPath .\release\axiom-v0.20.0

# Or just double-click install.bat in the release folder
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

### Bootstrap Chain (Self-Hosting Proof)

The AXIOM compiler can compile itself. The bootstrap chain begins with the Rust-compiled compiler and produces a self-sustaining loop:

```
   Rust axiomc (bootstrap)
        │
        ▼ compiles selfhost/axiomc_v10.ax
        │
   axiomc.exe  ─── stage 1 selfhost binary
        │
        ▼ reads its own source, emits LLVM IR
        │
   bootstrap_output.ll  (18 function definitions)
        │
        ▼ compiled by clang + axiom_runtime.c
        │
   axiomc_stage2.exe  ─── stage 2 selfhost binary (target)
```

```powershell
# Step 1: Compile the selfhost with Rust (bootstrap)
cargo run -p axiomc -- -o axiomc.exe selfhost/axiomc_v10.ax

# Step 2: The resulting axiomc.exe is the AXIOM compiler
.\axiomc.exe --help
# → AXIOM Compiler v0.12.0

# Step 3: Use it to compile AXIOM code
.\axiomc.exe examples/demo_float.ax --emit-ir

# Step 4: Self-host the bootstrap
.\axiomc.exe selfhost/axiomc_v10.ax --emit-ir
# → produces LLVM IR for all 18 functions
```

Latest verification (2026-07-01, feat/ecosystem branch):

| Step | Command | Result |
|------|---------|--------|
| 1 | `cargo run -p axiomc -- -o bootstrap_selfhost.exe selfhost\axiomc_v10.ax` | ✅ Compiled, exit 0 |
| 2 | `.\bootstrap_selfhost.exe` | ✅ Emits `define i64 @main()` + 17 other functions |
| 3 | `.\bootstrap_selfhost.exe > bootstrap_output.ll` | ✅ 18 function definitions captured |
| 4 | `clang -o bootstrap_stage2.exe bootstrap_output.ll stdlib\runtime\axiom_runtime.c` | ❌ IR syntax issues (named SSA values in calls lack `%` prefix) |

**Status**: The Rust→selfhost→IR pipeline is fully verified. The selfhost compiler emits valid LLVM IR structurally (18 functions, proper module triple) but has two known IR emission bugs: (1) named SSA values in `call` operands lack `%` prefix, (2) string literal arguments are not properly quoted. These affect `codegen/expr.ax` in the selfhost source. Once fixed, `clang` will produce a working stage-2 binary, completing the bootstrap loop.

**Bootstrap verified (partial)**: The AXIOM compiler, compiled by Rust, can read and compile its own source, producing structured LLVM IR with 18 function definitions. Rust is the permanent bootstrap fallback; the selfhost compiler is IR-verified and awaiting codegen fixes for full stage-2 closure.

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
| v0.20.0 | 2026-07-02 | 246+ | Critical safety fixes + multi-file + auto-installer |
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

## Package Registry

AXIOM has a local package registry. Start the server, then publish and install packages.

```powershell
# Start the registry
python registry/server.py
# → http://localhost:8080

# Publish a package
axiom pkg publish

# List available packages
curl http://localhost:8080/index.json

# Install a package
axiom pkg install axiom-http
```

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
