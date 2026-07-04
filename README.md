# XIOM Programming Language

**Safe · Verified · Precise** — A systems programming language with first-class contracts.

[![Tests](https://img.shields.io/badge/tests-246%20passed-brightgreen)]()
[![Version](https://img.shields.io/badge/version-0.20.0-blue)]()

XIOM is a compiled, statically typed, memory-safe systems programming language. It compiles to native code via LLVM and supports x86_64, ARM, RISC-V, and WebAssembly. The compiler is self-hosted — it compiles itself.

## Quick Start

### One-Command Install (Windows)

```powershell
# Clone and auto-install everything (Rust, clang, build tools)
git clone https://github.com/NgonArt_STUDIO/XIOM.git
cd XIOM
.\install_deps.ps1      # auto-installs missing dependencies
.\install.ps1            # builds + installs XIOM to PATH

# After restarting terminal:
xiom --version
# → XIOM Compiler v0.20.0 "Hardened"
```

### One-Command Install (macOS / Linux)

```bash
git clone https://github.com/NgonArt_STUDIO/XIOM.git
cd XIOM
chmod +x install_deps.sh install.sh
./install_deps.sh        # auto-installs missing dependencies
./install.sh             # builds + installs XIOM to PATH

# After restarting terminal:
xiom --version
```

### Pre-built Release (Windows)

Download the latest `xiom-v0.20.0-windows-x64.zip` from [Releases](https://github.com/NgonArt_STUDIO/XIOM/releases), extract, and double-click `install.bat`. It will:

1. Ask where to install (default: `%LOCALAPPDATA%\xiom`)
2. Copy binaries + stdlib + runtime
3. Offer to add to PATH
4. Offer to register `.xi` files with the XIOM icon
5. Detect if `clang` is missing and tell you how to install it

### What the Installer Installs

| Dependency | Source Build | Pre-built Release |
|------------|-------------|-------------------|
| **Rust** (rustc/cargo) | Auto-installed by `install_deps` | Not needed |
| **LLVM/clang** | Auto-installed by `install_deps` | Warned if missing* |
| **C build tools** | Auto-installed by `install_deps` | Not needed |
| **xiomc.exe** | Built from source | Included |
| **stdlib** | Copied from repo | Included |
| **`.xi` icon** | Registered (optional) | Registered (optional) |

\* clang is a runtime dependency — xiomc emits LLVM IR, clang compiles it to native binary. Without clang, use `xiomc --emit-ir file.xi` to view IR.

### Creating a Release

```powershell
# Build all tools + create portable folder + ZIP
.\package.ps1 -Version 0.20.0
# Produces:
#   release\xiom-v0.20.0\                 ← portable folder
#   release\xiom-v0.20.0-windows-x64.zip   ← distributable ZIP
```

Release folder structure:
```
xiom-v0.20.0\
├── bin\              xiomc.exe, xiom-fmt.exe, xiom-doc.exe,
│                     xiom-ffigen.exe, xiom-pkg.exe, xiom-lsp.exe,
│                     xiom-icon.ico
├── lib\              Standard library (.xi source files)
├── runtime\          C runtime (xiom_runtime.c)
├── install.bat       Double-click Windows installer
└── README.txt
```

Install from a release:
```powershell
# From local release folder
.\install.ps1 -BinaryPath .\release\xiom-v0.20.0

# Or just double-click install.bat in the release folder
```

### Building the Self-Hosted Compiler

The XIOM compiler can compile itself. The Rust compiler is the bootstrap.

```powershell
# Step 1: Compile the self-hosted compiler with Rust
cargo run -p xiomc -- -o xiomc.exe selfhost/xiomc_v10.xi

# Step 2: The resulting xiomc.exe IS the XIOM compiler
.\xiomc.exe --help

# Step 3: Use it to compile XIOM code
.\xiomc.exe examples/demo_float.xi --run
```

### Bootstrap Chain (Self-Hosting Proof)

The XIOM compiler can compile itself. The bootstrap chain begins with the Rust-compiled compiler and produces a self-sustaining loop:

```
   Rust xiomc (bootstrap)
        │
        ▼ compiles selfhost/xiomc_v10.xi
        │
   xiomc.exe  ─── stage 1 selfhost binary
        │
        ▼ reads its own source, emits LLVM IR
        │
   bootstrap_output.ll  (18 function definitions)
        │
        ▼ compiled by clang + xiom_runtime.c
        │
   xiomc_stage2.exe  ─── stage 2 selfhost binary (target)
```

```powershell
# Step 1: Compile the selfhost with Rust (bootstrap)
cargo run -p xiomc -- -o xiomc.exe selfhost/xiomc_v10.xi

# Step 2: The resulting xiomc.exe is the XIOM compiler
.\xiomc.exe --help
# → XIOM Compiler v0.12.0

# Step 3: Use it to compile XIOM code
.\xiomc.exe examples/demo_float.xi --emit-ir

# Step 4: Self-host the bootstrap
.\xiomc.exe selfhost/xiomc_v10.xi --emit-ir
# → produces LLVM IR for all 18 functions
```

Latest verification (2026-07-01, feat/ecosystem branch):

| Step | Command | Result |
|------|---------|--------|
| 1 | `cargo run -p xiomc -- -o bootstrap_selfhost.exe selfhost\xiomc_v10.xi` | ✅ Compiled, exit 0 |
| 2 | `.\bootstrap_selfhost.exe` | ✅ Emits `define i64 @main()` + 17 other functions |
| 3 | `.\bootstrap_selfhost.exe > bootstrap_output.ll` | ✅ 18 function definitions captured |
| 4 | `clang -o bootstrap_stage2.exe bootstrap_output.ll stdlib\runtime\xiom_runtime.c` | ❌ IR syntax issues (named SSA values in calls lack `%` prefix) |

**Status**: The Rust→selfhost→IR pipeline is fully verified. The selfhost compiler emits valid LLVM IR structurally (18 functions, proper module triple) but has two known IR emission bugs: (1) named SSA values in `call` operands lack `%` prefix, (2) string literal arguments are not properly quoted. These affect `codegen/expr.xi` in the selfhost source. Once fixed, `clang` will produce a working stage-2 binary, completing the bootstrap loop.

**Bootstrap verified (partial)**: The XIOM compiler, compiled by Rust, can read and compile its own source, producing structured LLVM IR with 18 function definitions. Rust is the permanent bootstrap fallback; the selfhost compiler is IR-verified and awaiting codegen fixes for full stage-2 closure.

## Toolchain

| Command | Description |
|---------|-------------|
| `xiomc` | Compiler — compiles .xi to native binary |
| `xiom fmt` | Canonical formatter |
| `xiom doc` | Documentation generator |
| `xiom lsp` | Language server |
| `xiom pkg` | Package manager |
| `xiom ffigen` | FFI binding generator |
| `xiom verify` | Contract verification (SMT-LIB) |

### Compiler Flags

```
xiomc [OPTIONS] <source.xi>

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
cargo build --release -p xiomc --target x86_64-pc-windows-msvc

# Build for Linux
cargo build --release -p xiomc --target x86_64-unknown-linux-gnu

# Build for macOS ARM
cargo build --release -p xiomc --target aarch64-apple-darwin
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
| `xiom-http` | HTTP client (libcurl) | 🚧 |
| `xiom-crypto` | Cryptography (OpenSSL) | 🚧 |
| `xiom-sql` | SQL database (SQLite) | 🚧 |

## Package Registry

XIOM has a local package registry. Start the server, then publish and install packages.

```powershell
# Start the registry
python registry/server.py
# → http://localhost:8080

# Publish a package
xiom pkg publish

# List available packages
curl http://localhost:8080/index.json

# Install a package
xiom pkg install xiom-http
```

## Project Structure

```
XIOM/
├── crates/           # Rust bootstrap compiler (12 crates)
│   ├── xiomc/       #   Compiler CLI
│   ├── xiom-ast/    #   AST definitions
│   ├── xiom-lexer/  #   Tokenizer
│   ├── xiom-parser/ #   Recursive descent parser
│   ├── xiom-check/  #   Type checker + borrow checker
│   ├── xiom-codegen/#   LLVM IR emitter
│   └── ...           #   fmt, doc, lsp, pkg, ffigen, verify
├── selfhost/         # XIOM self-hosted compiler
│   └── xiomc_v10.xi #   Main compiler source
├── stdlib/           # Standard library
│   ├── xiom/        #   core, io, collections, string, math, ffi, async
│   └── runtime/      #   C runtime (xiom_runtime.c)
├── examples/         # Example programs (21)
├── ecosystem/        # Ecosystem libraries
│   ├── xiom-http/   #   HTTP (libcurl)
│   ├── xiom-crypto/ #   Cryptography (OpenSSL)
│   └── xiom-sql/    #   SQL (SQLite)
├── editors/vscode/   # VS Code extension
├── playground/       # WASM playground
├── dist/             # Distribution packages
├── specs/            # Language specification + build strategy
└── docs/             # Documentation
```

## License

MIT OR Apache-2.0

Copyright (c) 2026 Eleftherios Notas
