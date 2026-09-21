<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Self-Hosting Readiness Audit -- v0.52.9

**Date:** 2026-07-27 | **Test baseline: 1067/1067** | **Auditor: Kilo (automated deep audit)**

---

## EXECUTIVE SUMMARY

**Honest Rating: 3/10 for TRUE self-hosting. 8/10 for the Rust-hosted compiler.**

The SESSION.md claims "Self-hosting readiness: 10/10". This is inaccurate.
The Rust-hosted compiler (xiom.exe) is production-grade and can compile any XIOM
source file, including all selfhost/*.xi attempts. However, the XIOM-written
compiler in selfhost/ cannot compile itself WITHOUT the C runtime doing the
actual codegen work. True self-hosting requires the XIOM compiler, written in
XIOM, to compile itself end-to-end with no C dependency for codegen.

### What "10/10 Self-Hosting" Would Mean
1. A XIOM source file (selfhost/xiomc.xi) implements lexer, parser, type checker,
   and LLVM IR emitter entirely in XIOM.
2. Running `xiom selfhost/xiomc.xi -o xiomc.exe` produces a native binary.
3. Running `xiomc.exe selfhost/xiomc.xi -o xiomc2.exe` produces an identical binary.
4. The self-hosted compiler passes the same test suite.
5. No C runtime body-parsing or IR emission is used.

### Where We Actually Are
- **The selfhost/ directory has 23 XIOM source files** implementing lexer, parser,
  checker, and codegen components. All compile successfully with the Rust-hosted
  compiler.
- **v10/v11 compilers read real .xi files** via C FFI and extract function signatures.
- **BUT: body-level parsing and ALL LLVM IR emission is done in C** (~1800 lines
  in `stdlib/runtime/xiom_runtime.c`, function `emit_body_ir()`).
- The XIOM portion is essentially a function-signature extractor + metadata feeder.
- The canonical `xiomc.xi` (672 lines) has **hardcoded IR strings** printed via
  `io.println()` -- it does NOT generate IR programmatically.

---

## DIMENSION 1: LANGUAGE FEATURE COVERAGE -- 7/10

### SUPPORTED (sufficient for self-hosting)
| Feature | Status |
|---------|--------|
| Structs with named fields | Full |
| Enums (tagged unions) with payloads | Full (M19 fix applied) |
| Pattern matching (match, if let, while let) | Full |
| Generics with monomorphisation | Full |
| Modules and imports (use) | Full |
| Type aliases | Full |
| Option / Result with `?` operator | Full |
| While / For / If-elif-else loops | Full |
| Recursion | Full |
| const / static declarations | Full |
| FFI (extern "C" blocks) | Full |
| Unsafe blocks | Full |
| Derive macros (Eq, Clone, Display, Hash, Ord, Debug) | Full |
| Borrow checker (ownership/move semantics) | Full |
| Function pointer types (`fn(T) -> U`) | Full |

### PARTIAL (usable but limited)
| Feature | Limitation |
|---------|-----------|
| Interfaces (traits) | Structural satisfaction only -- no `impl Trait for Type` syntax. Method name coincidence is the only way to satisfy an interface. |
| Or-patterns in match | Parser supports `A \| B`, checker validates, but codegen has `TODO` -- not compiled yet |
| Const-generics | Basic support but size inference relies on heuristics |

### MISSING (hard blockers)
| Feature | Impact on Self-Hosting |
|---------|----------------------|
| **Closures** | `Expr::Closure` and `Expr::PipeClosure` compile to constant `0` (expr.rs:1765). No capture environment, no callable object. **This is the #1 language blocker.** A compiler needs closures for AST visitors, iterator chains, and error-handling combinators. Without closures, all functional patterns must be rewritten as explicit recursive functions with manual state threading. |
| **`impl Trait for Type`** | No explicit interface implementation blocks. Cannot implement a shared trait across multiple types in different modules. Limits organized type hierarchies needed for AST node types. |

---

## DIMENSION 2: SELF-HOSTING COMPILER STATUS -- 2/10

### What Exists (23 files in selfhost/)
| Component | File | Status |
|-----------|------|--------|
| Lexer | `xiom-lexer.xi` (508 lines) | Works with hardcoded source |
| Parser | `xiom-parser.xi` (392 lines) | Works with hardcoded token stream |
| Type Checker | `xiom-check.xi` (205 lines) | Works with hardcoded test data |
| Codegen (minimal) | `xiom-codegen.xi` (45 lines) | LLVM type mapping only, no IR emission |
| Integrated compiler | `xiomc.xi` (672 lines) | All phases, hardcoded IR strings |
| v10 compiler | `xiomc_v10.xi` (291 lines) | Reads real files, delegates codegen to C |
| v11 compiler | `xiomc_v11_test.xi` (219 lines) | Refined v10, targets demo_float.xi |

### What's Missing for True Self-Hosting
| Gap | Lines in C | Must be Rewritten in XIOM |
|-----|-----------|--------------------------|
| Body-level parser (expressions, statements, types) | ~1800 | `emit_body_ir()` in xiom_runtime.c |
| LLVM IR emitter (SSA builder, type lowering) | ~1800 | Same C function |
| AST data structures (tree nodes, visitors) | 0 | Must be designed in XIOM |
| Semantic analysis (type inference, scope) | 0 | Checker stub returns 0 |
| Borrow checker | 0 | Checker stub returns 0 |
| Module resolver | 0 | Not implemented in XIOM |

### The v10/v11 "Self-Hosting" Reality
```
xiomc_v10.xi does:
  1. xiom_read_file(path) -> raw source bytes (C FFI)
  2. Scan for "fn" keyword, extract name/signature
  3. Record metadata in C function table (xiom_fn_table_add)
  4. Call xiom_fn_emit_all() -> C runtime does ALL body parsing + IR emission
```

The XIOM code is a thin shell. **Every byte of LLVM IR comes from C.**

---

## DIMENSION 3: RUST-HOSTED COMPILER -- 8/10

This is the compiler users actually run (`xiom.exe`). It is production-grade.

### Strengths
| Area | Assessment |
|------|-----------|
| Test coverage | 1067 tests across 21 suites, all green |
| Codegen correctness | Structs, enums, generics, FFI, recursion all verified |
| Error handling | Contracts, panic trapping, recursion depth limits |
| Toolchain | LSP, formatter, doc gen, FFI gen, package manager, debugger, verifier, MCP server |
| Cross-platform | Windows (MSVC) + Linux (GNU) release binaries |
| Performance | Release builds optimized; WASM target supported |
| Zero critical bugs | M20 has no open priority bugs |

### Weaknesses
| Area | Gap |
|------|-----|
| Closures | Not implemented in codegen (compile to constant 0) |
| Or-patterns | Not compiled (TODO) |
| impl Trait for Type | Not supported |
| Self-hosting | C-dependent, not true self-hosting |

---

## DIMENSION 4: STDLIB -- 7/10

### Complete (self-hosting ready)
- File I/O: read_file, write_file, file_exists, remove_file, create_dir, list_dir
- String ops: len, concat, slice, split, trim, starts_with, ends_with, contains, replace, upper/lower, lines, words, index_of, char_at, byte_at, format
- Collections: Vec (new, push, pop, len, get, with_capacity, clone), Map, Set
- FFI: Full extern "C" support with pointer types
- Error handling: Option, Result with is_some/is_ok/is_err/unwrap
- Math: basic arithmetic, float operations
- Command-line: args, get_argc, get_argv
- Memory: malloc/free via FFI, ptr operations (read, write, offset, is_null)
- Time: time() via FFI

### Gaps
| Area | Gap |
|------|-----|
| HashMap/HashSet | Map/Set exist but hashing may be limited |
| Sorting | Not in stdlib |
| Regular expressions | Not in stdlib |
| Networking | In packages/xiom-http, not core stdlib |
| JSON | In packages/xiom-json, not core stdlib |

---

## DIMENSION 5: TOOLCHAIN -- 9/10

| Tool | Status |
|------|--------|
| xiom (compiler) | Production-grade |
| xiom-fmt (formatter) | Working |
| xiom-doc (doc generator) | Working |
| xiom-ffigen (FFI generator) | Working |
| xiom-pkg (package manager) | Working |
| xiom-lsp (language server) | Working |
| xiom-mcp (MCP server for AI) | Working |
| xiom-dbg (debugger) | Working |
| xiom-verify (contract verifier) | Working |
| xiom run (scripting mode) | Working |
| xiom doctor (dependency checker) | Working |

---

## HONEST VERDICT

| Dimension | Rating | Note |
|-----------|--------|------|
| Language features | 7/10 | Closures and impl Trait missing |
| Self-host compiler | 2/10 | C-dependent shell, not real self-hosting |
| Rust-hosted compiler | 8/10 | Production-grade, 1067 tests, zero critical bugs |
| Standard library | 7/10 | Solid core, some advanced collections missing |
| Toolchain | 9/10 | Comprehensive, all tools working |
| **OVERALL** | **6.6/10** | Weighted toward Rust compiler (what users run) |
| **TRUE SELF-HOSTING** | **3/10** | Hard blockers: closures, C-dependent codegen |

---

## M20 TASK LIST: PATH TO TRUE SELF-HOSTING (10/10)

### Updated Honest Projection
| After | Rating | What changes |
|-------|--------|-------------|
| **Today** | 3/10 | C-dependent codegen, no closures |
| **After M20-A** | 5/10 | Closures + impl Trait + Self type unlock functional patterns |
| **After M20-B** | 8/10 | XIOM-native compiler exists, C runtime removed from codegen |
| **After M20-C** | **10/10** | Self-compile, round-trip, stdlib compilation, test parity |

### Phase M20-A: Language Feature Completion (weeks 1-2)
| # | Task | Priority |
|---|------|----------|
| M20-A1 | **Closure codegen**: stack-capturing lambdas (FnOnce semantics), lower to struct + function pointer, emit callable LLVM IR. Currently `Expr::Closure` compiles to constant `0` (expr.rs:1765). | **CRITICAL** |
| M20-A2 | **`impl Trait for Type` syntax**: parser, checker registration, codegen vtable or monomorphised dispatch. Currently interfaces are structural-only (name coincidence). | HIGH |
| M20-A3 | Fix or-pattern codegen (stmt.rs TODO at line 641) | MEDIUM |
| M20-A4 | **`Self` type in trait methods**: `fn clone() -> Self` is essential for reusable traits. Without it, every trait method must name the concrete type, making traits non-reusable across types. | HIGH |

### Phase M20-B: Self-Host Compiler Rewrite (weeks 3-6)
| # | Task | Priority |
|---|------|----------|
| M20-B1 | Design AST data structures in XIOM: Token, Expr, Stmt, Type, TopDecl enums with payloads | **CRITICAL** |
| M20-B2 | Rewrite body parser in XIOM: expression parsing, statement parsing, type parsing, operator precedence | **CRITICAL** |
| M20-B3 | Implement LLVM IR emitter in XIOM: SSA builder, type lowering, function emission, struct/enum layout | **CRITICAL** |
| M20-B4 | Implement semantic analysis: type checker, symbol resolution, scope management | HIGH |
| M20-B5 | Implement borrow checker in XIOM | MEDIUM |
| M20-B6 | Remove C runtime dependency for codegen (keep only for system calls) | HIGH |
| M20-B7 | **Generics + monomorphisation in XIOM**: the current selfhost only handles simple function signatures. True self-hosting requires the XIOM compiler to monomorphise generics (the Rust compiler currently does this). Estimated ~500 lines of additional XIOM code. | HIGH |

### Phase M20-C: Bootstrapping & Validation (weeks 7-8)
| # | Task | Priority |
|---|------|----------|
| M20-C1 | Self-compile: xiom.exe compiles xiomc.xi -> xiomc.exe | **CRITICAL** |
| M20-C2 | Round-trip: xiomc.exe compiles xiomc.xi -> xiomc2.exe, binary identical | **CRITICAL** |
| M20-C3 | Self-host test suite: xiomc passes all test suites | **CRITICAL** |
| M20-C4 | Performance parity: self-hosted compiler within 2x of Rust compiler | MEDIUM |
| M20-C5 | **Stdlib self-compilation**: the selfhosted compiler must also compile the stdlib (io.xi, string.xi, collections, etc.) and produce correct binaries. Without this, you have a compiler that can only compile itself, not real programs. | **CRITICAL** |

---

## M20 TEST PLAN: ~50 new tests needed

### Test Gap 1: Differential tests for self-hosted output (20 tests)
Expand `full_diff_tests.rs` to test EVERY language feature through the selfhost compiler path, comparing output against the Rust compiler.

### Test Gap 2: Closure tests (10 tests)
| # | Test |
|---|------|
| T-A1-1 | Stack-capturing closure (capture by value) |
| T-A1-2 | Closure as function argument |
| T-A1-3 | Closure returning value |
| T-A1-4 | Nested closures |
| T-A1-5 | Closure in match arms |
| T-A1-6 | Closure with generic parameters |
| T-A1-7 | Multiple closures in same scope |
| T-A1-8 | Closure capturing struct field |
| T-A1-9 | Closure chain (return closure from function) |
| T-A1-10 | Pipe closure (`|x, y| expr`) syntax |

### Test Gap 3: `impl Trait` + Self type tests (5 tests)
| # | Test |
|---|------|
| T-A2-1 | Basic `impl Trait for Type` block |
| T-A2-2 | Multiple impls for same trait |
| T-A2-3 | Generic impls (`impl[T] Trait for Vec[T]`) |
| T-A2-4 | Trait bounds on functions (`where T: Trait`) |
| T-A2-5 | `Self` type resolution in trait methods |

### Test Gap 4: Bootstrap tests (5 tests)
| # | Test |
|---|------|
| T-C1 | `xiom selfhost/xiomc.xi -o xiomc.exe` compiles successfully |
| T-C2 | `xiomc.exe selfhost/xiomc.xi -o xiomc2.exe` produces identical binary |
| T-C3 | `xiomc2.exe` passes a subset of the test suite |
| T-C4 | Selfhost compiles stdlib correctly |
| T-C5 | Selfhost compiles a real-world program (md_to_html.xi) |

### Test Gap 5: Selfhost regression tests (10 tests)
Every M19 bug fix re-tested through the selfhost compiler path:
| # | Test |
|---|------|
| T-B-1 | read_file content verification via selfhost |
| T-B-2 | Enum variant field collision via selfhost |
| T-B-3 | Result[Str, E].unwrap() via selfhost |
| T-B-4 | ptr.offset() inline via selfhost |
| T-B-5 | *deref on ptrtoint'd pointer via selfhost |
| T-B-6 | Large markdown -> HTML via selfhost |
| T-B-7 | String concatenation heavy load |
| T-B-8 | Recursive functions |
| T-B-9 | Generic function monomorphisation |
| T-B-10 | FFI extern calls via selfhost |

### Projected test count
| Phase | Current | After |
|-------|---------|-------|
| Today | 1067 | -- |
| M20-A complete | 1067 | 1082 (+15: closures + impl Trait) |
| M20-B complete | 1082 | 1112 (+30: selfhost regression + differential) |
| M20-C complete | 1112 | 1117 (+5: bootstrap) |
| **FINAL** | **1067** | **~1117** |

---

## CURRENT SESSION.md CORRECTION NEEDED

SESSION.md line 4: `Self-hosting readiness: 10/10` -> should be `3/10`
with honest notes about the C runtime dependency and missing language features.
