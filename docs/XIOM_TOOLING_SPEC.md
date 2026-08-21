# XIOM -- Tooling Specification

**Version:** 1.0
**Date:** 2026-07-04
**Status:** Specification -- phased implementation
**Companion to:** `docs/COMPILER_IMPROVEMENT_PLAN.md`, `docs/XIOM_DISTRIBUTION_SPEC.md`

> The tooling you build will be one of the strongest ways to showcase XIOM's excellence and differentiate it from Rust, Zig, and C++.

---

## 1. Priority Matrix

| # | Tool | Impact | Difficulty | Phase |
|---|------|--------|-----------|-------|
| 1 | **XIOM Debugger** (`xiom-dbg`) | Highest -- shows contracts + ownership in action | Medium-High | 3 |
| 2 | **Visual Benchmark Suite** (`xiom-bench`) | High -- proves performance + safety simultaneously | Medium | 3 |
| 3 | **Language Server** (`xiom-lsp`) + AI Mode | High -- makes AI coding experience exceptional | Medium | 3 |
| 4 | **Hot Reload / Live Coding** | High -- huge for game engines & robotics | Medium-High | 2 |
| 5 | **Contract Coverage Analyzer** | Medium -- unique metric no other language has | Low-Medium | 3 |
| 6 | **Web Playground** (WASM) | Medium -- instant try-before-install | Medium | 3 |
| 7 | **Formal Verification Dashboard** | Medium -- shows which contracts are statically proven | Medium | 3 |

---

## 2. XIOM Debugger (`xiom-dbg`)

**Priority:** #1 -- This is where XIOM can shine brightest. A contract-aware debugger exists nowhere else.

### 2.1 Architecture

```
+-------------+     DAP      +--------------+     PTrace/Debug API    +--------------+
|  VS Code /  | <------------> |  xiom-dbg    | <-----------------------> |  XIOM Binary |
|  JetBrains  |   Protocol   |  (DAP Server) |   Platform Debug API   |  (DWARF/PDB) |
`-------------+              `--------------+                        `--------------+
```

**Based on:** Debug Adapter Protocol (DAP) -- works with VS Code, JetBrains, Neovim, any DAP-compatible editor.
**Implementation:** Rust binary. Communicates with editor via stdio. Controls target process via platform debug APIs (ptrace on Linux, WinDbg API on Windows, lldb on macOS).

### 2.2 Core Features

#### Standard Debugging
- Step in / step over / step out
- Breakpoints (conditional, data, function)
- Call stack with source locations
- Local variable inspection with type information
- Memory view with hex + struct interpretation
- Register view
- Disassembly view (LLVM IR + native)

#### Contract-Aware Debugging (Unique to XIOM)

**Contract Check Visualization:**
```
+-------------------------------------------------------------+
|  fn divide(a: Float64, b: Float64) -> Float64               |
|    requires: b != 0.0                      <- [OK] PASS (b=5.0) |
|    ensures:  result * b == a               <- [WIP] EVALUATING   |
|  {                                                          |
| ->  return a / b;      // result = 2.0                      |
|  }                                                          |
|  // ensures check: 2.0 * 5.0 == 10.0  ->  [OK] PASS            |
`-------------------------------------------------------------+
```

**Contract Violation Debugger:**
```
+-------------------------------------------------------------+
|  CONTRACT VIOLATION -- divide.xi:3                            |
|                                                              |
|  ensures: result * b == a                                    |
|           -----------------                                  |
|  Expected: 10.0 * 0.0 == 5.0                                 |
|  Actual:   10.0 * 0.0 == 0.0    <- VIOLATION                 |
|                                                              |
|  Reason: b was set to 0.0 at main.xi:42                     |
|  Suggestion: Add `requires: b != 0.0` or check before call  |
|                                                              |
|  Call stack:                                                 |
|    main() at main.xi:42  ->  let x = divide(5.0, 0.0)        |
|    divide() at divide.xi:5                                   |
`-------------------------------------------------------------+
```

#### Ownership & Borrow Visualization

**Variable Lifetime Panel:**
```
+-----------------------------------------+
|  OWNERSHIP VIEW -- main()                |
|                                         |
|  var v = Vec.new()     ############  OWNED
|  read(&v)              ####........  BORROWED (read)
|  write(&mut v)         ....####....  BORROWED (mut)
|  consume(v)            ........##..  MOVED
|  // v no longer valid  ..........##  INVALID
|                                         |
|  Legend: # = valid, . = expired         |
`-----------------------------------------+
```

**Borrow Graph:**
```
+--------------------------------------+
|  BORROW GRAPH                        |
|                                      |
|  data --&---> read_only()             |
|    |                                  |
|    `--&mut---> mutate()  (exclusive)  |
|                                      |
|  Active borrows: 1 read, 1 write     |
|  [WARN] Write borrow is exclusive         |
`--------------------------------------+
```

### 2.3 Implementation Phases

| Phase | Features | Effort |
|-------|----------|--------|
| **Phase 3a -- Basic** | Step, breakpoints, call stack, locals, DWARF emission | 3-4 weeks |
| **Phase 3b -- Contract** | Contract check visualization, violation debugger, pre/post state inspection | 3-4 weeks |
| **Phase 3c -- Ownership** | Variable lifetime panel, borrow graph, move tracking | 2-3 weeks |
| **Phase 3d -- Advanced** | Conditional breakpoints on contracts, memory visualization, reverse debugging | 2-3 weeks |

---

## 3. Visual Benchmark Tool (`xiom-bench`)

**Priority:** #2 -- Proves performance AND safety simultaneously. Your marketing weapon.

### 3.1 Architecture

```
xiom-bench run --compare rust,zig,cpp --category all
  -> compiles each benchmark in each language
  -> runs N iterations
  -> collects: time, memory, binary size, safety metrics
  -> outputs: JSON + terminal table + HTML dashboard
```

### 3.2 Benchmark Categories

| Category | Benchmarks | Measures |
|----------|-----------|----------|
| **Compute** | matmul, n-body, mandelbrot, prime sieve, FFT | Raw CPU throughput |
| **Memory** | binary tree, b-tree, graph traversal, Vec stress | Allocation speed + safety |
| **I/O** | file read/write, JSON parse, HTTP server, CSV | I/O throughput + stdlib quality |
| **Safety** | contract stress, borrow torture, invariants | Bugs caught at compile time |
| **Compile** | 1K/10K/100K lines, 10/50/500 fns | Compile time scaling |

### 3.3 Output Formats

**Terminal Table:**
```
+---------------------+----------+----------+----------+----------+
| Benchmark           | XIOM     | Rust     | Zig      | C++      |
|---------------------+----------+----------+----------+----------|
| matmul 1024x1024    | 2.18s    | 2.12s    | 2.05s    | 1.98s    |
| n-body 10M steps    | 4.42s    | 4.51s    | 4.38s    | 4.15s    |
| prime sieve 100M    | 0.89s    | 0.91s    | 0.87s    | 0.82s    |
|---------------------+----------+----------+----------+----------|
| [BOLT] Safety Score      | 100%     | 100%     | 45%      | 0%       |
| [SCROLL] Contract Coverage | 87%      | N/A      | N/A      | N/A      |
| [PKG] Binary Size       | 1.8MB    | 1.6MB    | 1.2MB    | 0.9MB    |
`---------------------+----------+----------+----------+----------+
```

**HTML Dashboard:** `bench.xiom-lang.org` -- interactive charts, live comparison against latest Rust/Zig/C++.

**Radar Chart:**
```
              Speed
                ^
               /|\
              / | \
             /  |  \
      Safety -------- Binary Size
             \  |  /
              \ | /
               \|/
                v
           Compile Time
```

### 3.4 CI Integration

Every commit runs `xiom-bench` and posts results. The dashboard shows historical trend lines -- "XIOM is getting 2% faster per release."

---

## 4. Language Server (`xiom-lsp`) + AI Mode

**Priority:** #3 -- Makes the AI coding experience exceptional.

### 4.1 Core Features

| Feature | Description | Phase |
|---------|-------------|-------|
| Syntax highlighting | Token-level coloring | Now |
| Diagnostics | Error/warning display with suggestions | Now |
| Go-to-definition | Jump to type/function declaration | Now |
| Hover | Type info, contracts, documentation | Phase 3a |
| Autocomplete | Context-aware completion | Phase 3a |
| Find references | Where is this used? | Phase 3a |
| Rename | Safe rename across files | Phase 3a |

### 4.2 Contract Intelligence

| Feature | Description |
|---------|-------------|
| **Contract Lens** | Inline display of `requires`/`ensures` above function signatures |
| **Contract Coverage** | Highlight which contracts are exercised by tests |
| **"Add Contract" code action** | Suggest missing pre/post conditions based on body analysis |
| **Contract navigation** | Jump from call site `requires` to callee `ensures` |

### 4.3 Ownership Analysis

| Feature | Description |
|---------|-------------|
| **Borrow underlines** | Green = owned, blue = borrowed, red = moved |
| **Move explanation** | Hover on moved variable -> "Why was this moved?" with source location |
| **Borrow conflict highlight** | Red underline on conflicting borrows |
| **Clone suggestion** | "Consider cloning before moving" |

### 4.4 AI Co-Pilot Mode (`--ai`)

Special comment commands that invoke LLM assistance:

| Command | What It Does |
|---------|-------------|
| `@explain` | "Why does this compile error happen? Explain in detail." |
| `@fix` | "Suggest a fix for this error." |
| `@add-contract` | "Suggest contracts for this function based on its body." |
| `@add-test` | "Generate test cases that exercise these contracts." |
| `@strengthen-invariant` | "This invariant is too weak. Suggest a stronger one." |
| `@fix-borrow` | "This borrow conflict -- suggest a refactor (clone, restructure)." |
| `@optimize` | "This function is a hot path. Suggest performance improvements." |

---

## 5. Hot Reload / Live Coding

**Priority:** #4 -- Huge for game engines and robotics. See Phase 2.1 of the improvement plan.

### Architecture

```
Source Change -> File Watcher -> Recompile Module -> .dll/.so -> LoadLibrary -> Patch Function Table
```

### Use Cases

| Use Case | Benefit |
|----------|---------|
| **Game engines** | Tweak game logic without restarting |
| **Robotics** | Update control algorithms mid-operation |
| **GUI development** | Live-reload UI code |
| **Scientific computing** | Adjust simulation parameters live |

---

## 6. Contract Coverage Analyzer

Unique metric -- no other language has this.

```
$ xiom test --coverage

Contract Coverage Report
========================
  Overall: 87% (234/268 contracts exercised)

  [OK] bench_math.xi:add             requires: a + b doesn't overflow
  [OK] bench_math.xi:divide          requires: b != 0.0
  [OK] bench_math.xi:divide          ensures:  result * b == a
  [FAIL] bench_math.xi:sqrt_newton     ensures: result * result ~= x
    ^ Never triggered by any test. Add a test that calls sqrt_newton.

  [CHART] By module:
  bench_math.xi           92% (23/25)
  bench_primes.xi         88% (15/17)
  bench_ownership.xi      95% (19/20)
```

---

## 7. Web Playground (WASM)

**URL:** `play.xiom-lang.org`

The XIOM compiler compiles to WASM. A static HTML page loads the compiler and provides:
- Code editor (Monaco/VSCode editor in browser)
- "Compile & Run" button
- LLVM IR output panel
- Contract check visualization
- No backend. No account. No install.

---

## 8. Formal Verification Dashboard

Phase 3+ -- shows which contracts are statically proven vs runtime-guarded.

```
+--------------------------------------------------+
|  VERIFICATION DASHBOARD                          |
|                                                  |
|  Statically proven:  142 / 268 (53%)  ######.... |
|  Runtime guards:     126 / 268 (47%)  ######.... |
|  Unverified:           0              .......... |
|                                                  |
|  Z3 Time:  12.4s (avg 0.09s per function)        |
|  Timeouts: 3 functions (increase timeout?)       |
`--------------------------------------------------+
```

---

*XIOM Tooling Specification -- Version 1.0.*
*Cross-referenced with COMPILER_IMPROVEMENT_PLAN.md Phase 3.*
