# AXIOM Compiler — Improvement Plan

**Date:** 2026-07-03
**Status:** Living plan — updated as the compiler evolves
**Companion to:** `docs/COMPILER_ARCHITECTURE.md` (source of truth for current state)

> This document describes what the compiler SHOULD become. `COMPILER_ARCHITECTURE.md` describes what it IS.
> Every item here is grounded in the three pillars: SAFE, VERIFIED, PRECISE.
> The goal is production-grade: AXIOM must compete with Rust, Zig, and C++ in benchmarks — faster where possible, safer by design.

---

## Production-Grade Targets — What "Winning" Means

Before any phase work, these are the hard targets that define production-readiness. Every improvement in this plan maps to one of these targets.

### Speed Target: Within 10% of Rust (Release, O2)

| Benchmark | Rust | AXIOM Target | How |
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

| Category | Rust | AXIOM Target |
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

| Program | Rust | AXIOM Target | Notes |
|---------|------|-------------|-------|
| Hello World | ~200KB | ≤400KB | Contract guards add IR, stripped in release |
| JSON parser | ~600KB | ≤1.2MB | Stdlib + serde equivalent |
| HTTP server | ~1.5MB | ≤3.0MB | Stdlib net + HTTP |

**How to achieve:** Contract stripping (`--no-contracts` in release), link-time optimization (LTO via clang), dead code elimination.

### Compile-Time Target: ≤3× Rust (Debug)

| Project Size | Rust (debug) | AXIOM Target | Bottleneck |
|-------------|-------------|-------------|-----------|
| 1K lines, 10 fns | ~0.3s | ≤1.0s | Codegen string formatting |
| 10K lines, 50 fns | ~2.5s | ≤8.0s | Monomorphisation + codegen |
| 100K lines, 500 fns | ~25s | ≤75s | Sequential compilation |

**How to achieve:** Parallel monomorphisation (Phase 1.2), incremental compilation (Phase 1.3), indexed catalog (Phase 1.1), inkwell API (Phase 5.8).

### Benchmark Categories to Build

AXIOM must compete in all four categories:

**1. Compute-Intensive (measures LLVM codegen quality):**
```
matmul, n-body, mandelbrot, prime sieve, fibonacci, FFT, neural net inference
→ Competes on: raw CPU throughput, LLVM optimization
→ AXIOM advantage: contracts stripped in release → zero overhead
```

**2. Memory-Intensive (measures allocation safety):**
```
binary tree alloc/free, b-tree insert/lookup, graph traversal, ring buffer, Vec push/pop at scale
→ Competes on: allocation speed, memory safety guarantees
→ AXIOM advantage: ownership model prevents leaks, borrow checker prevents use-after-free
```

**3. I/O-Intensive (measures stdlib quality):**
```
file read/write, JSON parse, HTTP server, CSV processing, log parsing
→ Competes on: I/O throughput, string handling
→ AXIOM advantage: contracts on I/O operations catch errors at the boundary
```

**4. Safety-Verification (measures contract system):**
```
contract-intensive functions, invariant-heavy types, borrow stress tests
→ Competes on: bugs caught at compile time vs runtime
→ AXIOM advantage: UNIQUE — no other language has first-class contracts
```

### Competitive Benchmark Framework (`xiom bench`)

A dedicated tool that produces standardized, reproducible benchmarks:

```
$ xiom bench --compare rust,zig,cpp --category compute
┌─────────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Benchmark           │ AXIOM    │ Rust     │ Zig      │ C++      │
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
| 3 — Toolchain | — | Z3 static verification | — | — | Visual benchmark tool, safety scoring |
| 4 — Self-Hosting | Byte-for-byte IR parity with Rust compiler | Proves correctness | — | — | Ultimate correctness test |
| 5 — Production Pillars | inkwell API (10-50× codegen speedup) | Stdlib with contracts on every function | Contract stripping in release | Build system caching | Full benchmark suite |

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

**Every module function should have contracts.** AXIOM's killer feature — contracts — must be demonstrated in the stdlib itself. `fn read_file(path: Str) -> Result[Vec[UInt8], IOError] requires: path.len() > 0 ensures: result.is_ok() => result.unwrap().len() > 0`.

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

**Current state:** `axiomc` accepts files via CLI. No build configuration. No profiles. No build scripts.

**What production needs:**

1. **`xiom build`** — reads `package.xi`, resolves deps, compiles all source files, links binary
2. **Profiles:** `dev` (fast compile, no optimize), `release` (optimize + strip contracts), `bench` (optimize + instrumentation)
3. **Build scripts** (`build.xi`) — AXIOM code that runs at build time for code generation, FFI binding generation, asset processing
4. **`--features`** — conditional compilation via feature flags
5. **`xiom check`** — type-check only, no codegen (fast feedback loop)
6. **`xiom clean`** — remove build artifacts
7. **Incremental builds** — recompile only changed files (Phase 1.3 from compiler plan)

**Design principle:** Cargo.toml is the gold standard. `package.xi` should be equally simple. No Makefiles. No CMake. No build.rs complexity — just a declarative manifest and an optional build script.

**Effort:** 1-2 months. Builds on incremental compilation. Mostly CLI orchestration + file system.

### 5.4 Testing Framework

**Current state:** Rust `cargo test` runs compiler unit tests. No AXIOM-native test framework.

**What production needs:**

1. **`xiom test`** — discovers and runs all `fn test_*()` functions in `tests/` directory
2. **Contract-aware assertions:** `assert_eq!(a, b)`, `assert_contract!(fn_call)` — verifies contracts pass
3. **Test fixtures:** `setup()` / `teardown()` per test module
4. **Benchmark mode:** `xiom bench` — runs `fn bench_*()` functions N times, reports statistics
5. **Coverage:** `xiom test --coverage` — contract coverage (which contracts are exercised by tests)
6. **Property-based testing:** `xiom test --fuzz` — random input generation with contract validation

**Contract coverage is AXIOM's unique testing feature:**
```
Contract coverage: 87%
  ✓ bench_math.ax:add              requires: a + b doesn't overflow
  ✓ bench_math.ax:divide           requires: b != 0.0
  ✗ bench_math.ax:sqrt_newton      ensures: result * result ≈ x
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

**What production needs (AXIOM's 4-point error philosophy from the spec):**

| Component | Current | Target |
|-----------|---------|--------|
| **LOCATION** | Line:col ✓ | Point to EXACT token |
| **CAUSE** | "cannot call 'len' on this expression" — vague | "`Str` has no method `len`. Use `axiom::string::str_len(s: Str) -> Int` instead." |
| **IMPLICATION** | None | "Without this, the compiler cannot verify the return type." |
| **SUGGESTION** | None | "help: add `use axiom.string;` and call `string.str_len(name)`" |

**Examples from production compilers to match:**

```
// GOOD (Rust-level):
error[E0599]: no method named `len` found for type `Str`
  --> src/main.xi:76:3
   |
76 |   name.len()
   |       ^^^ method not found in `Str`
   |
   = help: `Str` is a UTF-8 slice. Use `axiom::string::str_len(s: Str) -> Int`.
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
| **Build system** (`xiom build`) | SOON | Needed before any external user tries AXIOM |
| **Package manager** (`xiom install`) | SOON | Needed for ecosystem |
| **Testing framework** (`xiom test`) | SOON | Needed for reliability |
| **Documentation** (`xiom doc`) | SOON | Needed for adoption |
| **Compiler P2** (hot reload, multithreaded) | LATER | Advanced features |
| **LLVM API (inkwell)** | LATER | Performance optimization |
| **Compiler P3** (debugger, LSP, Z3) | FUTURE | Toolchain maturity |
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

**When all 13 items are ✓, AXIOM is production-ready.** The compiler internals are ~40% of the work. The remaining 60% is everything around the compiler.
