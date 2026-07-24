# XIOM Compiler Architecture

**Version:** v0.50.0 "Production Edition"
**Date:** 2026-07-25
**Branch:** `feat/architect`
**Tests:** 1041/1041 all passing (681 compiler + 360 tooling)
**Status:** Production-hardened — all M1-M12 phases complete

This document is the authoritative reference for the XIOM compiler architecture,
pipeline, compilation modes, CLI flags, crate structure, and runtime model.

---

## 1. Pipeline Overview

```
                    .xi Source File
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                      XIOM COMPILER                           │
│                                                              │
│  ┌────────┐   ┌────────┐   ┌─────────┐   ┌───────────────┐ │
│  │ Lexer  │──▶│ Parser │──▶│ Checker │──▶│ Borrow Checker│ │
│  │(xiom-  │   │(xiom-  │   │(xiom-   │   │(xiom-check/   │ │
│  │ lexer) │   │ parser)│   │ check)  │   │ borrow/)       │ │
│  └────────┘   └────────┘   └─────────┘   └───────┬───────┘ │
│                                                   │         │
│                                          ┌────────▼────────┐│
│                                          │    Codegen       ││
│                                          │ (xiom-codegen)   ││
│                                          │ AST → LLVM IR    ││
│                                          └────────┬────────┘│
│                                                   │         │
│                              ┌────────────────────▼───────┐ │
│                              │  LLVM IR (.ll)             │ │
│                              │  ; XIOM v0.50.0 LLVM IR   │ │
│                              └────────────────────┬───────┘ │
│                                                   │         │
│  ┌────────────────────────────────────────────────┼───────┐ │
│  │              COMPILATION TARGETS               │       │ │
│  │                                                │       │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐ │       │ │
│  │  │ Native   │  │   WASM   │  │  ARM / RISC-V│ │       │ │
│  │  │ clang →  │  │ clang →  │  │ cross-clang  │ │       │ │
│  │  │ .exe/.out│  │  .wasm   │  │   → .out     │ │       │ │
│  │  └──────────┘  └──────────┘  └──────────────┘ │       │ │
│  └────────────────────────────────────────────────┘       │ │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              SCRIPTING / JIT MODE (xiom run)         │   │
│  │  Lex → Parse → Check → IR → clang -shared → .dll    │   │
│  │  → libloading::Library::new() → main() in-process   │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

## 2. Compilation Modes

### 2.1 Standard AOT (`xiom file.xi`)

Full ahead-of-time compilation. Produces a native binary.

```
xiom main.xi -o app.exe           # Windows
xiom main.xi -o app               # Linux/macOS
xiom main.xi --release            # Optimized build (LLVM -O3)
xiom main.xi --target wasm        # WASM target
```

### 2.2 Scripting Mode (`xiom run`)

Zero-boilerplate execution. Auto-wraps top-level code in `fn main()` and adds `use xiom.io;`.

```bash
xiom run script.xi                # Execute a script file
xiom run -e "io.println(42)"      # Inline expression
xiom run -                        # Read from stdin
xiom run --watch script.xi        # Watch and re-run on changes
xiom run --jit script.xi          # True JIT via shared library loading
```

**Implicit main wrapping:**
```xiom
// User writes:                       // Compiler treats as:
io.println("hello");                  use xiom.io;
                                      fn main() { io.println("hello"); }
```

**IMPORTANT:** `xiom run` does NOT relax type checking. Only adds `use xiom.io;` and wraps in `fn main()`.

### 2.3 Interactive REPL (`xiom repl`)

```bash
xiom> var x = 42
xiom> io.println(x.to_str())
42
xiom> :vars                        # Show accumulated state
  var x = 42
xiom> :reset                       # Clear state
xiom> :list io                     # Show module functions (M13)
xiom> :quit
```

### 2.4 Check-Only (`xiom --check`)

Type-checks without producing a binary. Used in CI and IDEs.

```bash
xiom --check file.xi               # Type-check only
xiom --check --diagnostics-json file.xi  # Machine-readable output
```

### 2.5 Emit IR (`xiom --emit-ir`)

Prints LLVM IR to stdout. Used for debugging and playground.

```bash
xiom --emit-ir file.xi             # Print IR to stdout
xiom --emit-ir --target wasm file.xi  # WASM IR
```

### 2.6 Standalone Build (`xiom --standalone`)

Converts a script to a production binary. Graduates scripting code to AOT.

```bash
xiom --standalone script.xi -o mytool.exe
xiom --standalone --scaffold script.xi   # Also create project structure
```

### 2.7 Package Manager (`xiom pkg`)

```bash
xiom pkg install <name>            # Install from registry
xiom pkg search <query>            # Search packages
xiom pkg publish                   # Publish to registry
```

---

## 3. Complete CLI Flags

| Flag | Mode | Description |
|------|------|-------------|
| `-o <path>` | All | Output binary path |
| `--release` | AOT | Optimized build (LLVM -O3) |
| `--target <t>` | AOT | Target: native (default), wasm, arm, riscv |
| `--check` | All | Type-check only, no binary |
| `--emit-ir` | All | Print LLVM IR to stdout |
| `--emit-tokens` | All | Print token stream (lexer debug) |
| `--diagnostics-json` | All | JSON-formatted diagnostics |
| `--standalone` | AOT | Script-to-binary conversion |
| `--scaffold` | AOT | Create project directory structure |
| `--watch` | Scripting | File watcher (with `xiom run`) |
| `--jit` | Scripting | True in-process JIT execution |
| `--clean` | — | Remove build artifacts |
| `--clean --cache` | — | Clear JIT script cache |
| `--verify` | AOT | Generate SMT-LIB for Z3 |
| `--no-contracts` | AOT | Disable contract runtime checks |
| `--runtime-contracts` | AOT | Force contracts in release builds |
| `--sanitize=<type>` | AOT | Enable sanitizer (address, undefined, leak, thread) |
| `--stack-protector` | AOT | Enable stack canaries |
| `--shared` | AOT | Produce shared library (.dll/.so) |
| `--static` | AOT | Produce static library (.lib/.a) |
| `--hot-reload` | AOT | Generate hot-reload manifest |
| `--incremental` | AOT | Incremental compilation |
| `--parallel` | AOT | Parallel compilation |
| `--jobs <N>` | AOT | Number of parallel jobs |
| `--max-depth <N>` | AOT | Recursion depth limit (default 2000) |
| `--strict` | AOT | Strict mode (error on unknown types) |
| `--debug` | AOT | Compile with debug symbols (-g) |
| `--time` | All | Print compilation timing |
| `--verbose` | All | Verbose output |
| `--explain <code>` | — | Explain error code (e.g. --explain T001) |
| `--version` | — | Print version |
| `--help` | — | Print help |

### Subcommands

| Command | Description |
|---------|-------------|
| `xiom run <file>` | Scripting execution |
| `xiom run -e "code"` | Inline expression |
| `xiom run -` | Read from stdin |
| `xiom run --watch <file>` | Watch and re-run |
| `xiom run --jit <file>` | True JIT execution |
| `xiom repl` | Interactive REPL |
| `xiom pkg install <name>` | Install package |
| `xiom pkg search <query>` | Search packages |
| `xiom build <dir>` | Build project (package.xi) |
| `xiom doctor` | Check toolchain dependencies |

---

## 4. Crate Architecture

```
xiom (CLI + Library)
├── xiom-ast        — AST definitions, types, expressions, statements
├── xiom-lexer      — Lexer (tokenizer), shebang support
├── xiom-parser     — LL(1) recursive descent parser
├── xiom-check      — Type checker, borrow checker, module catalog
│   ├── borrow/     — Borrow checking (loans, places)
│   └── compat/     — Compatibility/coercion logic
├── xiom-codegen    — LLVM IR code generation
│   ├── context.rs  — CodegenConfig, TypeContext, FunctionContext, MonoContext, LocalContext
│   ├── call.rs     — Function/method call compilation
│   ├── stmt.rs     — Statement compilation
│   ├── expr.rs     — Expression compilation
│   ├── decl.rs     — Declaration compilation
│   ├── types.rs    — Type resolution
│   ├── coerce.rs   — Value coercion
│   ├── contracts.rs— Contract guard emission
│   ├── vec_abi.rs  — Vec ABI primitives
│   ├── enum_ctors.rs— Enum constructor generation
│   ├── emitter.rs  — IR output helpers
│   ├── sandbox.rs  — Safety auditor
│   ├── jit.rs      — JIT via libloading
│   └── call.rs     — Function call compilation (extracted from expr.rs)
├── xiom-graph      — Project dependency graph, manifest parsing
├── xiom-verify     — SMT-LIB generation, Z3 contract verification
├── xiom-fmt        — XIOM source formatter
├── xiom-display    — Type/fn signature display utilities
├── xiom-doc        — Documentation generator
├── xiom-lsp        — Language Server Protocol (10 modules)
│   ├── backend     — Document storage + diagnostics
│   ├── transport   — JSON-RPC I/O
│   ├── handlers    — LSP method handlers
│   ├── resolver    — Type resolution, symbol lookup
│   ├── symbols     — Document/workspace symbols
│   ├── semantic_tokens — Syntax highlighting
│   └── uri, diagnostics, ai, text_edit
├── xiom-mcp        — MCP server (Model Context Protocol)
├── xiom-pkg        — Package manager (install, search, publish)
├── xiom-dbg        — Debug adapter protocol server (GDB/CDB backends)
├── xiom-ffigen     — FFI bindings generator
├── xiom-wasm       — WASM compiler for playground (browser)
└── xiom-playground — Playground server (Node.js), 372 lessons
```

### Crate Line Counts (v0.50.0)

| Crate | Lines | Rating |
|-------|-------|--------|
| xiom-display | 121 | 8.0 |
| xiom-graph | ~1,800 (7 files) | 8.0 |
| xiom-ast | 604 | 7.0 |
| xiom-lexer | 503 | 7.0 |
| xiom-check | 4,415 (7 files) | 7.0 |
| xiom-parser | 1,851 | 7.0 |
| xiom-codegen | ~13,000 (13 files) | 7.0 |
| xiom | 2,030 + tests | 7.0 |
| xiom-lsp | 2,850 (10 files) | 7.0 |
| xiom-fmt | 1,123 | 5.0 |
| xiom-pkg | 1,102 | 5.0 |
| xiom-mcp | 1,049 | 5.0 |
| xiom-dbg | 1,027 | 6.0 |
| xiom-ffigen | ~400 | 6.0 |
| xiom-verify | 863 | 5.0 |
| xiom-wasm | 142 | 6.0 |
| xiom-doc | ~300 | 5.0 |

---

## 5. LLVM IR Codegen Details

### Type Mapping

| XIOM Type | LLVM IR Type |
|-----------|-------------|
| `Int` | `i64` |
| `Int8`-`Int64` | `i8`-`i64` |
| `UInt`-`UInt64` | `i64` |
| `Float32` | `float` |
| `Float64` | `double` |
| `Bool` | `i1` (stored as `i64` in structs) |
| `Str` | `i8*` |
| `Char` | `i32` |
| `*T` | `i64` (pointer stored as integer) |
| `&T` | `i64` |
| `Option[T]` | `{ i64, i64 }` (discriminant, value) |
| `Result[T,E]` | `{ i64, i64, i64 }` (discriminant, value, error) |
| `Vec[T]` | `{ i8*, i64, i64, i64 }` (data ptr, len, cap, elem_size) |
| `struct` | `%struct.Name { field_types... }` |
| `enum` | `{ i64, i64 }` (discriminant + payload union as i64) |
| `fn()` | `i8*` (function pointer) |

### Codegen Architecture (M4.1 — God Object Decomposed)

```
IrEmitter
├── output: String           # Accumulated LLVM IR text
├── tmp_counter, block_counter, str_counter
├── config: CodegenConfig    # Target triple, contract mode, hot-reload
├── types: TypeContext       # Type registry, interface/enum metadata
├── fctx: FunctionContext    # Per-function state (locals, params, return)
├── mono: MonoContext        # Monomorphisation worklist
└── local: LocalContext      # Variable classification, globals, loop stack
```

---

## 6. Scripting & JIT Architecture

```
xiom run script.xi
        │
        ▼
┌───────────────────┐
│  Shebang strip    │  #!/usr/bin/env xiom → skip line
│  (xiom-lexer)     │
└───────┬───────────┘
        ▼
┌───────────────────┐
│  Implicit main    │  Top-level code → fn main() { ... }
│  wrap             │  Auto-adds use xiom.io;
│  (implicit_main)  │  Declarations stay at top level
└───────┬───────────┘
        ▼
┌───────────────────┐
│  Standard compile │  Lex → Parse → Check → IR → clang
│  pipeline         │
└───────┬───────────┘
        ▼
┌───────────────────┐
│  JIT: --jit flag  │  clang -shared → .dll/.so
│                   │  libloading → main() in-process
│  AOT: default     │  Compile to temp .exe, execute
│  Cache: automatic │  Content-hash → ~/.xiom/jit/
└───────────────────┘
```

### Script Cache

| Feature | Detail |
|---------|--------|
| Location | `~/.xiom/jit/` |
| Key | Content-hash (SHA-like) of source |
| Max size | 100 MB (LRU eviction) |
| Clean | `xiom clean --cache` |

---

## 7. LSP Architecture

```
VS Code / IDE
     │  JSON-RPC (stdin/stdout)
     ▼
┌─────────────────────────────────────────────┐
│                 xiom-lsp                     │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
│  │ transport│  │ handlers │  │  resolver  │ │
│  │ (I/O)    │  │(dispatch)│  │(type/sym)  │ │
│  └──────────┘  └──────────┘  └───────────┘ │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐ │
│  │ backend  │  │ symbols  │  │semantic_   │ │
│  │(docs,diag)│  │(doc/ws) │  │ tokens     │ │
│  └──────────┘  └──────────┘  └───────────┘ │
│                                              │
│  ┌──────────┐  ┌──────────┐                │
│  │    ai    │  │  text_   │                │
│  │(insights)│  │  edit    │                │
│  └──────────┘  └──────────┘                │
└─────────────────────────────────────────────┘
     │
     ▼
┌─────────┐
│  GDB/MI │──→ debugged process
└─────────┘
```

### LSP Capabilities

| Feature | Status | Tests |
|---------|--------|-------|
| Diagnostics (parse + type errors) | ✅ | 11 |
| Hover (type info, function signatures) | ✅ | — |
| Completion (keywords, snippets, symbols) | ✅ | — |
| Go-to-definition | ✅ | — |
| Signature help | ✅ | — |
| Document symbols | ✅ | — |
| Workspace symbols | ✅ | — |
| References | ✅ | — |
| Rename | ✅ | — |
| Semantic tokens | ✅ | — |
| Code actions (quickfix) | ✅ | — |
| Stdlib completion | ❌ M13.1 | — |
| Formatting | ❌ M13.3 | — |

---

## 8. Test Gates (v0.50.0)

| Suite | Count | Status |
|-------|-------|--------|
| E2E | 112 | OK |
| Feature Regression | 268 | OK |
| Stdlib Execution | 41 | OK |
| Diff | 25 | OK |
| Full-Diff | 23 | OK |
| Fuzz | 24 | OK |
| Integration | 119 | OK |
| Robustness | 29 | OK |
| Stdlib Compilation | 40 | OK |
| Checker | 123 | OK |
| Parser | 58 | OK |
| Formatter | 41 | OK |
| LSP | 11 | OK |
| Package Manager | 15 | OK |
| Doc Generator | 4 | OK |
| FFI Generator | 18 | OK |
| MCP Server | 18 | OK |
| Debugger | 8 | OK |
| Verifier | 15 | OK |
| Scripting | 34 | OK |
| Script Diff | 15 | OK |
| **TOTAL** | **1041** | **ALL GREEN** |

---

## 9. Memory & Safety Model

### Ownership Rules (compile-time enforced)

| Rule | Effect |
|------|--------|
| Single owner | Assignment moves ownership. Old binding invalid. |
| Scope lifetime | Value freed at end of owning scope. |
| `&T` (read borrow) | Multiple simultaneous. No mutation. |
| `&mut T` (write borrow) | Exactly one. No other borrows. |
| `.clone()` | Explicit deep copy. |
| Move on call | Pass by value moves. Use `&` to borrow. |

### Contract System

```xiom
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures:  result * b == a
{ return a / b; }
```

| Mode | Behavior |
|------|----------|
| Default | Runtime guards — `@llvm.trap()` on violation |
| `--no-contracts` | Strips all checks |
| `--verify` | Generates SMT-LIB for Z3 (xiom-verify) |

---

## 10. Crate Dependency Graph

```
                    xiom (CLI + Library)
                   /    |     |     |    \
                  /     |     |     |     \
         xiom-ast  xiom-lexer xiom-parser xiom-check xiom-codegen
            |         |         |         |           |
            └─────────┴─────────┴─────────┴───────────┘
                              |
                    xiom-graph xiom-verify xiom-fmt xiom-display
                              |
                    xiom-lsp xiom-mcp xiom-pkg xiom-dbg xiom-ffigen xiom-doc xiom-wasm
```

---

## Appendix A: M Phase Completion Status

| Phase | Status | Key Deliverables |
|-------|--------|-----------------|
| M1 | ✅ | Quick wins |
| M2 | ✅ | 113+ stdlib contracts across 10 modules |
| M3.2-M3.4 | ✅ | 25 AST round-trip tests, compile_with_diagnostics tests |
| M4.1-M4.7 | ✅ | IrEmitter split, expr.rs split, LSP split, 0 unwraps, 0 exits, SAFETY docs |
| M5 | ✅ | 22 property-based checker tests |
| M7 | ✅ | From/Into (16 conversions), Deref/DerefMut, PhantomData, MaybeUninit |
| M9 | ✅ | 11/11 language gaps closed |
| M10 | ✅ | Scripting: run, standalone, repl, watch, shebang, JIT, 49 tests |
| M11 | ✅ | Cache eviction, CI, MCP scripting guide, release prep |
| M12 | ✅ | script_mode, IR fix, garbled fix, auto stdlib discovery |

### Remaining Phases

| Phase | Scope | Effort | Target |
|-------|-------|--------|--------|
| M13 | LSP 10/10 (stdlib completion, lsp-types, formatting, workspace index) | 7.5d | v0.51.0 |
| M14 | Production cleanup (14 file splits, 8 function decompositions, 16 unwrap fixes, 49 docs) | 10d | v0.51.0 |

---

## Appendix B: Key Files Map

| File | Purpose |
|------|---------|
| `docs/ROADMAP.md` | Full M phase roadmap |
| `docs/SESSION.md` | Session handoff — current state |
| `docs/AI_CONTEXT.md` | Language spec (40 stdlib modules, all flags) |
| `docs/RELEASE_PROCESS.md` | Release packaging instructions |
| `docs/M10_SCRIPTING_MODE.md` | Scripting/JIT design document |
| `crates/xiom/src/main.rs` | CLI entry point, all flags and subcommands |
| `crates/xiom/src/lib.rs` | Library API — compile, compile_with_diagnostics |
| `crates/xiom/src/implicit_main.rs` | Scripting wrapper |
| `crates/xiom/src/jit.rs` | JIT via libloading, cache |
| `crates/xiom-codegen/src/context.rs` | M4.1 sub-contexts |
| `crates/xiom-codegen/src/call.rs` | Call compilation (extracted from expr.rs) |
| `crates/xiom-codegen/src/stmt.rs` | Statement compilation (extracted from expr.rs) |
| `crates/xiom-lsp/src/` | 10 LSP modules |
| `xiom-playground/server.js` | Playground server |
| `stdlib/xiom/` | 40-module standard library |
| `test_summary.ps1` | Full test suite runner |
| `.github/workflows/ci.yml` | CI for Win/Linux/macOS |
