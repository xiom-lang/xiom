<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Programming Language

**Safe - Verified - Precise** -- A systems programming language with first-class contracts.

[![Tests](https://img.shields.io/badge/tests-2%2C600%2B-brightgreen)]()
[![Version](https://img.shields.io/badge/version-0.64.8-blue)]()

XIOM is a compiled, statically typed, memory-safe systems programming language. It compiles to native code via LLVM and supports x86_64, ARM, RISC-V, and WebAssembly. The Rust compiler is the production toolchain; the self-hosted implementation is in Phase 6 (RD9: 14/64 byte-exact).

## Current Status (2026-10-20)

- **Latest release: [v0.64.8](https://github.com/xiom-lang/xiom/releases/tag/v0.64.8)** (2026-10-19), with Windows, Linux x64/arm64, macOS x64/arm64, wasm, and VSIX assets. The stdlib pin is `stdlib-v0.64.8`; the public registry has `xiom-std 0.64.8` with OIDC provenance.
- **Post-release fixes on `main`:** absolute-path package discovery, the m371 cross-module imported-struct alias layout, m372 nested `ptr.read`/`ptr.write` inference, m376 per-monomorphisation param/ref-state reset (combined Map/Set generic-helper AV), m377 scalar-pointee `ptr.read`/`ptr.write` inference + call-site pointer ABI, m2057 fn-type alias transparency (checker + codegen, including fn-typed struct fields) + module-global `Vec.pop`, m2100-W1 qualified module field types before method dispatch, m1626 semicolon-less trailing match as the block value, m2343 bare inline i32 extern sign-extension, m365 per-key dependency-cache build lock, m373 `[compiler]` `parallel`/`jobs`/`parallel-codegen` knobs, and the RISC-V LLVM triple correction. Checker suite: 213/213; the m371/m372/m376/m377/m2057/m2100/m1626/m2343/m37 e2e locks are green.
- **Self-hosting:** Phase 6 RD9 is green (full T3 suite 6/6; T6 **14/64** byte-exact). Next shapes: Option/`.len`, `Vec.pop`, `&Vec`+f64, index arithmetic.
- **Next:** a v0.64.9 candidate after lane verification; m373 product profiling (the v0.64.8 matrix is front-end-bound, with an ~85 MB peak-RSS floor and LSI 45), m374 parallel checker/CGU design, and MCU/bare-metal targets.
- **Open relay items:** orbitdb A6 1M-key soak (high). Measured on v0.64.8: the retention is Vec drop elaboration, not `io.read_at` or the arena (`read_at` 10k x 4 KiB = 304 MB; Vec[UInt8] churn alone = 317 MB/10k; the same churn in a helper is flat). Orbitdb has the direct-reader workaround; stdlib has a `read_at_into` caller-buffer wishlist (`REL-20261020-1302-compiler`), and the compiler proper fix (owned Result/Option Vec payloads at scope exit) needs design. The Map/Set generic-helper AV is fixed on `main` (m376); the Vec4f `*Float32` lane-miscompile compiler half is fixed (m377) with the element-based lane-math follow-up in stdlib (`REL-20261020-1242-compiler`). The gate-9 stdlib reds 1626/2343/2100-W1 are fixed on `main` (m1626/m2343/m2100-W1) pending reporter re-verify. m365's per-key dependency-cache lock is on `main`; the caller-side dependency-vs-consumer flag split is waiting on the engine's 117-target key-equality re-measure.

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

The self-hosted implementation lives in `selfhost/src/main.xi`. It is under active parity work (Phase 6 RD9: full T3 suite 6/6, T6 14/64 byte-exact); the Rust compiler remains the bootstrap and the production toolchain for releases.

```powershell
# Step 1: Compile the self-hosted compiler with Rust
cargo run -p xiom -- -o xiom-self.exe selfhost/src/main.xi

# Step 2: The resulting xiom-self.exe is the self-hosted compiler
.\xiom-self.exe --help

# Step 3: Use it to compile XIOM code
.\xiom-self.exe examples/demo_float.xi --run
```

### Bootstrap Chain (Historical)

The original self-hosting proof (v0.12.0, 2026-07-01) used the then-current `selfhost` source and produced a stage-2 compiler. That proof is historical. The current self-hosted implementation is `selfhost/src/main.xi`; parity is tracked in `docs/checklists/selfhost-phase6.md` (RD9: full T3 suite 6/6, T6 14/64 byte-exact). The Rust compiler remains the production bootstrap and release toolchain.

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

The public registry is at `https://registry.xiom-lang.org`; `xiom-std 0.64.8` is published there with OIDC provenance. For local development you can run the registry server, then publish and install packages.

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
