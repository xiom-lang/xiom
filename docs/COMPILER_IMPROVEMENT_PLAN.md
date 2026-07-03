# AXIOM Compiler — Improvement Plan

**Date:** 2026-07-03
**Status:** Living plan — updated as the compiler evolves
**Companion to:** `docs/COMPILER_ARCHITECTURE.md` (source of truth for current state)

> This document describes what the compiler SHOULD become. `COMPILER_ARCHITECTURE.md` describes what it IS.
> Every item here is grounded in the three pillars: SAFE, VERIFIED, PRECISE.
> The compiler must NEVER freeze or crash. That is not a goal. That is the baseline.

---

## Phase 0 — Critical Correctness (P0: Now)

These are NOT features. They are bugs that make the compiler unsafe to use at scale. Fix them before adding anything.

### 0.1 Vec Push Reallocation (V1)

**Problem:** `Vec.push` allocates 128 bytes once. Element 17 writes past the buffer — heap corruption, undefined behavior.

**Solution:** Add capacity check before every `push`. When full, `@realloc` with doubling strategy (128 → 256 → 512...). Emit the realloc logic directly in codegen's Vec builtin path (already has `@malloc`, just wire `@realloc`).

**Effort:** 2-3 hours. Small change in `compile_expr` Vec.push handler.

### 0.2 C Runtime Fixed Limits (V5)

**Problem:** The C runtime (`stdlib/runtime/axiom_runtime.c`) uses fixed-size arrays:
- `MAX_STRUCT_FIELDS = 16`
- `MAX_LOCAL_VARS = 64`
- `MAX_MATCH_ARMS = 16`

Exceeding any silently produces wrong IR.

**Solution:** Replace with dynamically allocated arrays using `malloc`/`realloc`. Start with a reasonable initial capacity (e.g., 64 fields) and grow on overflow. Alternatively, increase limits to 256/1024/256 — enough for any realistic program. The codegen already generates struct field metadata; the C runtime just needs to not truncate.

**Effort:** 1-2 days. Requires C runtime changes + codegen verification.

### 0.3 Unknown Type → Error, Not i64 (V7)

**Problem:** `axiom_to_llvm_type` has `_ => "i64"` as a catch-all. Unknown types silently compile as `i64`, producing wrong IR instead of a clear error.

**Solution:** Return `Result` instead of `String` from `llvm_type_for`. On unknown type, emit a structured error with the type name and source location. Never silently default to `i64`.

**Effort:** 3-4 hours. Touches ~40 call sites in codegen.

### 0.4 Generic Monomorphisation Loop Guard (V6)

**Problem:** If a monomorphised function body triggers another monomorphisation of the same function (e.g., recursive generic), the worklist grows unboundedly → infinite compile.

**Solution:** Hard limit of 5000 total monomorphisation iterations. On overflow, emit a clear error: "monomorphisation limit exceeded (loop in generic `Foo[Bar]`?)". Track a hash of `(fn_name, concrete_types)` per iteration to detect true cycles.

**Effort:** 1-2 hours. Add counter + cycle detection to `compile_generic_monomorphisations`.

### 0.5 Division-by-Zero in Compiler Internals

**Problem:** The compiler itself does unchecked integer division in several places (e.g., `sdiv` during offset calculations). A compiler crash from div-by-zero in the COMPILER is unacceptable.

**Solution:** Audit all `sdiv`/`srem` in Rust code (`axiom-codegen`, `axiom-check`, `axiomc`). Wrap in `checked_div` with proper error handling. The target AXIOM programs already have div-zero guards (V4 fixed) — the compiler itself needs them too.

**Effort:** 2-3 hours. Grep + audit ~15-20 division sites.

---

## Phase 1 — Performance Foundations (P1: Next)

These make the compiler fast enough for absurd benchmarks. Without them, 100+ generics or 1000+ modules will be slow but won't crash.

### 1.1 Indexed Module Catalog

**Problem:** `ModuleCatalog::load_module` walks all source directories and reads every `.ax` header on cache miss. For 1000 files, cold start = 1000 file reads × M lookups = O(N×M).

**Solution:**
1. On first `find_owned()` call OR at `axiomc` startup, build a `HashMap<String, String>` (module_path → file_path) by scanning source_dirs once.
2. All subsequent lookups are O(1) hash map access — no path guessing, no scan fallback.
3. The scan still exists as a cold-start bootstrap, but only runs ONCE per compilation session.

**Effort:** 1 day. New method `ModuleCatalog::build_index()` + refactor `load_module` to prefer index.

**Impact:** 1000-module projects resolve in milliseconds instead of seconds.

### 1.2 Parallel Monomorphisation

**Problem:** Generic instantiations are processed sequentially. Each clones the function AST and re-emits LLVM IR. With 100+ instantiations, this is the compile-time bottleneck.

**Solution:**
1. Each monomorphisation is independent (no shared state mutation except IR emission).
2. Use `rayon` or `std::thread` to process instantiations in parallel.
3. Collect results into a `Vec<(String, String)>` (name → IR text), then emit sequentially in deterministic order.
4. Dedup: before cloning, check a `HashSet<(String, Vec<String>)>` of `(fn_name, concrete_types)`. If already monomorphised, skip.

**Effort:** 2-3 days. Requires making `IrEmitter` cloneable or per-thread, careful about `Rc`/`RefCell`.

**Impact:** 100 instantiations in ~2 seconds instead of ~20.

### 1.3 Incremental Compilation Foundations

**Problem:** Every `axiomc` invocation re-parses every file. For a 1000-file project changing one line, this is wasteful.

**Solution (Phase 1 — foundations only):**
1. Add `--incremental` flag. On first build, hash every source file and write `target/incremental.json`.
2. On rebuild, only re-parse files whose hash changed.
3. Re-check only functions whose dependencies changed (track a dependency graph: function → types/functions it references).
4. Re-emit IR only for changed functions. Link unchanged IR from the previous build.

**Full incremental compilation (Phase 2):** Add file watcher + `--watch` flag. Full hot-reload needs Phase 3.

**Effort:** 3-5 days for foundations. Full incremental = weeks.

### 1.4 IR Optimization Pass

**Problem:** LLVM IR is emitted as text with no optimization. `clang` applies its own passes, but the IR is substantially larger than needed.

**Solution:**
1. After emitting IR to a temp file, invoke `opt -O1 -S` on it if `opt` is available on the system.
2. This is a 1-line change: `cmd.args(["-O1", "-S", &ir_path])`.
3. Fall back to unoptimized if `opt` not found.

**Effort:** 30 minutes. One `Command::new("opt")` call.

**Impact:** 30-50% smaller IR, faster clang compilation.

---

## Phase 2 — Advanced Compilation (P2: Later)

### 2.1 Hot Reload / DLL Compilation

**Goal:** Recompile a single function or module and patch it into a running process — essential for game engines and robotics.

**Architecture:**

```
Source Change → File Watcher → Recompile Module → Generate .dll/.so → LoadLibrary → Patch Function Table
```

**Implementation steps:**
1. **`--shared` flag:** Emit position-independent code (PIC). On x86-64 this is mostly free; on other targets use `-fPIC`.
2. **Symbol table export:** Generate a `.def` file or `__declspec(dllexport)` annotations for every `pub fn`.
3. **Function table indirection:** Instead of direct `call @foo`, use a function pointer table `@axiom_fn_table`. Hot-reload swaps the pointer.
4. **State migration:** When a module is reloaded, its global state (`static var`) must be preserved. Use a separate data segment that survives reload.
5. **`--watch` + `--hot-reload`:** Watch source files. On change, recompile only the changed module, emit `.dll`, inject via `LoadLibrary`.

**Limitations:**
- Struct layout changes require full restart (cannot change memory layout of live data).
- Generic function recompilation depends on call sites — track instantiation sites.
- Works best with message-passing architectures (state in channels, not globals).

**Effort:** 2-3 weeks for MVP. This is the most complex feature in this plan.

### 2.2 Multithreaded Compilation

**Problem:** The compiler is single-threaded. For large projects, CPU cores sit idle.

**Solution:**
1. **Per-file parallel parsing:** Parse each source file on a separate thread. `merge_programs` is already synchronous — just parallelize the parse step.
2. **Parallel codegen:** After type checking (which must be single-threaded for correctness), emit IR for independent functions in parallel. Function bodies don't depend on each other.
3. **Work-stealing thread pool:** Use `rayon` for both stages.

**Safety:** The checker's borrow analysis depends on global state and must remain sequential. But parsing and codegen are embarrassingly parallel.

**Effort:** 1-2 weeks. Requires making `IrEmitter` `Send` + `Sync`.

### 2.3 Memory Budget Tracking

**Problem:** The compiler has no memory budget. A malicious or absurd input could exhaust RAM and crash the OS (not just the compiler).

**Solution:**
1. Wrap all unbounded collections (`Vec`, `HashMap`, `String`) in budget-tracking wrappers.
2. At startup, query system RAM and set a budget (e.g., 80% of available).
3. On allocation that would exceed budget, emit a clear error: "compiler memory budget exceeded (2.1GB used of 2.5GB limit). Simplify module X or split file Y."
4. The compiler NEVER crashes from OOM — it gracefully degrades.

**Effort:** 2-3 days. Primarily in `IrEmitter::output`, `Checker::types`, and parser's token buffer.

### 2.4 Timeout Guards

**Problem:** A pathologically nested generic or infinite-loop contract could cause unbounded compile time.

**Solution:**
1. Every compilation stage gets a wall-clock timeout (configurable, default 60 seconds).
2. On timeout, the compiler emits: "Stage `codegen` timed out after 60s. Last processing: function `foo` in module `bar`."
3. The compiler exits cleanly — no hang, no zombie process.

**Effort:** 1 day. Wrap each stage in `std::thread::spawn` + `recv_timeout`.

---

## Phase 3 — Toolchain & Ecosystem (P3: Future)

### 3.1 Debugger (DAP-based)

**Architecture:** The debugger is a separate process that speaks the Debug Adapter Protocol (DAP) to VS Code / JetBrains / etc. It controls the target process via platform debug APIs.

**Contract-aware debugging:**
- Before each contract guard, the codegen emits a `@llvm.dbg.value` intrinsic with the contract expression text and expected values.
- On `@llvm.trap()`, the debugger captures: which contract, expected vs actual, source location, call stack.
- "Explain the violation" mode: renders the contract in natural language with highlighted failing sub-expression.

**Ownership visualization:**
- Compiler emits DWARF debug info with custom `DW_TAG_axiom_ownership` entries.
- Debugger renders: variable lifetime bars (color-coded: green=alive, yellow=borrowed, red=moved), borrow graph arrows, moved-from markers.

**Effort:** 2-3 months. Requires DWARF emission in codegen + DAP server implementation.

### 3.2 Visual Benchmark Tool

**Goal:** Compare AXIOM against Rust, Zig, C++, Go — with beautiful visualizations.

**Architecture:**
1. `axiom bench` compiles each benchmark, runs it N times, collects metrics.
2. Results stored as JSON with schema: `{bench_name, language, metrics: {time_ns, mem_bytes, binary_size, contract_count}}`.
3. A static HTML dashboard (built with AXIOM → WASM) renders comparisons.
4. CI integration: every commit runs benchmarks and updates the dashboard.

**Effort:** 1-2 weeks for MVP (benchmark runner + JSON output + basic HTML).

### 3.3 Enhanced LSP

**Already exists** (syntax highlighting, diagnostics, go-to-def, hover).

**Additions:**
- **Contract lens:** Inline display of `requires`/`ensures` above function signatures.
- **Ownership overlay:** Color-coded variable underlines (green=owned, blue=borrowed, red=moved). Hover shows "Why was this moved?" with source location.
- **AI co-pilot commands:** `@explain` (why does this compile error?), `@add-contract` (suggest missing pre/post conditions), `@fix-borrow` (suggest clone/refactor).
- **Refactoring:** Extract function → automatically generates contracts from the extracted body's pre/post state.

**Effort:** 3-6 months. Incremental — add one feature at a time.

### 3.4 CLI & Project Management

See the XIOM CLI specification. Key commands:

```
axiom build          # compile project
axiom run            # build + run
axiom check          # type check only (fast, no codegen)
axiom test           # run test suite
axiom fmt            # canonical formatter
axiom bench          # run benchmarks
axiom debug          # build + launch debugger
axiom new <name>     # scaffold project
axiom install <pkg>  # install dependency
```

**`--hot-reload`**: For game engines, the CLI watches source files and Live-reloads changed modules into the running process. See Phase 2.1.

**`--ai`**: Verbose error messages designed for AI consumption — includes fix suggestions, contract implications, and structured JSON output.

---

## Phase 4 — Self-Hosting (P4: After Stability)

Self-hosting means the AXIOM compiler is written in AXIOM, compiled by the previous version of itself. This is the ultimate correctness test.

**Prerequisites:**
- All Phase 0-1 bugs fixed
- Full language surface stable (no breaking syntax changes for 6+ months)
- Standard library mature enough to write a compiler (string handling, file I/O, collections, FFI)
- Benchmark suite passing at 100%
- The Rust compiler is kept as the PERMANENT bootstrap fallback — never deleted

**Bootstrapping sequence:**
1. Write `axiom-lexer.ax`, `axiom-parser.ax`, `axiom-check.ax`, `axiom-codegen.ax` in AXIOM.
2. Compile Phase 4 AXIOM compiler with Phase 3 Rust compiler.
3. Compile Phase 4 AXIOM compiler with Phase 4 AXIOM compiler (self-compile).
4. Diff the output binaries — byte-for-byte identical → bootstrap complete.

**Do NOT start this until Phase 3 is rock-solid.** Every self-hosting attempt on an unstable compiler wastes weeks debugging the compiler AND the compiler-being-compiled simultaneously.

---

## Priority Matrix

| What | When | Why |
|------|------|-----|
| Vec realloc (V1) | NOW | Heap corruption makes every benchmark using Vec unreliable |
| C runtime limits (V5) | NOW | 16-field limit blocks many benchmarks |
| Unknown type → error (V7) | NOW | Silent wrong IR is worse than a crash |
| Mono loop guard (V6) | NOW | Infinite compilation violates "never freeze" |
| Div-zero in compiler | NOW | Compiler crash from internal div-by-zero is unacceptable |
| Indexed module catalog | NEXT | 1000+ file projects need O(1) resolution |
| Parallel monomorphisation | NEXT | 100+ generics bottleneck |
| IR optimization (`opt -O1`) | NEXT | 30-minute change, immediate impact |
| Incremental compilation | NEXT | Save minutes per build on large projects |
| Hot reload / DLL | LATER | Game engines and robotics — high value, high effort |
| Multithreaded compilation | LATER | Cores sit idle today |
| Memory budget tracking | LATER | Graceful OOM instead of OS crash |
| Timeout guards | LATER | No infinite hangs |
| Debugger (DAP) | FUTURE | Contract-aware, ownership visualization |
| Visual benchmark tool | FUTURE | Marketing + trust |
| Enhanced LSP | FUTURE | Developer productivity |
| CLI toolchain | FUTURE | `axiom build/run/test/fmt/bench` |
| Self-hosting | LAST | After everything above is stable |

---

## Honest Assessment: What Changes to Handle Absurd Benchmarks

### 10K Contracts

**Will it compile?** Yes, with Phase 0 fixes applied. Runtime guards scale linearly.

**Will it be fast?** The IR will be large (50K-100K extra instructions). `opt -O1` (Phase 1.4) helps but doesn't eliminate the overhead. Contracts are fundamentally a binary-size vs safety tradeoff.

**The real solution:** Z3 static verification (Phase 3 of the AXIOM roadmap). Prove contracts at compile time → eliminate runtime guards for statically-proven contracts → 0 overhead. This requires: translating the full type system to SMT theories, modeling heap state, loop invariants. This is a 3-6 month effort for a dedicated team — not a quick fix.

**Until Z3:** Accept that contracts have runtime cost, and make `--no-contracts` the production flag. Development uses contracts for catching bugs; release builds strip them.

### 100+ Generic Instantiations

**Will it compile?** Yes, but slowly (seconds to minutes).

**Will it be fast?** Parallel monomorphisation (Phase 1.2) cuts compile time by ~4x on a 4-core machine. Dedup eliminates redundant work.

**The real solution:** The compiler only monomorphises what's USED. If a generic function is instantiated 100 times but only 5 are called, only those 5 should be emitted. Current codegen eagerly tracks all instantiations — change to lazy/deferred emission.

### 1000+ Module Files

**Will it compile?** Yes, but the first `axiomc` invocation will spend seconds scanning for modules.

**Will it be fast?** Indexed catalog (Phase 1.1) makes cold-start resolution O(N) for initial scan + O(1) per lookup. After that, sub-second resolution.

**The real solution:** Incremental compilation (Phase 1.3) — only re-parse changed files. Module resolution reuses the previous index. A 1000-file project changing 1 file should rebuild in milliseconds.

### Hot Reload for Game Engines

**Will it work?** Yes, with Phase 2.1. The architecture is proven (Unreal Engine, Unity, many game engines do this).

**Limitations:**
- Struct layout changes require full restart
- Global state migration is tricky (serialization or separate data segments)
- Works best with ECS (Entity Component System) architectures where state is in arrays, not scattered objects

**Effort reality check:** This is weeks of work, not days. The function table indirection alone requires changes to codegen's call emission, the C runtime, and the linker. But it's achievable.

---

## The "Never Freeze, Never Crash" Guarantee

This is achieved through layered defenses:

1. **Resource budgets** (timeout + memory) — Phase 2.3/2.4
2. **Iteration guards** (mono loop, recursion depth) — Phase 0.4
3. **Error on unknown** (no silent wrong code) — Phase 0.3
4. **Graceful degradation** (skip problematic module, continue) — Phase 2.3
5. **Heap safety** (Vec realloc, C runtime limits) — Phase 0.1/0.2

Every crash or hang is treated as a P0 bug. The compiler's reliability is the foundation for everything else.

---

*AXIOM Compiler Improvement Plan — Living document. Updated as milestones are reached.*
