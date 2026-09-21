<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM -- Scaling Architecture (10K+ Files, Millions of Lines)

**Version:** Design Spec v0.2 | **Target:** Post-Selfhost (v0.58-v0.60)
**Status:** DESIGN ONLY -- no implementation until selfhost complete. v0.2 incorporates the pre-selfhost architecture review (Sealed Generics/Pre-Mono Table, Layout Hash, Compiler Daemon -- see S12).

---

## 0. Problem Statement

XIOM v0.56 is a single-translation-unit compiler. All source files are merged into one AST, compiled to one LLVM IR text buffer, and emitted as one object file. This works for selfhost scale (~50K LOC) but fails at 10K files / 1M+ LOC:

| Bottleneck | Limit | Root Cause |
|-----------|-------|------------|
| Memory | ~100 MB heap at 50K LOC -> ~2 GB at 1M LOC | Full AST in memory |
| Stack | ~1000 nesting levels | Recursive-descent parser + checker |
| Codegen | Single String buffer for entire .ll file | No streaming |
| Build time | Full rebuild on any change | No incremental compilation |
| Parallelism | Only per-function codegen | No parallel parse/check across files |

---

## 1. Target Architecture

### 1.1 High-Level Pipeline (Post-Scale)

```
                              .xi Source Tree
                                   |
                    +--------------+--------------+
                    |              |              |
              +-----v-----+  +-----v-----+  +-----v-----+
              |  Parser   |  |  Parser   |  |  Parser   |   (Parallel per-file)
              |  file1.xi |  |  file2.xi |  |  fileN.xi |
              `-----+-----+  `-----+-----+  `-----+-----+
                    |              |              |
              +-----v-----+  +-----v-----+  +-----v-----+
              |  Checker  |  |  Checker  |  |  Checker  |   (Parallel per-file)
              |  file1.xi |  |  file2.xi |  |  fileN.xi |
              `-----+-----+  `-----+-----+  `-----+-----+
                    |              |              |
              +-----v-----+  +-----v-----+  +-----v-----+
              |  Codegen  |  |  Codegen  |  |  Codegen  |   (Parallel per-file)
              |  file1.ll |  |  file2.ll |  |  fileN.ll |
              `-----+-----+  `-----+-----+  `-----+-----+
                    |              |              |
              +-----v-----+  +-----v-----+  +-----v-----+
              | clang -c  |  | clang -c  |  | clang -c  |   (Parallel per-object)
              | file1.o   |  | file2.o   |  | fileN.o   |
              `-----+-----+  `-----+-----+  `-----+-----+
                    |              |              |
                    `--------------+--------------+
                                   |
                          +--------v--------+
                          |   Linker         |   (clang/lld -- single pass)
                          |   final binary   |
                          `-----------------+
```

### 1.2 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **File = compilation unit** | One .xi file -> one .o object. Same granularity as C/C++. |
| **Separate type checking per file** | Files check independently; cross-file type errors caught at link time. |
| **Keep LLVM IR text emission** | Still text-based (no llvm-sys dependency). Each file produces its own .ll. |
| **clang -c per object** | Same backend. Parallelizes naturally. |
| **Use system linker (lld/clang)** | No custom linker. Emit standard .o files. |

---

## 2. Separate Compilation Units

### 2.1 File -> Object Mapping

```
project/
  src/
    main.xi          ->  main.o       (entry point)
    lexer.xi         ->  lexer.o       (selfhost)
    parser.xi        ->  parser.o      (selfhost)
    check.xi         ->  check.o       (selfhost)
    codegen.xi       ->  codegen.o     (selfhost)
    ...
  lib/
    xiom/std/io.xi   ->  io.o          (stdlib)
    xiom/std/math.xi ->  math.o        (stdlib)
```

### 2.2 Compilation Command

```bash
# Individual file compilation (new -c flag)
xiom -c src/lexer.xi -o build/lexer.o

# Build entire project (new --build flag)
xiom build                          # reads xiom.toml / package.xi
xiom build --jobs=$(nproc)          # parallel compilation
xiom build --incremental            # only recompile changed files
```

### 2.3 Manifest Format (xiom.toml)

```toml
[package]
name = "xiom-compiler"
version = "0.57.0"
edition = "2026"

[build]
target = "x86_64-unknown-linux-gnu"
source = ["src/**/*.xi"]
stdlib = ["lib/**/*.xi"]

[dependencies]
xiom-stdlib = { path = "lib/xiom/std" }

[profile.release]
lto = true
overflow-checks = true
```

---

## 3. Symbol Resolution Across Units

### 3.1 Export Model (Per-File Symbol Table)

Each compiled file produces:
1. `.o` -- standard object file (ELF/COFF/Mach-O)
2. `.xiom.sym` -- JSON symbol table (for cross-file type checking + incremental rebuild tracking)

```
// lexer.xiom.sym
{
  "exports": {
    "functions": ["tokenize", "Lexer.new", "Lexer.next"],
    "types": ["Lexer", "Token", "TokenKind"],
    "globals": ["LEXER_BUFFER_SIZE"]
  },
  "imports": ["xiom.io.println", "xiom.string.str_len"],
  "types": {
    "Lexer": {"kind": "struct", "fields": {"source": "Str", "pos": "Int"}},
    "Token": {"kind": "enum", "variants": {"Identifier": ["Str"], "Number": ["Int"]}}
  },
  "dependencies": {
    "xiom/io.xi": "sha256:abc123...",
    "xiom/string.xi": "sha256:def456..."
  },
  "source_hash": "sha256:789abc..."
}
```

### 3.2 Import Resolution Algorithm

```
1. Parse import statements: use xiom.io;
2. Resolve import path to file on disk
3. Read .xiom.sym for imported file (if cached) OR read .xi source
4. Register imported symbols in current file's type context
5. Type-check current file against registered imports
6. At link time, resolve symbol references against .o files
```

### 3.3 Link-Time Type Checking

Cross-file type errors are caught when generating the `.xiom.sym` index:

```bash
xiom check-types                    # reads all .xiom.sym files, validates consistency
# Error: src/lexer.xi: Tokenizer source type 'Str' = lib/xtd.xi: Str (different definitions)
```

---

## 4. Stack-Safe Parser

### 4.1 Problem

Current recursive-descent parser uses the call stack. Deeply nested expressions (1000+ levels of `((((...))))`) overflow.

### 4.2 Solution: Trampolined Pratt Parser

Convert the recursive descent parser to use an explicit operator stack with a loop-based trampoline:

```rust
// Current (stack-unsafe):
fn parse_expr(&mut self, precedence: u8) -> Expr {
    let mut left = self.parse_prefix()?;
    while self.peek().precedence() > precedence {
        left = self.parse_infix(left)?;  // recursion!
    }
    Ok(left)
}

// Post-scale (stack-safe):
fn parse_expr(&mut self, precedence: u8) -> Expr {
    let mut left = self.parse_prefix()?;
    // Use explicit stack instead of recursion
    self.op_stack.push((precedence, Frame::Expr(left)));
    loop {
        match self.op_stack.pop() {
            Frame::Expr(expr) => {
                if self.peek().precedence() > precedence {
                    let left = self.parse_infix_trampoline(expr)?;
                    self.op_stack.push(Frame::Continue(left));
                } else {
                    return expr;
                }
            }
            Frame::Continue(left) => {
                self.op_stack.push(Frame::Expr(left));
                continue;
            }
        }
    }
}
```

### 4.3 Stack-Safe Block Parsing

Same pattern for nested blocks (if/while/match/defer/spawn):

- Replace recursive `compile_block()` calls with an explicit work queue
- Use a `Vec<Block>` queue instead of call-stack recursion
- Process blocks in FIFO order with depth counter

---

## 5. Streaming Codegen

### 5.1 Problem

Current codegen builds the entire LLVM IR as one `String` in memory. For 10K files, this is 100+ MB.

### 5.2 Solution: Per-Function IR Emission

Already partially done in v0.56 (parallel codegen). Extend to per-file:

```rust
// Current (monolithic):
self.output.push_str(&fn_ir);  // everything in one String

// Post-scale (streaming):
struct FileEmitter {
    output: BufWriter<File>,   // write directly to .ll file
    types: TypeContext,
    deferred: Vec<DeferredFn>, // functions emitted on-demand
}

impl FileEmitter {
    fn emit_function(&mut self, fd: &FnDecl) -> Result<()> {
        // Write function IR directly to file
        writeln!(self.output, "define {} @{}(...) {{", ret_ty, name)?;
        self.compile_block_to_writer(&fd.body, &mut self.output)?;
        writeln!(self.output, "}}")?;
        Ok(())
    }
}
```

### 5.3 Module-Level IR

Module-level content (type definitions, globals, declare statements) is emitted once at file start and doesn't grow with function count:

```llvm
; Module header (emitted once per file) -- constant size
%struct.Vec = type { i8*, i64, i64, i64 }
@xiom_recursion_counter = internal thread_local global i64 0
declare i8* @malloc(i64)

; Function bodies (streamed per-function) -- variable size
define i64 @tokenize(i8* %source) { ... }
define %struct.Token @Lexer.next(%struct.Lexer %self) { ... }
```

---

## 6. Incremental Build with Dirty Tracking

### 6.1 Change Detection

| What changes | What gets rebuilt | Detection |
|-------------|------------------|-----------|
| Function body in file A | File A only | SHA-256 of source |
| Function signature in file A | File A + all files that import A | SHA-256 vs .xiom.sym |
| Type definition in file A | File A + all files that use the type | .xiom.sym dependency graph |
| No changes | Nothing (cache hit) | SHA-256 match |

### 6.2 Dependency Graph

```
main.xi
  |-- lexer.xi          (imports: Str, Vec)
  |   `-- xiom/string.xi
  |   `-- xiom/collections/collections.xi
  |-- parser.xi         (imports: Lexer, Token)
  |   `-- lexer.xi       <- dependency
  `-- check.xi          (imports: Parser, AST)
      `-- parser.xi      <- dependency
```

When `lexer.xi` changes:
1. SHA-256 of `lexer.xi` changes -> dirty
2. Walk dependency graph: `parser.xi` depends on `lexer.xi` -> dirty
3. `check.xi` depends on `parser.xi` -> dirty
4. Rebuild: lexer.xi -> parser.xi -> check.xi (topological order)
5. Files with unchanged SHA-256 -> cache hit, skip

### 6.3 Cache Structure

```
~/.xiom/cache/
  |-- sha256_abc123/
  |   |-- lexer.ll          (LLVM IR text)
  |   |-- lexer.o           (compiled object)
  |   `-- lexer.xiom.sym    (symbol table)
  |-- sha256_def456/
  |   |-- parser.ll
  |   |-- parser.o
  |   `-- parser.xiom.sym
  `-- cache_index.json      (LRU tracking, 100 MB limit)
```

---

## 7. Parallel Frontend

### 7.1 Phase 1: Parallel Parse + Check (Per-File)

Already have: parallel parse (`--parallel`) + parallel codegen (`--parallel-codegen`).
Need: parallel checking.

```rust
use rayon::prelude::*;

fn compile_project(files: &[PathBuf], config: &CompileConfig) -> Result<()> {
    // Phase 1: Parse all files in parallel
    let parsed: Vec<_> = files.par_iter()
        .map(|path| (path, parse_file(path)))
        .collect();

    // Phase 2: Resolve imports (sequential -- dependency graph)
    let dep_graph = build_dep_graph(&parsed);
    let sorted = dep_graph.topological_sort();

    // Phase 3: Check files in parallel (respecting dependency order)
    sorted.par_iter()
        .for_each(|(path, program)| {
            check_file(program, &dependency_types(path, &dep_graph));
        });

    // Phase 4: Codegen in parallel (already working -- I2)
    sorted.par_iter()
        .for_each(|(path, program)| {
            codegen_file(program, path);
        });
}
```

### 7.2 Thread-Safe Type Registry

Already done: `SyncRegistry` (Arc<RwLock<HashMap>>) from v0.54. This is the foundation. Extend to:

```rust
pub struct FileTypeContext {
    // Imported types from other files (read-only, shared)
    pub imports: Arc<TypeRegistry>,
    // Types defined in this file (mutable, file-local)
    pub local_types: HashMap<String, TypeInfo>,
    // Exported types (published to imports after check)
    pub exports: SymbolTable,
}
```

---

## 8. Linker Integration

### 8.1 Strategy

No custom linker. Use the system linker (ld/lld/clang) by generating standard object files:

```bash
# Per-file compilation
xiom -c src/lexer.xi -o build/lexer.o
xiom -c src/parser.xi -o build/parser.o

# Link all objects
clang build/*.o -o build/xiom-compiler -lxiom_runtime -lpthread
```

### 8.2 Runtime Linking

The XIOM C runtime (`libxiom_runtime.a` / `.dll`) is linked once at the final step:

```bash
xiom build-runtime                  # Builds libxiom_runtime.a
xiom build                          # Compiles all .xi -> .o, links with runtime
```

---

## 9. Migration Path (Incremental, Non-Breaking)

### Phase 1: Enable File-Level Compilation (v0.58 -- 2 weeks)
- Add `-c` flag: compile one file to one object (`xiom -c file.xi -o file.o`)
- Emit `.xiom.sym` alongside `.o`
- Keep existing monolithic mode as default

### Phase 2: Dependency Resolution (v0.58 -- 2 weeks)
- Resolve `use` statements to file paths
- Load `.xiom.sym` from dependencies
- Type-check against imported symbol tables
- Link-time type validation (`xiom check-types`)

### Phase 3: Incremental Build (v0.59 -- 2 weeks)
- SHA-256 per-file source hashing
- Dependency graph construction
- Dirty propagation (change -> rebuild dependents)
- Cache `.ll` + `.o` + `.xiom.sym` per SHA-256

### Phase 4: Stack-Safe + Streaming (v0.59 -- 2 weeks)
- Trampolined parser
- Streaming codegen (per-function FileEmitter)
- Stack-safe checker (work-queue pattern)

### Phase 5: Parallel Frontend (v0.60 -- 2 weeks)
- Parallel parse + check (respecting dependency order)
- Thread-safe per-file TypeRegistry
- Parallel clang -c invocations

### Phase 6: Build System (v0.60 -- 1 week)
- `xiom build` command with xiom.toml
- `xiom build --incremental`
- `xiom build --jobs=N`
- `xiom clean`

---

## 10. Performance Targets

| Metric | Current (v0.56) | Target (v0.58+) | Improvement |
|--------|----------------|-----------------|-------------|
| 1 file, 100 LOC | ~200ms | ~200ms | -- |
| 100 files, 10K LOC | ~45s | ~8s (parallel) | 5.6x |
| 1,000 files, 100K LOC | OOM | ~30s | Infinite |
| 10,000 files, 1M LOC | Impossible | ~2-3 min (parallel) | inf |
| Incremental (1 file changed) | ~45s (full rebuild) | ~0.5s (1 file recompile + link) | 90x |
| Memory (100K LOC project) | ~200 MB | ~50 MB (per-worker) | 4x |

---

## 11. Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Cross-file generics need full type info at codegen time | Emit `.xiom.sym` with full generic signatures; monomorphise at link time if needed |
| Thread-local state leaks across parallel workers | `FileTypeContext` is file-local; shared only via `Arc<ReadOnly>` |
| C ABI incompatibility across objects | All objects emit through clang; ABI is guaranteed by LLVM |
| Circular imports (`A.xi` imports `B.xi`, `B.xi` imports `A.xi`) | Detect + error at import resolution. No circular dependencies. |
| Existing monolithic mode breaks | Keep monolithic mode as `xiom --monolithic` for selfhost bootstrap until fully migrated |

---

## 12. Architecture Review -- 3 Critical Gaps (2026-08-10, pre-selfhost review)

An external production-compiler review of this design surfaced **three architectural
gaps** that, if unaddressed, will make the scaling architecture collapse during the
selfhost phase or at the 10K-file milestone. Each is addressed below with the
bullet-proof refinement and a revised migration path.

### Gap 1: Generic Monomorphization Across Files (the "C++ Header" Problem)

**Problem.** With per-file codegen, `Vec[Int]` used in File A and File B is
monomorphized in BOTH objects. Without LTO, duplicate symbols explode the binary;
with LTO, compilation times skyrocket -- defeating parallel compilation.

**Bullet-Proof Fix: Sealed Generics + Pre-Mono Table.**

1. **Sealed generics:** Generics in the stdlib (`Vec[T]`, `Map[K,V]`, `Set[T]`, ...)
   are "sealed" when the stdlib is compiled. `libxiom_std.a` ships a
   **Pre-Monomorphized Table** of function bodies for all primitive types
   (`Int`, `Float64`, `Str`, `Bool`, `UInt8`, ...) as generic-erased symbols.
2. User files calling `Vec[Int]` do NOT generate code -- they call the
   pre-compiled, generic-erased functions from `libxiom_std.a`.
3. User generics over their own types (`Vec[MyStruct]`) are monomorphized
   locally and emitted as **Weak Symbols** in the `.o`; the linker keeps the
   first definition and discards the rest.

**Design change to S3:**
- `.xiom.sym` gains a `"generics"` section listing the sealed generic
  instantiations provided by the file/object (name -> instantiation set).
- Codegen, when emitting a generic call, consults the pre-mono table first;
  only user-type instantiations are emitted locally (weak).

### Gap 2: Stable Type Layouts in `.xiom.sym` (the "Recompile the World" Trap)

**Problem.** Changing `Vec`'s layout in `stdlib.xi` triggers a full dirty
propagation -> recompiles all 10,000 user files (2-3 minutes) even when only a
method body changed and the ABI is identical.

**Bullet-Proof Fix: Layout Hash + Forced Recheck.**

1. `.xiom.sym` stores a **Layout Hash** -- SHA-256 of the struct's FIELD TYPES
   and ALIGNMENT only, NOT the source code.
2. The dependency graph distinguishes **signature changes** (rebuild dependents)
   from **layout-hash changes**:
   - Layout hash unchanged -> ABI identical -> dependents keep their cached `.o`
     (only re-emit signatures, which are unchanged).
   - Layout hash changed -> only files that embed the type in THEIR OWN layout
     fully recompile (typically ~1% of the codebase).
3. Cache keeps the object under the layout-hash key, so old/new coexist during
   migration.

**Design change to S6.1/S6.3:** dirty-propagation table becomes:

| What changes | Rebuild | Detection |
|-------------|---------|-----------|
| Function body | File only | SHA-256 of source |
| Function signature | File + importers | `.xiom.sym` signature hash |
| Type layout (field set/order/types) | File + files embedding the type | `.xiom.sym` **layout hash** |
| Method body only (layout unchanged) | File only | layout hash unchanged -> ABI-fast path |

### Gap 3: The Compiler Daemon (why `xiom build` alone bottlenecks at 10K files)

**Problem.** Spawning the compiler binary per file (even 100 times) costs
hundreds of ms in process spawn + `stat()` scans of 10,000 files.

**Bullet-Proof Fix: Lazy Import Resolver / Daemon (`xiom daemon`).**

1. `xiom daemon` (or `xiom build --persistent`) holds the dependency graph and
   file SHA hashes in memory.
2. File-system notifications (inotify / ReadDirectoryChangesW) trigger a
   **Delta analysis**: which imports changed, which signatures changed.
3. Only dirty files spawn compiler workers (pre-warmed pool).
4. Target: 1-line change on a 10K-file project -> **~200 ms** (1 dirty file +
   relink), matching Bazel/cargo-check behavior.

**Design addition to S7/S9:** a new Phase 6b for the daemon; the watcher
infrastructure already exists (OrcJIT `HotReloadWatcher`).

### Revised Migration Path (v0.58-v0.60)

| Phase | Original Plan | Revised Additions (Critical) | Effort |
|-------|---------------|------------------------------|--------|
| Phase 1 | `xiom -c` + `.xiom.sym` | **Add Layout Hash to `.xiom.sym`** | +2 days |
| Phase 2 | Dependency Resolution | **Detect layout-hash vs signature changes** to minimize dirty propagation | +3 days |
| Phase 3 | Incremental Build | **Pre-mono Table for stdlib generics** (prevent duplicate codegen) | +3 days |
| Phase 6 | Build System | **`xiom daemon` persistent mode** for watch/build | +5 days |

**New total effort: ~15 weeks (still realistic).**

### Review Verdict (recorded)

- The blueprint (parallel units, symbol tables, SHA caching) is the exact pattern
  used by `go build` and rustc incremental -- it scales logarithmically.
- Layout Hash + Pre-mono Table are REQUIRED or the linker chokes on duplicate
  symbols at ~5K files and stdlib layout changes trigger full rebuilds.
- Selfhost (~200 files) does NOT need these; the true test is user projects.
- **CTFE note:** keep the CTFE engine (RefCell on IrEmitter) EXACTLY as-is for
  v0.57 selfhost. Only at Scaling Phase 5, migrate the CTFE cache to a
  thread-safe global (`RwLock<HashMap<u64, CtfeValue>>` in SyncRegistry) so
  128 parallel workers don't panic on shared RefCell or recompute constants.

---

## 13. Summary

| Component | Approach | Effort | Dependencies |
|-----------|----------|--------|-------------|
| Separate compilation | `xiom -c file.xi -o file.o` | 2 weeks | -- |
| Symbol tables | `.xiom.sym` JSON per file | 2 weeks | Separate compilation |
| Dependency graph | Topological sort + dirty propagation | 2 weeks | Symbol tables |
| Stack-safe parser | Trampolined Pratt parser | 1 week | -- |
| Streaming codegen | BufWriter per-file emitter | 2 weeks | Per-function codegen (already done) |
| Parallel frontend | Rayon per-file parse+check | 2 weeks | Thread-safe registry (already done) |
| Incremental build | SHA-256 per-file + graph | 2 weeks | Dependency graph |
| Linker | System ld/lld via clang | 1 week | Separate compilation |
| Build system | `xiom.toml` + `xiom build` | 1 week | All of the above |
| Compiler daemon | `xiom daemon` / `--persistent` (Gap 3) | 1 week | Incremental build |
| **Total estimated effort** | | **~15 weeks** | |

**Start condition:** Selfhost complete (v0.57). Don't start before then -- scaling architecture needs to be designed against a language that has been proven correct through self-compilation.
