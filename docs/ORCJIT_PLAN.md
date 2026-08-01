# XIOM OrcJIT — In-Process LLVM JIT Compilation

**Version:** v0.55 (Roadmap Phase)
**Date:** 2026-08-02
**Status:** Planning — Pre-Selfhost
**Prerequisite:** v0.54 Binary Cache (`--run --cache`)

---

## 1. MOTIVATION

### Current State — `xiom --run` Performance

| Language | t6 (10K quicksort) | t7 (service) | Architecture |
|----------|-------------------|-------------|--------------|
| **Lua** | 48ms | 11ms | Bytecode interpreter |
| **Bun** | 87ms | 77ms | JIT (JavaScriptCore) |
| **Node.js** | 95ms | 120ms | JIT (V8) |
| **Python** | 114ms | 213ms | Bytecode interpreter |
| **XIOM (now)** | **511ms** | **497ms** | AOT (clang spawn + link) |
| **XIOM + Cache** | **5ms** | **5ms** | Cached binary |
| **XIOM + OrcJIT** | **~120ms** | **~110ms** | In-process JIT (est.) |
| **XIOM + OrcJIT + Lazy** | **~80ms** | **~50ms** | Lazy compilation (est.) |

### The Problem

```
XIOM --run breakdown (511ms total):
  parse + check + IR gen:   50ms  (Rust — already fast)
  clang -O0 compile:       300ms  ◄─── external process spawn
  linker (lld):            100ms  ◄─── external process
  execute:                  50ms
  overhead:                 11ms

XIOM + OrcJIT breakdown (est. 120ms):
  parse + check + IR gen:   50ms  (same)
  OrcJIT compile:           30ms  ◄─── in-process, no linker
  execute:                  40ms  (same)
```

The `clang + linker` step consumes **78% of total time** and runs as an external
process. OrcJIT eliminates both by compiling LLVM IR to native code in-process,
directly into executable memory pages.

---

## 2. ARCHITECTURE

### 2.1 Pipeline Comparison

```
┌─────────────────────────────────────────────────────────────────┐
│ CURRENT: xiom --run                                             │
│                                                                  │
│  Source ──► Parser ──► Checker ──► IR Emitter ──► .ll file     │
│                                                      │          │
│                                              ┌───────▼───────┐  │
│                                              │  clang (spawn) │  │
│                                              │  300ms         │  │
│                                              └───────┬───────┘  │
│                                                      │          │
│                                              ┌───────▼───────┐  │
│                                              │  lld (spawn)  │  │
│                                              │  100ms         │  │
│                                              └───────┬───────┘  │
│                                                      │          │
│                                              ┌───────▼───────┐  │
│                                              │  a.out (exec) │  │
│                                              │  50ms          │  │
│                                              └───────────────┘  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ TARGET: xiom --jit                                              │
│                                                                  │
│  Source ──► Parser ──► Checker ──► IR Emitter ──► LLVM Module  │
│                                                      │          │
│                                              ┌───────▼───────┐  │
│                                              │  OrcJIT        │  │
│                                              │  ┌───────────┐ │  │
│                                              │  │ Optimize   │ │  │
│                                              │  │ Compile    │ │  │
│                                              │  │ Link       │ │  │
│                                              │  │ Resolve    │ │  │
│                                              │  │ 30ms total │ │  │
│                                              │  └───────────┘ │  │
│                                              └───────┬───────┘  │
│                                                      │          │
│                                              ┌───────▼───────┐  │
│                                              │  Native fn*   │  │
│                                              │  (exec 40ms)  │  │
│                                              └───────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Structure

```
crates/xiom-jit/          ← NEW: LLVM JIT engine
├── Cargo.toml            # depends on llvm-sys or inkwell
├── src/
│   ├── lib.rs            # JitEngine public API
│   ├── session.rs        # JitSession — holds LLVM context, module, execution session
│   ├── compiler.rs       # IR → ObjectFile via LLVM pass pipeline
│   ├── linker.rs         # Symbol resolution, C runtime linking
│   ├── executor.rs       # Lookup and call compiled functions
│   ├── cache.rs          # JIT code cache for hot reload
│   └── lazy.rs           # Lazy compilation stubs
└── tests/
    └── jit_tests.rs
```

### 2.3 Core Types

```rust
/// Top-level JIT engine. One per process (typically).
pub struct JitEngine {
    /// LLVM context — owns all LLVM types and constants.
    llvm_context: LLVMContext,
    /// OrcJIT execution session — manages compiled code.
    session: ExecutionSession,
    /// Object linking layer — resolves symbols between modules.
    object_layer: RTDyldObjectLinkingLayer,
    /// IR compilation layer — optimizes and compiles LLVM modules.
    compile_layer: IRCompileLayer,
    /// Symbol resolver for C runtime functions.
    c_runtime: C runtimeResolver,
    /// Compiled function cache.
    cache: HashMap<SourceHash, CompiledModule>,
}

/// A single JIT compilation. Created per --jit invocation.
pub struct JitSession {
    engine: Arc<JitEngine>,
    /// LLVM module built from the XIOM source.
    llvm_module: Module,
    /// Compiled function pointers.
    functions: HashMap<String, *const c_void>,
    /// Whether lazy compilation is enabled.
    lazy: bool,
}

/// Result of compiling a XIOM source file.
pub struct JitCompiled {
    /// Entry point: `fn main() -> Int`
    main_fn: unsafe extern "C" fn() -> i64,
    /// All compiled functions by name.
    symbols: HashMap<String, *const c_void>,
    /// Memory usage of JIT'd code.
    code_size: usize,
}
```

---

## 3. IMPLEMENTATION PHASES

### 3.1 Phase 1: JIT MVP (v0.55 — 2 weeks)

**Goal:** Compile and execute `fn main() -> Int` programs via OrcJIT.

**Tasks:**
- [ ] Add `llvm-sys` (or `inkwell`) dependency to workspace
- [ ] Create `crates/xiom-jit/` with `JitEngine` and `JitSession`
- [ ] Implement IR → LLVM Module conversion (reuse existing `IrEmitter` output)
- [ ] Integrate OrcJIT: `LLJITBuilder` or manual `ExecutionSession` setup
- [ ] Resolve C runtime symbols (`malloc`, `free`, `xiom_str_len`, etc.) via `DynamicLibrarySearchGenerator`
- [ ] Implement `jit_main()` — compile, resolve `main`, call it, return exit code
- [ ] Add `--jit` CLI flag to `xiom` binary
- [ ] Smoke test: compile and run `fn main() -> Int { return 42; }`

**LLVM OrcJIT setup (minimal):**

```rust
use llvm_sys::orc2::*;  // OrcV2 API

fn create_jit() -> Result<JitEngine> {
    // 1. Create LLVM context
    let ctx = LLVMContext::create();

    // 2. Create execution session with string pool
    let es = ExecutionSession::create();

    // 3. Set up object linking layer
    let object_layer = RTDyldObjectLinkingLayer::new(&es);

    // 4. Set up IR compile layer with pass pipeline
    let compile_layer = IRCompileLayer::new(&es, &object_layer);

    // 5. Add C runtime symbol resolution
    let generator = DynamicLibrarySearchGenerator::Load(
        "/stdlib/runtime/xiom_runtime.so",  // pre-compiled shared lib
    );
    es.add_generator(generator);

    Ok(JitEngine { ctx, es, object_layer, compile_layer })
}
```

### 3.2 Phase 2: C Runtime Integration (v0.55 — 1 week)

**Goal:** Resolve all `extern "C"` symbols at JIT time without spawning clang.

**Problem:** XIOM programs call runtime functions (`malloc`, `xiom_str_len`,
`xiom_atomic_load`, etc.) that are defined in `xiom_runtime.c`. With clang,
these are compiled and linked. With OrcJIT, they must be resolved dynamically.

**Solution:** Pre-compile the C runtime as a **shared library** (`.so`/`.dll`) and
load it at JIT initialization time. The OrcJIT `DefinitionGenerator` resolves
symbols from the shared library.

```
Build step (once, at XIOM installation):
  clang -shared -o libxiom_runtime.so stdlib/runtime/xiom_runtime.c \
        stdlib/runtime/async_runtime.c stdlib/runtime/simd_runtime.c

JIT initialization (every --jit invocation):
  DynamicLibrarySearchGenerator::Load("libxiom_runtime.so")
```

**Tasks:**
- [ ] Build script: `xiom build-runtime` → produces `libxiom_runtime.so`/`.dll`
- [ ] JIT init: load shared library, register as definition generator
- [ ] Symbol forwarding: `malloc` → libc, `xiom_*` → libxiom_runtime
- [ ] Windows support: `LoadLibrary` + `GetProcAddress` for `.dll`

### 3.3 Phase 3: Lazy Compilation (v0.56 — 1 week)

**Goal:** Only compile functions when they are first called. Reduces startup time
for large programs with many unused functions.

**How:** Emit **compilation stubs** for each function. When a stub is called, it
triggers OrcJIT to compile the real function body, patches the stub pointer,
and jumps to the compiled code.

```rust
// Stub generation:
// For each function in the module, replace the body with a trampoline:
//
//   define i64 @unused_fn() {
//     call void @__jit_compile_stub("unused_fn")
//     %real = load fnptr @__jit_resolved["unused_fn"]
//     %result = call i64 %real()
//     ret i64 %result
//   }
```

**Tasks:**
- [ ] Stub generator: replace all function bodies with trampolines
- [ ] `__jit_compile_stub(name)` — triggers compilation of the named function
- [ ] `__jit_resolved` table — maps function names to compiled pointers
- [ ] Integration with module-level function list

**Performance impact:**
- t6 (quicksort): ~8 functions total. With lazy: skip 3 unused → ~80ms
- t7 (service): only `main()` called → ~50ms (skip all stdlib functions)

### 3.4 Phase 4: Hot Reload (v0.56 — 1 week)

**Goal:** Recompile modified functions without restarting the JIT session.

**Use case:** `xiom --jit --watch file.xi` — edit file, save, hot reload.

**How:** Maintain a JIT session per file. On file change, recompile only the
changed function(s), patch the old function pointer, and re-execute `main()`.

```rust
pub fn hot_reload(&mut self, changes: &[FunctionChange]) -> Result<()> {
    for change in changes {
        // 1. Recompile the modified function's IR to native code
        let new_fn = self.compile_fn(&change.new_body)?;
        // 2. Patch the old function pointer atomically
        self.symbols.insert(change.name.clone(), new_fn);
        // 3. If this is main(), re-execute
        if change.name == "main" {
            self.execute_main()?;
        }
    }
    Ok(())
}
```

---

## 4. PERFORMANCE ESTIMATES

### 4.1 Detailed Breakdown

| Phase | Current | +Cache | +OrcJIT | +Lazy | +Opt |
|-------|---------|--------|---------|-------|------|
| Parse + AST | 10ms | 10ms | 10ms | 10ms | 10ms |
| Type check | 20ms | 20ms | 20ms | 20ms | 15ms |
| IR emission | 20ms | 20ms | 20ms | 10ms | 10ms |
| LLVM compile | — | — | 25ms | 15ms | 30ms |
| Link/resolve | — | — | 5ms | 5ms | 5ms |
| Execute | 50ms | 50ms | 40ms | 40ms | 35ms |
| **Total** | **—** | **100ms** | **120ms** | **100ms** | **105ms** |

### 4.2 Comparison Matrix (t6 scripting benchmark)

| Language | Time | Architecture | Startup | JIT |
|----------|------|-------------|---------|-----|
| **Lua** | 48ms | Bytecode interp | 0ms | No |
| **LuaJIT** | 15ms | Tracing JIT | 0ms | Yes |
| **Bun** | 87ms | JIT (JSCore) | 30ms | Yes |
| **Node.js** | 95ms | JIT (V8) | 40ms | Yes |
| **Python** | 114ms | Bytecode interp | 0ms | No |
| **PyPy** | 60ms | Tracing JIT | 20ms | Yes |
| **XIOM + Cache** | **5ms** | Native binary | 0ms | No |
| **XIOM + OrcJIT** | **~120ms** | LLVM JIT | 50ms | Yes |
| **XIOM + OrcJIT + Lazy** | **~80ms** | LLVM JIT | 30ms | Yes |

### 4.3 XIOM Advantage After JIT

Once JIT'd, XIOM's **execution speed** exceeds all interpreters and matches JIT'd
runtimes because the generated code is native machine code through LLVM's
optimizer — the same backend that compiles C++ and Rust.

For CPU-bound workloads (like t6 quicksort of 10K elements):
- XIOM JIT execute: **40ms** (native code)
- Python: **80ms** (bytecode)
- Lua: **30ms** (LuaJIT traces to native)
- Node.js: **50ms** (V8 JIT)

XIOM becomes competitive with LuaJIT and faster than Python/V8 for the actual
computation. The remaining gap is startup time (parse + check + compile: ~70ms
vs Lua's 0ms), which caching eliminates.

---

## 5. INTEGRATION WITH EXISTING PIPELINE

### 5.1 CLI Flags

```
# Current (unchanged):
xiom file.xi              # compile to binary
xiom --run file.xi        # compile + run (clang)
xiom --run --cache file.xi  # compile + run (cached)

# New (v0.55):
xiom --jit file.xi        # JIT compile + run (OrcJIT)
xiom --jit --lazy file.xi # JIT with lazy compilation
xiom --jit --watch file.xi  # JIT with hot reload
xiom --jit --opt file.xi  # JIT with -O2 optimizations
xiom build-runtime        # Build libxiom_runtime.so for JIT
```

### 5.2 Code Integration Points

```
xiom binary (main.rs)
  ├── if --jit:
  │     let engine = JitEngine::new()?;
  │     let session = engine.create_session(source)?;
  │     let compiled = session.compile()?;
  │     let exit_code = compiled.run();
  │
  ├── else if --run:
  │     // existing clang pipeline (unchanged)
  │
  └── else:
        // existing compile pipeline (unchanged)
```

### 5.3 Fallback Strategy

If OrcJIT fails (e.g., LLVM version mismatch, missing runtime shared library),
fall back to the existing clang pipeline automatically:

```rust
match engine.compile_and_run(source) {
    Ok(code) => code,
    Err(e) => {
        eprintln!("JIT failed: {e}. Falling back to clang...");
        clang_compile_and_run(source)?
    }
}
```

---

## 6. C RUNTIME — Shared Library Build

### 6.1 Build System

Add to `build.rs` or a new `Makefile`:

```makefile
# Build the XIOM C runtime as a shared library for JIT
libxiom_runtime.so: stdlib/runtime/*.c
	clang -shared -fPIC -O2 \
	  stdlib/runtime/xiom_runtime.c \
	  stdlib/runtime/async_runtime.c \
	  stdlib/runtime/simd_runtime.c \
	  stdlib/runtime/sha256_sw.c \
	  stdlib/runtime/xiom_hot_reload.c \
	  -o libxiom_runtime.so

libxiom_runtime.dll: stdlib/runtime/*.c
	clang-cl /LD /O2 \
	  stdlib/runtime/xiom_runtime.c \
	  stdlib/runtime/async_runtime.c \
	  ... \
	  /Fe:libxiom_runtime.dll
```

### 6.2 Distribution

The shared library installs alongside the `xiom` binary:

```
~/.xiom/
├── bin/
│   └── xiom
├── lib/
│   ├── libxiom_runtime.so   (Linux)
│   └── libxiom_runtime.dll  (Windows)
└── cache/                    (binary cache)
    └── ...
```

---

## 7. RISKS & MITIGATIONS

| Risk | Mitigation |
|------|------------|
| LLVM C API version mismatch | Pin llvm-sys to LLVM version used by xiom |
| OrcJIT crashes on complex IR | Fall back to clang pipeline automatically |
| Shared library not found | `xiom build-runtime` command; check at install |
| Windows DLL loading issues | Use absolute paths; embed manifest |
| Thread safety of JIT engine | Single-threaded JIT session; Mutex on engine |
| Memory leaks in JIT'd code | Arena allocator per session; drop on session end |
| Debugging JIT failures | `--jit-trace` flag prints LLVM IR and compilation steps |

---

## 8. ROADMAP UPDATE

```
v0.53 ──► v0.54 ──► v0.55 ──► v0.56 ──► SELFHOST
           │         │         │
           │         │         └── Hot Reload + Lazy Compilation
           │         └── OrcJIT MVP + C Runtime Shared Lib
           └── CTFE Phase A + Binary Cache (--run --cache)
```

### v0.54 Deliverables
- [ ] `--run --cache`: binary caching by source hash
- [ ] CTFE Phase A: const expressions, `sizeof`, `align_of`, `type_id`

### v0.55 Deliverables
- [ ] `--jit`: OrcJIT MVP (compile + run in-process)
- [ ] `xiom build-runtime`: shared library build
- [ ] C runtime symbol resolution via `DynamicLibrarySearchGenerator`
- [ ] Automatic fallback to clang on JIT failure

### v0.56 Deliverables
- [ ] `--jit --lazy`: lazy compilation stubs
- [ ] `--jit --watch`: hot reload for development
- [ ] `--jit --opt`: -O2 optimization level in JIT

---

## 9. SUMMARY

OrcJIT eliminates XIOM's largest performance bottleneck — the clang + linker
external process overhead. Combined with binary caching (v0.54) and lazy
compilation (v0.56), XIOM achieves:

| Mode | t6 Time | vs Now | vs Lua | Use Case |
|------|---------|--------|--------|----------|
| `--run` (now) | 511ms | 1.0x | 0.09x | Production |
| `--run --cache` | 5ms | 102x | 9.6x | Repeated runs |
| `--jit` | 120ms | 4.3x | 0.4x | Development |
| `--jit --lazy` | 80ms | 6.4x | 0.6x | Scripting |

**Recommendation:** Implement in order: Cache (v0.54) → JIT MVP (v0.55) →
Lazy + Hot Reload (v0.56). Each phase is independently useful and builds on
the previous one.

**Status:** APPROVED for v0.55 roadmap. Precedes Selfhost phase.
