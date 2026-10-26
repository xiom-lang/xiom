<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Programming Language

**Safe - Verified - Precise** -- A systems programming language with first-class contracts.

[![Tests](https://img.shields.io/badge/tests-2%2C600%2B-brightgreen)]()
[![Version](https://img.shields.io/badge/version-0.65.1-blue)]()

XIOM is a compiled, statically typed, memory-safe systems programming language. It compiles to native code via LLVM and supports x86_64, ARM, RISC-V, and WebAssembly. The Rust compiler is the production toolchain; the self-hosted implementation is in Phase 6 (RD10: T3 83/83, 14/64 byte-exact).

## Current Status (2026-10-26)

- **v0.65.1 is the current candidate** for the next production cut: owned-vector reclamation at loop/match/aggregate/function exits, a flat 1,200-frame animated UI soak, and the `stdlib-v0.65.1` read-helper ownership contracts. See `docs/RELEASE_GATE_v0.65.1.md`; the tag is cut only after CI and the all-platform dry-run are green.
- **Production pin until v0.65.1 ships:** `v0.65.0` (tag `f527d5a`; release 40041016248, 11 assets). **v0.64.9 remains withdrawn** -- do not pin or deploy it.
- **Registry:** `xiom-std 0.65.0`; the `xiom-std 0.65.1` decision follows the stdlib tag.
- **Self-hosting:** Phase 6 RD11 is **T3 83/83 green** (phase-5 19/19, T6 **14/64** byte-exact). Next: m1136 block-arm mirror, Option/`.len`, `Vec.pop`, `&Vec`+f64, index arithmetic.
- **Open coordination:** xvector, orbitdb, engine, packages, and benchmark holds; the v0.65.1 release check records the reporter rows.

## Quick Start

### One-Command Install (Windows)

```powershell
# Clone and auto-install everything (Rust, clang, build tools)
git clone https://github.com/xiom-lang/xiom.git
cd xiom
.\install_deps.ps1      # auto-installs missing dependencies
.\install.ps1            # builds + installs XIOM to PATH

# After restarting terminal:
xiom --version
# -> XIOM Compiler v0.64.8
```

### One-Command Install (macOS / Linux)

```bash
git clone https://github.com/xiom-lang/xiom.git
cd xiom
chmod +x install_deps.sh install.sh
./install_deps.sh        # auto-installs missing dependencies
./install.sh             # builds + installs XIOM to PATH

# After restarting terminal:
xiom --version
```

### Pre-built Release (Windows)

Download the latest `xiom-0.64.8-windows-x64.zip` from [Releases](https://github.com/xiom-lang/xiom/releases/tag/v0.64.8), extract, and double-click `install.bat`. It will:

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
| **xiom.exe** | Built from source | Included |
| **xiom-pkg** (package manager) | Built from source | Included |
| **stdlib** | Copied from repo | Included |
| **`.xi` icon** | Registered (optional) | Registered (optional) |

\* clang is a runtime dependency -- xiom emits LLVM IR, clang compiles it to native binary. Without clang, use `xiom --emit-ir file.xi` to view IR.

### Creating a Release

```powershell
# Build all tools + create portable folder + ZIP
# (version defaults to Cargo.toml [workspace.package] version)
.\package.ps1
# Produces:
#   release\xiom-0.64.8\                  <- portable folder
#   release\xiom-0.64.8-windows-x64.zip   <- distributable ZIP
```

Release folder structure:
```
xiom-0.64.8\
|-- bin\              xiom.exe, xiom-pkg.exe, xiom-fmt.exe, xiom-doc.exe,
|                     xiom-lsp.exe, xiom-dbg.exe, xiom-mcp.exe,
|                     xiom-verify.exe, xiom-ffigen.exe, z3.exe
|-- lib\              Standard library (.xi source files)
|-- runtime\          C runtime (xiom_runtime.c)
|-- install.bat       Double-click Windows installer
|-- LICENSE-Z3        Z3 solver license (bundled bin/z3 for --verify)
`-- README.txt
```

Install from a release:
```powershell
# From local release folder
.\install.ps1 -BinaryPath .\release\xiom-0.64.8

# Or just double-click install.bat in the release folder
```

### Building the Self-Hosted Compiler

The self-hosted implementation lives in `selfhost/src/main.xi`. It is under active parity work (Phase 6 RD10: T3 83/83, T6 14/64 byte-exact); the Rust compiler remains the bootstrap and the production toolchain for releases.

```powershell
# Step 1: Compile the self-hosted compiler with Rust
cargo run -p xiom -- -o xiom-self.exe selfhost/src/main.xi

# Step 2: The resulting xiom-self.exe is the self-hosted compiler
.\xiom-self.exe --help

# Step 3: Use it to compile XIOM code
.\xiom-self.exe examples/demo_float.xi --run
```

### Bootstrap Chain (Historical)

The original self-hosting proof (v0.12.0, 2026-07-01) used the then-current `selfhost` source and produced a stage-2 compiler. That proof is historical. The current self-hosted implementation is `selfhost/src/main.xi`; parity is tracked in `docs/checklists/selfhost-phase6.md` (RD10: T3 83/83, T6 14/64 byte-exact). The Rust compiler remains the production bootstrap and release toolchain.

## Toolchain

| Command | Description |
|---------|-------------|
| `xiom` | Compiler -- compiles .xi to native binary |
| `xiom fmt` | Canonical formatter |
| `xiom doc` | Documentation generator |
| `xiom lsp` | Language server |
| `xiom pkg` | Package manager |
| `xiom ffigen` | FFI binding generator |
| `xiom verify` | Contract verification (SMT-LIB) |

### Compiler Flags

```
xiom [OPTIONS] <source.xi>

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
| RISC-V | `--target riscv` | `riscv64-unknown-linux-gnu` |

> The v0.64.8 archive predates the RISC-V LLVM triple correction; use `main` (or the next release) for the `riscv64-unknown-linux-gnu` spelling.

## Language Features

| Feature | Status |
|---------|--------|
| Ownership / Borrow Checker | [OK] Lexical scope: &T, &mut T, move semantics |
| Contracts (requires/ensures/invariant) | [OK] Runtime guards with @llvm.trap |
| derive (Eq, Clone, Display, Hash, Ord) | [OK] Compiler-generated implementations |
| Generics with inline constraints | [OK] Monomorphisation ([T: Ord]) |
| Error handling (Result, Option, ?) | [OK] |
| Module system (module/use/pub) | [OK] |
| Async / Channels | [OK] Parsed + checked (codegen deferred) |
| C FFI (extern) | [OK] Zero-cost interop |
| Match / Pattern matching | [OK] Exhaustion checking |
| Structs + Enums | [OK] With derive support |
| Interfaces (structural) | [OK] |

## Platform Build Matrix

### Pre-built Binaries

| Platform | Rust compiler | Selfhost |
|----------|--------------|----------|
| Windows x64 | [OK] | [OK] |
| macOS ARM | [OK] (via CI) | [WIP] |
| Linux x64 | [OK] (via CI) | [WIP] |

### Build from Source Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| **Rust** | 1.75+ | `rustup` recommended |
| **LLVM/clang** | 15+ | For native compilation |
| **Windows SDK** | 10.0+ | Included with Visual Studio |
| **WebAssembly** | -- | Built-in via LLVM |

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
cargo build --release -p xiom --target x86_64-pc-windows-msvc

# Build for Linux
cargo build --release -p xiom --target x86_64-unknown-linux-gnu

# Build for macOS ARM
cargo build --release -p xiom --target aarch64-apple-darwin
```

## Version History

See the [Releases page](https://github.com/xiom-lang/xiom/releases) for the full version history with changelog, test counts, and download links.

| Version | Date | Tests | Milestone |
|---------|------|-------|-----------|
| v0.64.9 | 2026-10-22 | 2,600+ | **Pre-release, withdrawn from adoption** -- nested enum-pattern payloads, m372/1451 + true 1550 regressions, m1511 transitive port args, C07; superseded by v0.65.0 |
| v0.64.8 | 2026-10-19 | 2,600+ | Scoped arena for frame-shaped memory; zero-capacity Vec fix; m360 i128 div/rem; m368 Vec-drop reclaim; m369 catalog index; m370 generic-struct Vec; C-ORBIT-12/13, XVC-C-20, B-34 |
| v0.64.0 | 2026-10-05 | 2426 | Major correctness: reflect heap corruption, TcpStream.read elision, unsafe stack exhaustion, installed runtime links, exact float bits |
| v0.63.1 | 2026-10-05 | 2423 | Fixes: runtime contract evaluator, verifier SMT self/sort emission, LZ4 duplicate leaf, catalog flush |
| v0.63.0 | 2026-10-04 | 2421 | LZ4 block compression, verifier SMT validity, scripting temp-tree fix, compile caches |
| v0.62.4 | 2026-10-04 | 2421 | Correctness: script cache, constant tables, nested modules, match arms, unsigned compares |
| v0.62.3 | 2026-10-03 | 2411 | Correctness: shared references, contracts index, match-pattern checks |
| v0.20.0 | 2026-07-02 | 246+ | Critical safety fixes + multi-file + auto-installer |
| v0.12.0 | 2026-07-01 | 234 | Production self-hosted compiler |
| v0.11.0 | 2026-07-01 | 213 | Full body IR, if/else/while |
| v0.10.0 | 2026-06-30 | 208 | Self-hosting bootstrap |
| v0.9.0 | 2026-06-30 | 197 | E2E pipeline validation |
| v0.5.0 | 2026-06-30 | 125 | Self-hosting structural proof |
| v0.2.5 | 2026-06-30 | 109 | Full language surface |
| v0.1.0 | 2026-06-30 | 36 | First working pipeline |

Intermediate releases v0.64.1-v0.64.7 are on the [Releases page](https://github.com/xiom-lang/xiom/releases).

## Ecosystem

| Library | Purpose | Status |
|---------|---------|--------|
| `xiom-http` | HTTP client (libcurl) | [WIP] |
| `xiom-crypto` | Cryptography (OpenSSL) | [WIP] |
| `xiom-sql` | SQL database (SQLite) | [WIP] |

## Package Registry

The public registry is at `https://registry.xiom-lang.org`; `xiom-std 0.64.8` is the production pin with OIDC provenance. `xiom-std 0.64.9` is published but held with the withdrawn compiler release. For local development you can run the registry server, then publish and install packages.

```powershell
# Start the registry
python registry/server.py
# -> http://localhost:8080

# Publish a package
xiom pkg publish

# List available packages
curl http://localhost:8080/index.json

# Install a package
xiom pkg install xiom-http
```

Refuse unsigned artifacts by pinning the registry index key:
`xiom pkg trust --registry <registry-url> --index-key <hex>`; list pins with
`xiom pkg trusted`.

## Project Structure

```
XIOM/
|-- crates/           # Rust bootstrap compiler (12 crates)
|   |-- xiom/       #   Compiler CLI
|   |-- xiom-ast/    #   AST definitions
|   |-- xiom-lexer/  #   Tokenizer
|   |-- xiom-parser/ #   Recursive descent parser
|   |-- xiom-check/  #   Type checker + borrow checker
|   |-- xiom-codegen/#   LLVM IR emitter
|   `-- ...           #   fmt, doc, lsp, pkg, ffigen, verify
|-- selfhost/         # XIOM self-hosted compiler (Phase 6)
|   `-- src/main.xi   #   Main compiler source
|-- stdlib/           # Standard library
|   |-- xiom/        #   core, io, collections, string, math, ffi, async
|   `-- runtime/      #   C runtime (xiom_runtime.c)
|-- examples/         # Example programs (21)
|-- ecosystem/        # Ecosystem libraries
|   |-- xiom-http/   #   HTTP (libcurl)
|   |-- xiom-crypto/ #   Cryptography (OpenSSL)
|   `-- xiom-sql/    #   SQL (SQLite)
|-- editors/vscode/   # VS Code extension
|-- playground/       # WASM playground
|-- dist/             # Distribution packages
|-- specs/            # Language specification + build strategy
`-- docs/             # Documentation
```

## License

Dual-licensed under your choice of:

- MIT License - see `LICENSE-MIT`
- Apache License, Version 2.0 - see `LICENSE-APACHE`

Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
See `NOTICE` for attributions.
