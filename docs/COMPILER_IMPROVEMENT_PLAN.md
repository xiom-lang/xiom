# XIOM Compiler — Improvement Plan

**Date:** 2026-07-03
**Status:** Living plan — updated as the compiler evolves
**Companion to:** `docs/COMPILER_ARCHITECTURE.md` (source of truth for current state), `docs/XIOM_DISTRIBUTION_SPEC.md` (distribution + installer spec), `docs/XIOM_TOOLING_SPEC.md` (debugger, benchmarks, LSP, hot reload), `docs/XIOM_ECOSYSTEM_ROADMAP.md` (packages, FFI, demos)

> This document describes what the compiler SHOULD become. `COMPILER_ARCHITECTURE.md` describes what it IS.
> Every item here is grounded in the three pillars: SAFE, VERIFIED, PRECISE.
> The goal is production-grade: XIOM must compete with Rust, Zig, and C++ in benchmarks — faster where possible, safer by design.

---

## Production-Grade Targets — What "Winning" Means

Before any phase work, these are the hard targets that define production-readiness. Every improvement in this plan maps to one of these targets.

### Speed Target: Within 10% of Rust (Release, O2)

| Benchmark | Rust | XIOM Target | How |
|-----------|------|-------------|-----|
| matmul 1024×1024 | ~2.1s | ≤2.3s | LLVM O2, no-contracts mode |
| n-body simulation | ~4.5s | ≤5.0s | LLVM O2, struct-return optimization |
| binary tree allocate/free | ~1.8s | ≤2.0s | Vec realloc, malloc/free efficiency |
| regex compile + match × 1M | ~0.9s | ≤1.0s | LLVM O2, no-contracts |
| JSON parse 100MB | ~1.2s | ≤1.3s | LLVM O2, stdlib string |
| HTTP server (req/s) | ~45K | ≥40K | stdlib net + O2 |
| generic sort 10M Ints | ~0.4s | ≤0.45s | monomorphised, O2 |

**How to achieve:** LLVM optimization passes (Phase 1.4), no-contracts release mode (Phase 0), Vec realloc (Phase 0.1), inkwell API for direct IR construction (Phase 5.8).

### Safety Target: Zero Memory Bugs in Benchmark Suite

| Category | Rust | XIOM Target |
|----------|------|-------------|
| Use-after-free | Compile error | Compile error (borrow checker) |
| Double-free | Compile error | Compile error (ownership) |
| Null dereference | Compile error (Option) | Compile error (Option) |
| Buffer overflow | Runtime panic or compile error | Runtime trap (contract guards) or compile error (invariants) |
| Data race | Compile error (Send/Sync) | Compile error (lexical borrows + no shared mutable state) |
| Division by zero | Runtime panic | Runtime trap (compile error with Z3 Phase 3) |
| Integer overflow | Release: wrap, Debug: panic | Runtime contract check: `requires: a + b doesn't overflow` |
| Contract violation | None (assertions only) | Runtime trap or compile error (Z3 Phase 3) |

**How to achieve:** Borrow checker (done), lexical scope borrows (done), Vec bounds checking, contract runtime guards (done), Z3 static verification (Phase 3).

### Binary Size Target: ≤2× Rust (Release, stripped)

| Program | Rust | XIOM Target | Notes |
|---------|------|-------------|-------|
| Hello World | ~200KB | ≤400KB | Contract guards add IR, stripped in release |
| JSON parser | ~600KB | ≤1.2MB | Stdlib + serde equivalent |
| HTTP server | ~1.5MB | ≤3.0MB | Stdlib net + HTTP |

**How to achieve:** Contract stripping (`--no-contracts` in release), link-time optimization (LTO via clang), dead code elimination.

### Compile-Time Target: ≤3× Rust (Debug)

| Project Size | Rust (debug) | XIOM Target | Bottleneck |
|-------------|-------------|-------------|-----------|
| 1K lines, 10 fns | ~0.3s | ≤1.0s | Codegen string formatting |
| 10K lines, 50 fns | ~2.5s | ≤8.0s | Monomorphisation + codegen |
| 100K lines, 500 fns | ~25s | ≤75s | Sequential compilation |

**How to achieve:** Parallel monomorphisation (Phase 1.2), incremental compilation (Phase 1.3), indexed catalog (Phase 1.1), inkwell API (Phase 5.8).

### Benchmark Categories to Build

XIOM must compete in all four categories:

**1. Compute-Intensive (measures LLVM codegen quality):**
```
matmul, n-body, mandelbrot, prime sieve, fibonacci, FFT, neural net inference
→ Competes on: raw CPU throughput, LLVM optimization
→ XIOM advantage: contracts stripped in release → zero overhead
```

**2. Memory-Intensive (measures allocation safety):**
```
binary tree alloc/free, b-tree insert/lookup, graph traversal, ring buffer, Vec push/pop at scale
→ Competes on: allocation speed, memory safety guarantees
→ XIOM advantage: ownership model prevents leaks, borrow checker prevents use-after-free
```

**3. I/O-Intensive (measures stdlib quality):**
```
file read/write, JSON parse, HTTP server, CSV processing, log parsing
→ Competes on: I/O throughput, string handling
→ XIOM advantage: contracts on I/O operations catch errors at the boundary
```

**4. Safety-Verification (measures contract system):**
```
contract-intensive functions, invariant-heavy types, borrow stress tests
→ Competes on: bugs caught at compile time vs runtime
→ XIOM advantage: UNIQUE — no other language has first-class contracts
```

### Competitive Benchmark Framework (`xiom bench`)

A dedicated tool that produces standardized, reproducible benchmarks:

```
$ xiom bench --compare rust,zig,cpp --category compute
┌─────────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Benchmark           │ XIOM    │ Rust     │ Zig      │ C++      │
├─────────────────────┼──────────┼──────────┼──────────┼──────────┤
│ matmul 1024×1024    │ 2.18s    │ 2.12s    │ 2.05s    │ 1.98s    │
│ n-body 10M steps    │ 4.42s    │ 4.51s    │ 4.38s    │ 4.15s    │
│ prime sieve 100M    │ 0.89s    │ 0.91s    │ 0.87s    │ 0.82s    │
│ fib(45) recursive   │ 3.21s    │ 3.18s    │ 3.25s    │ 3.10s    │
├─────────────────────┼──────────┼──────────┼──────────┼──────────┤
│ Safety score        │ 100%     │ 100%     │ 45%      │ 0%       │
│ Contract coverage   │ 87%      │ N/A      │ N/A      │ N/A      │
│ Binary size         │ 1.8MB    │ 1.6MB    │ 1.2MB    │ 0.9MB    │
└─────────────────────┴──────────┴──────────┴──────────┴──────────┘
```

**Output formats:** Terminal table (above), JSON, HTML dashboard, PDF report.
**CI integration:** Every commit runs benchmarks and posts results.
**Public dashboard:** `bench.xiom-lang.org` — live comparison against latest Rust/Zig/C++.

---

## Phase Mapping to Production Targets

| Phase | Speed | Safety | Binary Size | Compile Time | Benchmarks |
|-------|-------|--------|-------------|-------------|-----------|
| 0 — Correctness | Fixes Vec crash (precondition for any benchmark) | Div-zero guards, mono loop guard | — | No infinite compiles | — |
| 1 — Performance | O2 passes, parallel mono | — | LTO stripping | Indexed catalog, incremental, parallel mono | — |
| 2 — Advanced | Hot reload (game engines) | Memory budgets, timeout guards | — | Multithreaded compilation | — |
| 3 — Toolchain | — | Z3 static verification (3.0A-G), contract coverage (3.5) | — | — | Visual benchmark tool (3.2), verification dashboard (3.7), WASM playground (3.6) |
| 4 — Self-Hosting | Byte-for-byte IR parity with Rust compiler | Proves correctness | — | — | Ultimate correctness test |
| 5 — Production Pillars | inkwell API (10-50× codegen speedup), FFI binding gen (3.8) | Stdlib with contracts on every function | Contract stripping in release | Build system caching | Full benchmark suite |

---

## Phase 0 — Critical Correctness (P0: Now)

These are NOT features. They are bugs that make the compiler unsafe to use at scale. Fix them before adding anything.

### 0.1 Vec Push Reallocation (V1)

**Problem:** `Vec.push` allocates 128 bytes once. Element 17 writes past the buffer — heap corruption, undefined behavior.

**Solution:** Add capacity check before every `push`. When full, `@realloc` with doubling strategy (128 → 256 → 512...). Emit the realloc logic directly in codegen's Vec builtin path (already has `@malloc`, just wire `@realloc`).

**Effort:** 2-3 hours. Small change in `compile_expr` Vec.push handler.

### 0.2 C Runtime Fixed Limits (V5)

**Problem:** The C runtime (`stdlib/runtime/xiom_runtime.c`) uses fixed-size arrays:
- `MAX_STRUCT_FIELDS = 16`
- `MAX_LOCAL_VARS = 64`
- `MAX_MATCH_ARMS = 16`

Exceeding any silently produces wrong IR.

**Solution:** Replace with dynamically allocated arrays using `malloc`/`realloc`. Start with a reasonable initial capacity (e.g., 64 fields) and grow on overflow. Alternatively, increase limits to 256/1024/256 — enough for any realistic program. The codegen already generates struct field metadata; the C runtime just needs to not truncate.

**Effort:** 1-2 days. Requires C runtime changes + codegen verification.

### 0.3 Unknown Type → Error, Not i64 (V7)

**Problem:** `xiom_to_llvm_type` has `_ => "i64"` as a catch-all. Unknown types silently compile as `i64`, producing wrong IR instead of a clear error.

**Solution:** Return `Result` instead of `String` from `llvm_type_for`. On unknown type, emit a structured error with the type name and source location. Never silently default to `i64`.

**Effort:** 3-4 hours. Touches ~40 call sites in codegen.

### 0.4 Generic Monomorphisation Loop Guard (V6)

**Problem:** If a monomorphised function body triggers another monomorphisation of the same function (e.g., recursive generic), the worklist grows unboundedly → infinite compile.

**Solution:** Hard limit of 5000 total monomorphisation iterations. On overflow, emit a clear error: "monomorphisation limit exceeded (loop in generic `Foo[Bar]`?)". Track a hash of `(fn_name, concrete_types)` per iteration to detect true cycles.

**Effort:** 1-2 hours. Add counter + cycle detection to `compile_generic_monomorphisations`.

### 0.5 Division-by-Zero in Compiler Internals

**Problem:** The compiler itself does unchecked integer division in several places (e.g., `sdiv` during offset calculations). A compiler crash from div-by-zero in the COMPILER is unacceptable.

**Solution:** Audit all `sdiv`/`srem` in Rust code (`xiom-codegen`, `xiom-check`, `xiomc`). Wrap in `checked_div` with proper error handling. The target XIOM programs already have div-zero guards (V4 fixed) — the compiler itself needs them too.

**Effort:** 2-3 hours. Grep + audit ~15-20 division sites.

---

## Phase 1 — Performance Foundations (P1: Next)

These make the compiler fast enough for absurd benchmarks. Without them, 100+ generics or 1000+ modules will be slow but won't crash.

### 1.1 Indexed Module Catalog

**Problem:** `ModuleCatalog::load_module` walks all source directories and reads every `.xi` header on cache miss. For 1000 files, cold start = 1000 file reads × M lookups = O(N×M).

**Solution:**
1. On first `find_owned()` call OR at `xiomc` startup, build a `HashMap<String, String>` (module_path → file_path) by scanning source_dirs once.
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

**Problem:** Every `xiomc` invocation re-parses every file. For a 1000-file project changing one line, this is wasteful.

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
3. **Function table indirection:** Instead of direct `call @foo`, use a function pointer table `@xiom_fn_table`. Hot-reload swaps the pointer.
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

### 3.0 Advanced Contract Verification

The contract system is XIOM's killer feature. Phase 1 provides runtime guards. Phase 3 makes contracts zero-cost and statically proven. This is what separates XIOM from every other systems language.

#### 3.0A SMT-Based Static Verification (Z3 Integration)

**Goal:** Prove contracts at compile time. No runtime guards, no performance penalty.

**Architecture:**
```
Source.xi → Parser → AST → Checker → Contract IR → Z3 SMT Solver → Proof / Counterexample
```

**How it works:**
1. For each function with contracts, the verifier extracts the contract clauses as SMT-LIB assertions
2. The function body is symbolically executed to produce a path condition
3. Z3 checks: does the path condition imply the `ensures` clause, assuming the `requires` clause?
4. If SAT (satisfiable): contract is provably correct → strip runtime guard, emit `; verified: divide.ensures`
5. If UNSAT (unsatisfiable): contract can be violated → emit compile error with counterexample

**Example:**
```xiom
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures:  result * b == a
{
  return a / b;
}

// Z3 proves: (b != 0.0) ∧ (result = a / b) → (result * b == a)
// Result: VERIFIED. Runtime guard eliminated. Zero overhead.
```

**Supported theories:**
- QF_NRA (non-linear real arithmetic) — floats, division, multiplication
- QF_LIA (linear integer arithmetic) — integers, addition, subtraction
- QF_ABV (arrays + bitvectors) — Vec bounds, bit operations
- UF (uninterpreted functions) — function calls with contracts

**Limitations:**
- Loops require explicit `invariant` annotations for the verifier to reason about them
- Recursive functions need termination proofs (well-founded ordering)
- Heap-allocated data (Vec, Map) requires a memory model — significant complexity
- Z3 timeout per function (configurable, default 30s)

**Effort:** 3-6 months for initial Z3 integration. Ongoing for covering more theories. The `axiom-verify` crate already generates SMT-LIB — this extends it to compile-time integration.

#### 3.0B Abstract Interpretation

**Goal:** Prove invariants for array bounds, integer ranges, and numerical properties WITHOUT full SMT solving.

**Architecture:**
```
AST → Abstract Domain (intervals, octagons, polyhedra) → Fixpoint computation → Verified invariants
```

**Best for:**
- Array/Vec bounds checking: `arr[i]` requires `0 <= i < arr.len()`
- Integer overflow detection: `a + b` doesn't overflow
- Simple numerical invariants: `x > 0`, `i <= n`

**Effort:** 2-3 months. Simpler than Z3, covers 80% of common contracts.

#### 3.0C Contract Composition Analysis

**Goal:** Automatically verify that function A's `ensures` satisfy function B's `requires` when A calls B.

**Example:**
```xiom
fn transfer(from: &mut Account, to: &mut Account, amount: Float64)
  requires: from.balance >= amount
  ensures:  from.balance == from.balance@pre - amount
  ensures:  to.balance == to.balance@pre + amount
{
  from.withdraw(amount);   // withdraw requires: balance >= amount
  to.deposit(amount);      // deposit requires: amount > 0
}
```

**Verifier checks:**
1. At `from.withdraw(amount)`: does `from.balance >= amount` (the `requires` of `transfer`) imply `balance >= amount` (the `requires` of `withdraw`)? → YES
2. At `to.deposit(amount)`: does `amount > 0` hold? → Not guaranteed by `transfer`'s contracts → WARNING: missing `requires: amount > 0` on `transfer`

**Effort:** 1-2 months. Builds on Z3 integration. Walks the call graph.

#### 3.0D Symbolic Execution Engine

**Goal:** Explore all execution paths symbolically, generate test cases automatically from contracts.

**How it works:**
1. For each function, create symbolic inputs (variables instead of concrete values)
2. Execute the function symbolically, collecting path constraints
3. For each path, Z3 generates a concrete input that satisfies the path constraints
4. These become test cases: `fn test_divide_generated_01() { assert_eq!(divide(10.0, 2.0), 5.0); }`

**Output:** Auto-generated test suite with 100% path coverage.

**Effort:** 2-3 months. Requires Z3 integration first.

#### 3.0E AI-Assisted Proof

**Goal:** Use LLMs to suggest loop invariants, termination measures, and proof steps for complex contracts.

**How it works:**
1. When the verifier cannot prove a contract (e.g., because a loop invariant is missing), it generates a prompt for an LLM
2. The prompt includes: the function signature, contracts, body, and the specific verification failure
3. The LLM suggests an invariant annotation
4. The verifier retries with the suggested invariant
5. If it passes, the invariant is added to the source code

**Example prompt:**
```
The following function's `ensures` cannot be proven because the loop
in `sum_range` has no invariant. Suggest an invariant annotation:

fn sum_range(lo: Int, hi: Int) -> Int
  requires: lo <= hi
  ensures:  result == (lo + hi) * (hi - lo + 1) / 2
{
  var total = 0;
  var i = lo;
  while i <= hi {
    total = total + i;
    i = i + 1;
  }
  return total;
}
```

**LLM-suggested invariant:** `invariant: total == (lo + i - 1) * (i - lo) / 2`

**Effort:** 1-2 months for prototype. Requires Z3 integration + LLM API.

#### 3.0F Modular Verification

**Goal:** Verify one module at a time, using only the contracts of imported modules. Changes to module A do not require re-verification of module B.

**Architecture:**
1. Each module exports a verification interface: its public function signatures + contracts
2. The verifier treats imported functions as uninterpreted with their contracts as axioms
3. A change to module A triggers re-verification of A only — B is re-verified only if A's verification interface changed

**Effort:** 2-3 months. Requires contract composition analysis (3.0C) as a foundation.

#### Verification Modes Summary

| Mode | Phase | Overhead | Use Case |
|------|-------|----------|----------|
| Runtime guards | 1 (now) | ~5-10 IR instructions per contract | Development, testing |
| Abstract interpretation | 3 | None (compile-time) | Array bounds, integer ranges |
| Z3 static proof | 3 | None (compile-time) | Full function contracts |
| Symbolic execution | 3 | None (compile-time) | Auto-generated tests |
| Contract composition | 3 | None (compile-time) | Call chain verification |
| AI-assisted proof | 3 | None (compile-time) | Complex contracts needing invariants |
| Modular verification | 3 | None (compile-time) | Large multi-module projects |
| `--no-contracts` | 1 (now) | None (stripped) | Production, benchmarks |

### 3.0G Borrow System Enhancement — Lifetime Tracking

**Goal:** Remove the two primary borrow system restrictions: storing borrows in struct fields and returning borrows from functions.

**Current state (Phase 1):** XIOM uses lexical scope borrowing — borrows are valid from `let r = &x` to the closing `}` of the block. This is simple and correct but restrictive:
- ❌ `type Container = { ref: &Vec[Int]; }` — compile error: cannot store borrow in struct
- ❌ `fn get_first(v: &Vec[Int]) -> &Int { return &v[0]; }` — compile error: cannot return borrow

**Alternatives developers use today:**
- Clone values instead of borrowing (`v[0].clone()`)
- Return owned types, not references
- Use indices instead of references (`return 0` instead of `&v[0]`)
- Allocate in arenas or use `Rc[T]` for shared ownership

**Phase 3 plan — Relaxed borrow rules:**

| Milestone | What | Effort |
|-----------|------|--------|
| 3.0G.1 | **Borrow-from-borrow:** Allow returning a borrow that is derived from a borrow parameter. The checker must verify: output lifetime ≤ input lifetime. | 2-3 weeks |
| 3.0G.2 | **Struct field borrows:** Allow storing borrows in struct fields with lexical lifetime tracking. Struct lifetime = min(field lifetimes). | 2-3 weeks |
| 3.0G.3 | **Lifetime elision:** Simple heuristic rules (like Rust's elision) so most functions need no annotations. | 1 week |
| 3.0G.4 | **Explicit lifetime annotations:** For complex cases, allow `fn foo<'a>(x: &'a Int) -> &'a Int` syntax. | 1-2 weeks |

**Design principle:** Keep it simpler than Rust. No lifetime subtyping, no variance, no HRTB (Higher-Ranked Trait Bounds). The goal is to cover the 90% use case (iterators, views, zero-copy parsers) without the complexity burden of Rust's full lifetime system.

**Example after Phase 3:**
```xiom
// 3.0G.1: Return borrow derived from parameter (today: compile error)
fn first[T](v: &Vec[T]) -> &T { return &v[0]; }

// 3.0G.2: Store borrow in struct (today: compile error)
type Window<'a> = { data: &'a [Int]; start: Int; end: Int; }

// 3.0G.3: Lifetime elision — no annotations needed for common patterns
fn get_ref(v: &Vec[Int]) -> &Int { return &v[0]; }  // elided lifetime
fn get_mut(v: &mut Vec[Int]) -> &mut Int { return &mut v[0]; }
```

**Total effort:** 5-8 weeks for the full borrow enhancement system. This is a Phase 3 feature — DO NOT start until Phase 0-2 hardening is complete and the compiler is stable.

### 3.1 Debugger (DAP-based)

**Architecture:** The debugger is a separate process that speaks the Debug Adapter Protocol (DAP) to VS Code / JetBrains / etc. It controls the target process via platform debug APIs.

**Contract-aware debugging:**
- Before each contract guard, the codegen emits a `@llvm.dbg.value` intrinsic with the contract expression text and expected values.
- On `@llvm.trap()`, the debugger captures: which contract, expected vs actual, source location, call stack.
- "Explain the violation" mode: renders the contract in natural language with highlighted failing sub-expression.

**Ownership visualization:**
- Compiler emits DWARF debug info with custom `DW_TAG_xiom_ownership` entries.
- Debugger renders: variable lifetime bars (color-coded: green=alive, yellow=borrowed, red=moved), borrow graph arrows, moved-from markers.

**Effort:** 2-3 months. Requires DWARF emission in codegen + DAP server implementation.

### 3.2 Visual Benchmark Tool

**Goal:** Compare XIOM against Rust, Zig, C++, Go — with beautiful visualizations.

**Architecture:**
1. `xiom bench` compiles each benchmark, runs it N times, collects metrics.
2. Results stored as JSON with schema: `{bench_name, language, metrics: {time_ns, mem_bytes, binary_size, contract_count}}`.
3. A static HTML dashboard (built with XIOM → WASM) renders comparisons.
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

**Command:** `xiomc` (the official XIOM compiler and toolchain)

#### Core Philosophy
- Simple, intuitive, powerful
- Excellent defaults — `xiom build` works without configuration
- Rich feedback and helpful error messages (Phase 5.6)
- First-class support for AI agents (`--ai`, `--diagnostics=json`, `--dump-contracts`)

#### Main Commands

| Command | Description | Example |
|---------|-------------|---------|
| `xiom build` | Build project or file | `xiom build` |
| `xiom run` | Build + run | `xiom run main.xi` |
| `xiom check` | Type check only (fast, no codegen) | `xiom check` |
| `xiom test` | Run tests | `xiom test` |
| `xiom fmt` | Canonical formatter | `xiom fmt` |
| `xiom lint` | Lint + style check | `xiom lint` |
| `xiom debug` | Build with debug info + launch debugger | `xiom debug` |
| `xiom bench` | Run benchmarks | `xiom bench` |
| `xiom doc` | Generate documentation | `xiom doc` |
| `xiom new` | Create new project | `xiom new myapp` |
| `xiom init` | Initialize in current folder | `xiom init` |
| `xiom install` | Install package from registry | `xiom install http-server` |
| `xiom publish` | Publish package to registry | `xiom publish` |
| `xiom clean` | Remove build artifacts | `xiom clean` |
| `xiom version` | Print version + platform info | `xiom --version` |

#### Build Options

| Flag | Description |
|------|-------------|
| `--target native/wasm/arm/riscv` | Target platform |
| `--release` | Optimized build (O3 + strip contracts) |
| `--debug` | Debug symbols (DWARF/PDB) |
| `--shared` | Build as `.dll` / `.so` |
| `--static` | Build static library |
| `--no-contracts` | Disable contract checks |
| `--emit ir` | Output XIOM IR |
| `--emit llvm` | Output LLVM IR |
| `--emit obj` | Output object file |
| `-o <file>` | Output filename |
| `--diagnostics=json` | Structured output for AI/tools |
| `--dump-contracts` | Export contract index |

#### Developer Experience

| Flag | Description |
|------|-------------|
| `--watch` | Watch mode (recompile on change) |
| `--hot-reload` | Enable hot reloading (for games, Phase 2.1) |
| `--ai` | AI-friendly mode (verbose explanations, fix suggestions) |
| `--incremental` | Incremental compilation (Phase 1.3) |

#### Project Management

```
xiom new mygame --template game    # creates project with game template
xiom init                          # initialize in current folder
xiom package                       # manage package.xi manifest
```

#### Project Scaffold (`xiom new hello`)

```
hello/
├── package.xi           ← project manifest (see Section 11 of distribution spec)
├── src/
│   └── main.xi          ← entry point
├── tests/
│   └── test_main.xi     ← tests
├── examples/
│   └── demo.xi          ← example usage
├── benches/
│   └── bench_main.xi    ← benchmarks
└── xiom.lock            ← lockfile (committed to VCS)
```

### 3.5 Contract Coverage Analyzer

**Goal:** Unique metric no other language has — what percentage of contracts are exercised by tests.

**How it works:**
1. During codegen, every contract guard is instrumented: `@xiom_contract_hit_count_N`
2. `xiom test --coverage` runs the test suite and reads counters
3. Outputs a report: which contracts were hit, which were never triggered

```
Contract Coverage: 87% (234/268 contracts exercised)
  ✓ divide.requires:b!=0       hit 142 times
  ✓ divide.ensures:result*b==a  hit 142 times
  ✗ sqrt_newton.ensures:...     NEVER triggered — add a test
```

**Compiler changes needed:** Instrumentation pass in codegen, `--coverage` flag, report generation.
**Effort:** 1-2 weeks. Primarily codegen instrumentation + test runner.

### 3.6 WASM Compiler Playground

**Goal:** XIOM compiler compiles to WASM — instant try-before-install at `play.xiom-lang.org`. No backend. No account.

**Compiler changes needed:**
- `xiomc` compiles to WASM with virtual filesystem (catalog uses in-memory FS)
- `--target wasm-playground` flag skips clang, returns IR as string for browser display
- Contract visualization in browser: highlighted pass/fail on contract clauses

**Effort:** 1-2 weeks for MVP. Mostly making compiler WASM-compatible + static HTML UI.

### 3.7 Formal Verification Dashboard

**Goal:** Show which contracts are statically proven vs runtime-guarded. Trust signal for safety-critical users.

**Compiler changes needed:**
- `xiom build --verify` emits verification metadata JSON (which contracts proven, which timed out)
- Static HTML dashboard reads JSON and renders proven/runtime/unverified breakdown

**Effort:** 1-2 weeks. Builds on Z3 integration (3.0A). Primarily metadata emission + HTML.

### 3.8 FFI Binding Generator

**Goal:** Mechanical generation of XIOM FFI wrappers from C headers, with auto-inferred contracts.

**How it works:**
1. `xiom bind --header math.h` parses the C header
2. Generates `extern "C" { ... }` block with correct types
3. Wraps each function in a safe XIOM function with inferred contracts:
   - `int* foo(int* p)` → `requires: p != null` (non-null pointer)
   - `size_t strlen(const char* s)` → `ensures: result >= 0`

**Effort:** 3-4 weeks for MVP. Requires C header parser + inference rules.

### 3.9 Code Translator (`xiom translate`)

**Goal:** Translate existing C, Zig, and Rust code to XIOM. Massive adoption lever — "Bring your existing code to XIOM."

**Phase 1 — `xiom translate-c`:**
- Parse C headers and source files
- Map C types to XIOM types: `int` → `Int`, `float` → `Float32`, `double` → `Float64`, `char*` → `Str`, `void*` → `*UInt8`
- Generate `extern "C" { ... }` blocks
- Auto-infer contracts on generated wrappers (non-null pointers → `requires: ptr != null`)

**Phase 2 — Expand to Zig and Rust:**
- Zig: similar semantics, straightforward translation
- Rust: via C ABI export layer

**Effort:** 4-6 weeks per language. C is the biggest win — start there.

**Why this matters:** Zig's `translate-c` is one of its most praised features. A XIOM equivalent with auto-inferred contracts would be strictly better.

---

## Phase 4 — Self-Hosting (P4: After Stability)

Self-hosting means the XIOM compiler is written in XIOM, compiled by the previous version of itself. This is the ultimate correctness test.

**Prerequisites:**
- All Phase 0-1 bugs fixed
- Full language surface stable (no breaking syntax changes for 6+ months)
- Standard library mature enough to write a compiler (string handling, file I/O, collections, FFI)
- Benchmark suite passing at 100%
- The Rust compiler is kept as the PERMANENT bootstrap fallback — never deleted

**Bootstrapping sequence:**
1. Write `xiom-lexer.xi`, `xiom-parser.xi`, `xiom-check.xi`, `xiom-codegen.xi` in XIOM.
2. Compile Phase 4 XIOM compiler with Phase 3 Rust compiler.
3. Compile Phase 4 XIOM compiler with Phase 4 XIOM compiler (self-compile).
4. Diff the output binaries — byte-for-byte identical → bootstrap complete.

**Do NOT start this until Phase 3 is rock-solid.** Every self-hosting attempt on an unstable compiler wastes weeks debugging the compiler AND the compiler-being-compiled simultaneously.

### Why a Self-Hosted Compiler Outperforms the Rust Bootstrap

A XIOM compiler written in XIOM can be MORE performant and MORE secure than the current Rust bootstrap. This is not speculation — it follows from the language's own features applied to its own implementation:

| Advantage | How | Impact |
|-----------|-----|--------|
| **Contracts on compiler internals** | `requires`/`ensures`/`invariant` on the lexer, parser, checker, codegen | Catches bugs in the compiler BEFORE they produce wrong output. Every optimization pass is contract-verified. |
| **Deeper comptime** | `comptime` can precompute parse tables, keyword sets, precedence maps at compile time | Faster compiler startup, smaller binary |
| **Zero-cost ownership in the compiler** | The compiler's own memory management benefits from borrow checking — no GC pauses during compilation | Deterministic latency, no allocation spikes |
| **Specialized optimizations for XIOM patterns** | The compiler can optimize for contracts, ownership, structural interfaces — things Rust's compiler doesn't know about | Better codegen for XIOM-specific patterns |
| **Custom allocators** | The compiler can use arena allocators for AST nodes, bump allocators for IR emission | 2-3× less memory, fewer malloc/free calls |
| **Verifiable correctness** | Z3 static verification on the compiler's own critical paths (Phase 3) | Provably correct type checker, borrow checker |

**When to self-host:**
1. Phase 3 is complete (Z3, debugger, LSP, CLI, benchmarks)
2. Language is stable — no breaking syntax changes for 6+ months
3. Standard library is mature — string handling, file I/O, collections, FFI
4. 500+ tests pass, including full benchmark suite
5. The Rust compiler is kept as PERMANENT bootstrap fallback

**The bootstrap sequence:**
1. Write `xiom-lexer.xi`, `xiom-parser.xi`, `xiom-check.xi`, `xiom-codegen.xi` in XIOM
2. Compile with Rust `xiomc` → produces `xiomc-v1` (native binary)
3. `xiomc-v1` compiles itself → produces `xiomc-v2`
4. Diff `xiomc-v1` and `xiomc-v2` output on the full test suite → byte-for-byte identical
5. `xiomc-v2` replaces Rust `xiomc` as the primary compiler

**Target timeline:** 18-24 months from now. The self-hosted compiler is the CAPSTONE, not the foundation.

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
| CLI toolchain | FUTURE | `xiom build/run/test/fmt/bench` |
| Self-hosting | LAST | After everything above is stable |

---

## Honest Assessment: What Changes to Handle Absurd Benchmarks

### 10K Contracts

**Will it compile?** Yes, with Phase 0 fixes applied. Runtime guards scale linearly.

**Will it be fast?** The IR will be large (50K-100K extra instructions). `opt -O1` (Phase 1.4) helps but doesn't eliminate the overhead. Contracts are fundamentally a binary-size vs safety tradeoff.

**The real solution:** Z3 static verification (Phase 3 of the XIOM roadmap). Prove contracts at compile time → eliminate runtime guards for statically-proven contracts → 0 overhead. This requires: translating the full type system to SMT theories, modeling heap state, loop invariants. This is a 3-6 month effort for a dedicated team — not a quick fix.

**Until Z3:** Accept that contracts have runtime cost, and make `--no-contracts` the production flag. Development uses contracts for catching bugs; release builds strip them.

### 100+ Generic Instantiations

**Will it compile?** Yes, but slowly (seconds to minutes).

**Will it be fast?** Parallel monomorphisation (Phase 1.2) cuts compile time by ~4x on a 4-core machine. Dedup eliminates redundant work.

**The real solution:** The compiler only monomorphises what's USED. If a generic function is instantiated 100 times but only 5 are called, only those 5 should be emitted. Current codegen eagerly tracks all instantiations — change to lazy/deferred emission.

### 1000+ Module Files

**Will it compile?** Yes, but the first `xiomc` invocation will spend seconds scanning for modules.

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

## Phase 5 — Production Pillars (Missing from Plan, Must Add)

The improvement plan covers compiler internals. But a compiler alone is not a language. Five additional pillars are needed to reach production-grade. These are not "nice to have" — they are prerequisites for real-world adoption.

### 5.1 Standard Library Completion

**Current state:** 45 stdlib modules listed in `AI_CONTEXT.html`. Most are stubs or minimal implementations. Only `core`, `io`, `collections`, `string`, `math`, `ffi`, `async` are partially functional.

**What production needs:**

| Category | Modules | Priority |
|----------|---------|----------|
| **I/O** | File read/write, stdin/stdout/stderr, buffered I/O, paths, directories, temp files | P0 |
| **Networking** | TCP/UDP sockets, HTTP client, HTTP server, TLS (via FFI), WebSocket, DNS | P1 |
| **Concurrency** | Threads, mutex, rwlock, channels (real), atomics, barriers, thread pool | P1 |
| **Collections** | HashMap, BTreeMap, HashSet, BTreeSet, LinkedList, BinaryHeap, VecDeque | P0 |
| **String** | UTF-8 validation, case conversion, split/join/replace, pattern matching, regex | P0 |
| **Math** | Full trig, log, exp, rounding, random, complex numbers, statistics | P1 |
| **Time** | System time, monotonic clock, Duration, Timer, sleep, timeouts | P1 |
| **OS** | Process spawning, environment vars, command-line args, exit codes, signals | P1 |
| **Serialization** | JSON, binary (bincode), base64, hex, CSV | P1 |
| **Crypto** | SHA-256, AES, HMAC, RSA (via FFI), secure random | P2 |
| **Compression** | gzip, zlib, deflate (via FFI) | P2 |
| **FFI** | Full `extern "C"` with struct layout, callback support, `unsafe` blocks | P0 |

**Every module function should have contracts.** XIOM's killer feature — contracts — must be demonstrated in the stdlib itself. `fn read_file(path: Str) -> Result[Vec[UInt8], IOError] requires: path.len() > 0 ensures: result.is_ok() => result.unwrap().len() > 0`.

**Effort:** 3-6 months. Largest single body of work after the compiler itself. Can be incremental — ship modules as they stabilize.

### 5.2 Package Manager + Registry

**Current state:** No package manager. No registry. Multi-file projects work via merge, but there's no dependency resolution.

**What production needs:**

1. **`xiom install <package>`** — downloads from registry, resolves transitive deps, locks versions
2. **`xiom publish`** — uploads package to registry with version, metadata
3. **`xiom new <project>`** — scaffolds a new project with `package.xi`, `src/main.xi`, `tests/`
4. **`package.xi` manifest** — name, version, dependencies with semver ranges, authors, license
5. **Lockfile** (`xiom.lock`) — hashed dependency tree for reproducible builds
6. **Registry** — simplest possible: a Git repo with `packages.json` index. Each package is its own repo. No database needed until 100+ packages.

**Architecture (MVP):**
```
xiom install http-server
  → reads https://registry.xiom-lang.org/packages.json
  → finds http-server v1.2.0
  → clones https://github.com/xiom-lang/http-server (tag v1.2.0)
  → reads package.xi for transitive deps
  → resolves tree, writes xiom.lock
  → `use xiom.http` now resolves
```

**Design principle:** Cargo got it right. Don't innovate on the package model — specialize on the contract-aware verification. `xiom install` should also verify contracts of dependencies.

**Effort:** 2-3 months for MVP. Registry is 1 JSON file + Git hosting. The package manager itself is the complex part — resolve, fetch, verify, cache.

### 5.3 Build System

**Current state:** `xiomc` accepts files via CLI. No build configuration. No profiles. No build scripts.

**What production needs:**

1. **`xiom build`** — reads `package.xi`, resolves deps, compiles all source files, links binary
2. **Profiles:** `dev` (fast compile, no optimize), `release` (optimize + strip contracts), `bench` (optimize + instrumentation)
3. **Build scripts** (`build.xi`) — XIOM code that runs at build time for code generation, FFI binding generation, asset processing
4. **`--features`** — conditional compilation via feature flags
5. **`xiom check`** — type-check only, no codegen (fast feedback loop)
6. **`xiom clean`** — remove build artifacts
7. **Incremental builds** — recompile only changed files (Phase 1.3 from compiler plan)

**Design principle:** Cargo.toml is the gold standard. `package.xi` should be equally simple. No Makefiles. No CMake. No build.rs complexity — just a declarative manifest and an optional build script.

**Effort:** 1-2 months. Builds on incremental compilation. Mostly CLI orchestration + file system.

### 5.4 Testing Framework

**Current state:** Rust `cargo test` runs compiler unit tests. No XIOM-native test framework.

**What production needs:**

1. **`xiom test`** — discovers and runs all `fn test_*()` functions in `tests/` directory
2. **Contract-aware assertions:** `assert_eq!(a, b)`, `assert_contract!(fn_call)` — verifies contracts pass
3. **Test fixtures:** `setup()` / `teardown()` per test module
4. **Benchmark mode:** `xiom bench` — runs `fn bench_*()` functions N times, reports statistics
5. **Coverage:** `xiom test --coverage` — contract coverage (which contracts are exercised by tests)
6. **Property-based testing:** `xiom test --fuzz` — random input generation with contract validation

**Contract coverage is XIOM's unique testing feature:**
```
Contract coverage: 87%
  ✓ bench_math.xi:add              requires: a + b doesn't overflow
  ✓ bench_math.xi:divide           requires: b != 0.0
  ✗ bench_math.xi:sqrt_newton      ensures: result * result ≈ x
  ↑ This ensure was never triggered by any test
```

**Effort:** 1-2 months. Test discovery + runner is straightforward. Contract coverage tracking requires compiler instrumentation.

### 5.5 Documentation Generator

**Current state:** No documentation tooling. `docs/` directory is manually written Markdown.

**What production needs:**

1. **`xiom doc`** — generates HTML documentation from source code
2. **Contract extraction:** every `requires`/`ensures`/`invariant` rendered as structured docs
3. **Type signatures** with links to referenced types
4. **Examples** extracted from doc comments, compiled and tested
5. **Search:** full-text search across all docs
6. **`--contracts-json`** — machine-readable contract index for AI tooling (already planned in Phase 3 recommendations)

**Design:** `rustdoc` is the model. Javadoc-style comments (`///`) above declarations. Markdown in comments. Generated HTML with search.

**Effort:** 3-4 weeks. Mostly HTML generation from AST traversal.

### 5.6 Error Message Quality

**Current state:** `error[T001]: 76:3: cannot call 'len' on this expression`

**What production needs (XIOM's 4-point error philosophy from the spec):**

| Component | Current | Target |
|-----------|---------|--------|
| **LOCATION** | Line:col ✓ | Point to EXACT token |
| **CAUSE** | "cannot call 'len' on this expression" — vague | "`Str` has no method `len`. Use `xiom::string::str_len(s: Str) -> Int` instead." |
| **IMPLICATION** | None | "Without this, the compiler cannot verify the return type." |
| **SUGGESTION** | None | "help: add `use xiom.string;` and call `string.str_len(name)`" |

**Examples from production compilers to match:**

```
// GOOD (Rust-level):
error[E0599]: no method named `len` found for type `Str`
  --> src/main.xi:76:3
   |
76 |   name.len()
   |       ^^^ method not found in `Str`
   |
   = help: `Str` is a UTF-8 slice. Use `xiom::string::str_len(s: Str) -> Int`.
   = note: `Str` is not `Vec`. `.len()` on Vec returns element count, but Str needs
     UTF-8 length computation.
```

**Effort:** Ongoing. Add suggestions incrementally. Each error type gets a dedicated diagnostic with cause/implication/suggestion.

### 5.7 Cross-Platform CI

**Current state:** Windows-only development. No CI/CD. No Linux or macOS builds.

**What production needs:**

| Target | Priority |
|--------|----------|
| `x86_64-unknown-linux-gnu` | P0 |
| `x86_64-pc-windows-msvc` | P0 (current) |
| `x86_64-apple-darwin` | P1 |
| `aarch64-apple-darwin` | P1 |
| `aarch64-unknown-linux-gnu` | P2 |
| `wasm32-unknown-unknown` | P1 |

**CI matrix (GitHub Actions):**
```yaml
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --all
      - run: cargo run -p xiomc -- --run examples/phase1_full.xi
```

**Effort:** 1-2 days for initial CI setup. Ongoing maintenance. The compiler already generates correct LLVM IR for all targets — CI just needs to verify it.

### 5.8 LLVM API Integration (inkwell)

**Current state:** Text IR emission — every instruction is `format!("  {tmp} = add i64 ...")`. No LLVM library dependency.

**Why this matters:** Text IR is the #1 compile-time bottleneck. For a 10K-line file, the compiler spends most of its time formatting strings and writing them to a buffer. Using LLVM's C API (via inkwell) would:
- 10-50x faster codegen (no string formatting, direct in-memory IR construction)
- Built-in optimization passes (O1/O2/O3 without shelling to `opt`)
- Built-in verification (`LLVMVerifyModule`)
- Debug info generation (DWARF via DIBuilder)
- JIT compilation (for hot-reload, see Phase 2.1)

**Why NOT do it yet:** Inkwell adds a heavy dependency (LLVM shared library), complicates the build, and the current text-IR approach works. This is a P2 optimization — do it when compile time is the bottleneck, not before.

**Effort:** 3-4 weeks to port codegen to inkwell. Significant code change — ~1000 lines of format!() become builder methods.

---

## Updated Priority Matrix

| What | When | Why |
|------|------|-----|
| **Compiler P0 fixes** (V1-V7) | NOW | Crashes block all benchmarking |
| **Compiler P1** (performance) | NEXT | Speed for large projects |
| **Stdlib core** (I/O, collections, string) | NEXT | Required for any real program |
| **Error messages** | ONGOING | Developer experience |
| **Cross-platform CI** | THIS WEEK | 1-2 day setup, immediate trust signal |
| **Build system** (`xiom build`) | SOON | Needed before any external user tries XIOM |
| **Package manager** (`xiom install`) | SOON | Needed for ecosystem |
| **Testing framework** (`xiom test`) | SOON | Needed for reliability |
| **Documentation** (`xiom doc`) | SOON | Needed for adoption |
| **Compiler P2** (hot reload, multithreaded) | LATER | Advanced features |
| **LLVM API (inkwell)** | LATER | Performance optimization |
| **Compiler P3** (debugger, LSP, Z3) | FUTURE | Toolchain maturity |
| **Contract coverage analyzer** | FUTURE | Unique metric — no other language has this |
| **WASM playground** | FUTURE | Instant try-before-install |
| **Verification dashboard** | FUTURE | Trust signal for safety-critical |
| **FFI binding generator** | FUTURE | Mechanical C→XIOM wrappers with contracts |
| **Stdlib full** (net, crypto, compression) | FUTURE | Ecosystem growth |
| **Self-hosting** | LAST | Final validation |

---

## Production Readiness Checklist

| Capability | Status | Target |
|-----------|--------|--------|
| Compiles non-trivial programs | ✓ (with Vec ≤16) | Vec realloc |
| Passes entire test suite | ✓ 186/186 | 500+ tests |
| Compiles on all 3 major OSes | ✗ Windows only | CI matrix |
| Has package manager | ✗ | `xiom install` |
| Has build system | ✗ | `xiom build` |
| Has testing framework | ✗ | `xiom test` |
| Has documentation generator | ✗ | `xiom doc` |
| Has LSP for IDE support | ✗ | Enhanced LSP |
| Has debugger | ✗ | DAP debugger |
| Has production users | ✗ | Showcase projects |
| Has community RFC process | ✗ | Governance model |
| Has reproducible builds | ✗ | Lockfile + CI |
| Has security response process | ✗ | SECURITY.md |

**When all 13 items are ✓, XIOM is production-ready.** The compiler internals are ~40% of the work. The remaining 60% is everything around the compiler.
