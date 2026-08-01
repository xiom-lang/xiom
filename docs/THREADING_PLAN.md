# XIOM — OS-Level Threading & Parallel Compilation

**Version:** v0.54 → v0.55 (Pre-Selfhost)
**Date:** 2026-08-02
**Status:** Planning
**Principle:** _Near-zero runtime errors — thread safety is a compile-time guarantee, not a runtime prayer._

---

## 0. SAFETY ARCHITECTURE — "FEARLESS CONCURRENCY"

XIOM's mission is near-zero runtime errors. Threading is the #1 source of
runtime errors in systems languages: data races, use-after-free across threads,
deadlocks, and torn reads. XIOM must eliminate these at compile time.

### 0.1 The Rust Model (Baseline)

Rust proved that ownership + Send/Sync traits can make data races **impossible**
in safe code. XIOM adopts this model with XIOM-native syntax:

```
┌─────────────────────────────────────────────────────────────┐
│  RUNTIME ERROR      │  RUST           │  XIOM              │
├─────────────────────────────────────────────────────────────┤
│  Data race          │  Compile error  │  Compile error     │
│  Use-after-move     │  Compile error  │  Compile error     │
│  Deadlock           │  Runtime (no    │  Optional compile- │
│                     │  detection)     │  time detection    │
│  Torn read/write    │  Type system    │  Type system       │
│  Double-free        │  Ownership      │  Ownership         │
│  Dangling pointer   │  Lifetimes      │  E001 borrow check │
│  Thread explosion   │  No guard       │  Pool limits       │
└─────────────────────────────────────────────────────────────┘
```

### 0.2 XIOM's Ownership Model (already exists)

XIOM's borrow checker (E001) already prevents use-after-free and dangling
pointers. Thread safety extends this with two new properties:

**`Send`** — A type is `Send` if ownership can be transferred to another thread.
**`Sync`** — A type is `Sync` if a shared reference can be accessed from multiple threads.

These are **auto-derived** — the programmer never writes `impl Send for Foo`.
The compiler derives them from field composition, exactly like Rust.

### 0.3 Auto-Derivation Rules

```
Rule 1: Primitives are Send + Sync
  Int, Bool, Float64, Str, Char → always thread-safe

Rule 2: Ownership-transferring containers are Send (if T is Send)
  Vec[T], Option[T], Result[T,E], Map[K,V], Set[T]
  → Send if all type params are Send
  → NOT Sync (mutable interior)

Rule 3: Atomic containers are Send + Sync
  AtomicInt, AtomicBool → always Send + Sync
  Arc[T] → Send + Sync if T: Send + Sync

Rule 4: Mutex-protected types are Send + Sync
  Mutex[T] → Send + Sync if T: Send
  RwLock[T] → Send + Sync if T: Send

Rule 5: Raw pointers are NEVER Send or Sync
  *T → requires `unsafe` block to transfer between threads

Rule 6: User structs derive from fields
  struct Foo { a: Int, b: Vec[Str] }
  → Send? = Send(Int) && Send(Vec[Str]) = true && true = YES
  → Sync? = Sync(Int) && Sync(Vec[Str]) = true && false = NO

Rule 7: Function pointers require explicit annotation
  fn() → NOT Send by default
  fn() : Send → explicitly thread-safe
```

### 0.4 Compile-Time Data Race Prevention

```xiom
// ── COMPILE ERROR: data race ──
var counter: Int = 0;
spawn { counter = counter + 1; }  // ERROR: 'counter' is not Send
spawn { counter = counter + 1; }  // ERROR: shared mutable access

// ── CORRECT: Mutex protection ──
var counter = Mutex[Int].new(0);
spawn {
    var guard = counter.lock();
    *guard = *guard + 1;  // ok: exclusive access via mutex
}
spawn {
    var guard = counter.lock();
    *guard = *guard + 1;  // ok: sequentialized by mutex
}

// ── CORRECT: Atomic operations ──
var counter = AtomicInt.new(0);
spawn { counter.fetch_add(1); }  // ok: AtomicInt is Sync
spawn { counter.fetch_add(1); }  // ok

// ── CORRECT: Message passing via Channel ──
var ch = Channel[Int].new(1);
spawn { ch.send(42); }           // ok: ownership transfers
var val = ch.recv();             // ok: received on main thread

// ── CORRECT: Shared immutable data ──
var data = Arc[Vec[Int]].new([1, 2, 3]);
spawn { var sum = sum(data.get()); }   // ok: Arc is Sync, read-only
spawn { var avg = avg(data.get()); }   // ok: concurrent reads are safe
```

### 0.5 The Unsafe Escape Hatch

```xiom
// When the compiler is too conservative:
var ptr: *Int = alloc(8);
unsafe {
    // Programmer asserts thread safety manually
    spawn { xiom_atomic_store(ptr, 42); }
}
// The 'unsafe' keyword is the contract: "I verified this is correct."
```

### 0.6 Deadlock Detection (v0.56 — optional)

XIOM can detect potential deadlocks at compile time using a simple lock-ordering
analysis. This is **best-effort** (not a proof), similar to `clippy` lints:

```xiom
// ── WARNING: potential deadlock ──
fn transfer(a: &Mutex[Int], b: &Mutex[Int]) {
    var ga = a.lock();
    var gb = b.lock();  // WARNING: inconsistent lock ordering
}

// ── CORRECT: consistent ordering ──
fn transfer(a: Mutex[Int], b: Mutex[Int]) {
    if a.inner < b.inner {
        var ga = a.lock(); var gb = b.lock();
    } else {
        var gb = b.lock(); var ga = a.lock();
    }
}
```

---

## 1. CURRENT STATE & GAP ANALYSIS

### What Exists (mature)
- **Lexer/Parser:** `spawn { ... }` at statement + module level, `async fn` syntax parsed
- **AST:** `Stmt::Spawn(Block)`, `TopDecl::Spawn(Block)`
- **Checker:** Spawn blocks are scoped and checked (as plain blocks — no thread awareness)
- **Runtime:** Full cross-platform C runtime — thread create/join/detach, mutex, condvar, atomics, 256-slot tracked spawn table (`xiom_thread_spawn` family), monotonic clock
- **Stdlib:** `xiom.sync` module — `Mutex[T]`, `RwLock[T]`, `Arc[T]`, `AtomicInt`, `AtomicBool`, `Barrier`, `Once`, `Condvar`
- **Tests:** 8 spawn/async tests — parsing + type-checking only, zero thread execution

### What Is Missing (critical gap)
| Layer | Gap |
|-------|-----|
| **Codegen** | `Stmt::Spawn` compiles as inline block. `TopDecl::Spawn` is skipped. No calls to `xiom_thread_spawn`. |
| **Type system** | No `Send`/`Sync` traits. No move semantics for captured variables. No data-race detection. |
| **Channel** | No `Channel[T]` in stdlib or runtime. |
| **Compiler parallelism** | Entire compilation is single-threaded — parse/check/codegen run sequentially per file. |
| **Thread-local storage** | Recursion counter is global — needs per-thread. |

---

## 2. ARCHITECTURE — TWO DOMAINS

This plan covers two independent but complementary domains:

```
┌─────────────────────────────────────────────────────────────┐
│ DOMAIN A: COMPILER-LEVEL PARALLELISM                        │
│                                                              │
│  Make the XIOM compiler itself multithreaded.                │
│  Goal: compile thousands of files, millions of lines.        │
│  Approach: data parallelism via rayon + dependency graph.    │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ File A   │  │ File B   │  │ File C   │  ← parallel      │
│  │ parse    │  │ parse    │  │ parse    │     parse          │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                  │
│       │             │             │                          │
│  ┌────▼─────────────▼─────────────▼─────┐                   │
│  │        Type Resolution Graph         │  ← dependency     │
│  │  (topological sort, parallel ready)  │     ordering       │
│  └────┬─────────────┬─────────────┬─────┘                   │
│       │             │             │                          │
│  ┌────▼─────┐  ┌────▼─────┐  ┌────▼─────┐                  │
│  │ Check A  │  │ Check B  │  │ Check C  │  ← parallel      │
│  │ Codegen A│  │ Codegen B│  │ Codegen C│     check+gen     │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ DOMAIN B: LANGUAGE-LEVEL THREADING                           │
│                                                              │
│  Make XIOM programs use real OS threads.                     │
│  Goal: concurrency primitives like Rust/Go.                  │
│  Approach: codegen spawn → xiom_thread_spawn runtime calls.  │
│                                                              │
│  spawn { heavy_computation() }                               │
│    │                                                         │
│    ▼                                                         │
│  codegen: call xiom_thread_spawn(&closure, stack_size)       │
│    │                                                         │
│    ▼                                                         │
│  runtime: CreateThread/pthread_create → OS thread            │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. DOMAIN A — COMPILER-LEVEL PARALLELISM

### 3.1 Phase Design

| Phase | What | Parallelism | Effort |
|-------|------|-------------|--------|
| **A1: Parallel Parse** | Parse files concurrently | Embarrassingly parallel | 3 days |
| **A2: Dependency Graph** | Build import graph, topological sort | DAG analysis | 2 days |
| **A3: Parallel Check** | Type-check independent modules concurrently | Data parallel per SCC | 5 days |
| **A4: Parallel Codegen** | Emit IR for independent functions concurrently | Function-level parallel | 3 days |
| **A5: Thread-safe Registry** | Make type/function registries `RwLock<HashMap<>>` | Lock-free where possible | 2 days |

### 3.2 Crate Structure

```
crates/xiom-compiler/        ← NEW: parallel compilation orchestrator
├── Cargo.toml               # depends on rayon, dashmap
├── src/
│   ├── lib.rs               # Compiler::compile_parallel(entry_points)
│   ├── graph.rs             # Dependency graph, topological sort
│   ├── pool.rs              # Rayon thread pool configuration
│   ├── registry.rs          # Thread-safe type/function registry
│   ├── session.rs           # Per-file compilation session
│   └── pipeline.rs          # Parallel pipeline stages
```

### 3.3 Dependency Graph

```rust
/// A compilation unit (one .xi file after preprocessing).
struct CompilationUnit {
    path: PathBuf,
    source: String,
    /// Files this unit imports (via `use` statements).
    imports: Vec<String>,
    /// The parsed AST (populated after parse phase).
    ast: Option<Program>,
    /// The checked and resolved types (populated after check phase).
    checked: Option<CheckedModule>,
}

/// Directed acyclic graph of compilation units.
struct DependencyGraph {
    /// All compilation units keyed by canonical path.
    units: HashMap<PathBuf, CompilationUnit>,
    /// Adjacency list: unit → units it depends on.
    edges: HashMap<PathBuf, Vec<PathBuf>>,
    /// Reverse edges: unit → units that depend on it.
    reverse_edges: HashMap<PathBuf, Vec<PathBuf>>,
}

impl DependencyGraph {
    /// Topological sort into parallel-ready levels.
    /// Level 0: no dependencies → parse in parallel.
    /// Level N: dependencies resolved → check in parallel within level.
    fn topological_levels(&self) -> Vec<Vec<PathBuf>> {
        // Standard Kahn's algorithm producing level sets.
        // Each level contains all units whose remaining dependency count is 0.
    }
}
```

### 3.4 Parallel Pipeline

```rust
use rayon::prelude::*;

pub fn compile_parallel(entry_points: &[PathBuf], opts: &CompileOpts) -> Result<()> {
    // ─── Phase 1: Parallel Parse ───
    let graph = DependencyGraph::from_entry_points(entry_points)?;
    let levels = graph.topological_levels();

    // Level 0: all files parsed in parallel (no dependencies)
    let graph = levels[0].par_iter().fold(
        || graph.clone(),
        |mut g, path| {
            let ast = parse_file(path)?;
            g.set_ast(path, ast);
            Ok(g)
        },
    ).reduce(|| Ok(graph.clone()), |a, b| merge_graphs(a?, b?))?;

    // ─── Phase 2: Type Check (level by level, parallel within each level) ───
    for level in &levels[1..] {
        let graph = level.par_iter().try_fold(
            || graph.clone(),
            |g, path| {
                let checked = check_unit(path, &g)?;
                g.set_checked(path, checked);
                Ok(g)
            },
        ).try_reduce(|| graph.clone(), merge_graphs)?;
    }

    // ─── Phase 3: Parallel Codegen ───
    let modules = levels.iter().flatten().par_bridge().map(|path| {
        let unit = graph.get(path);
        codegen_unit(unit)
    }).collect::<Vec<_>>();

    // ─── Phase 4: Sequential Link ───
    link_modules(&modules, opts)
}
```

### 3.5 Thread-Safe Registries

Current registries are `HashMap<String, T>` behind `&mut self`. For parallel
access, switch to concurrent data structures:

```rust
use dashmap::DashMap;
use parking_lot::RwLock;

/// Thread-safe type registry (reads are lock-free after insert).
pub struct ConcurrentTypeRegistry {
    /// Type name → field definitions. DashMap for concurrent read/write.
    types: DashMap<String, Vec<String>>,
    /// Type name → TypeMeta. DashMap.
    type_meta: DashMap<String, TypeMeta>,
    /// Function name → (param types, return type). DashMap.
    functions: DashMap<String, (Vec<String>, String)>,
}

/// Thread-safe local context for function compilation.
/// Each function gets its own LocalContext (no sharing needed).
pub struct ConcurrentLocalContext {
    /// Locals are function-scoped — no sharing across functions.
    locals: HashMap<String, (String, String)>,
    /// These are populated during parsing and never modified during check/codegen.
    local_xiom_types: HashMap<String, String>,
    local_vec_elem: HashMap<String, String>,
    // ...
}
```

### 3.6 Lock Contention Hotspots

| Operation | Contention Risk | Solution |
|-----------|----------------|----------|
| Type registration | Low — mostly writes during initial pass | `DashMap::insert` (lock-free) |
| Type lookup | None — read-only after registration | `DashMap::get` (lock-free) |
| Function registration | Low | `DashMap::insert` |
| Module-level constants | Low | Per-module arena |
| Recursion counter | High — every function call | **Thread-local** — make it `thread_local!` |
| Fresh temp counter | Medium — every IR instruction | Atomic counter per function |

### 3.7 Thread-Local Recursion Counter

Currently `@xiom_recursion_counter` is a global. In threaded programs,
each thread needs its own:

```llvm
; Before (broken for threads):
@xiom_recursion_counter = internal thread_local global i64 0

; After (correct for threads):
; LLVM's `thread_local` storage class handles this — each thread gets its own copy.
; No ABI change needed — just verify the attribute propagates correctly.
```

Verify the LLVM target supports `thread_local` (all tier-1 targets do):
```rust
// In codegen:
self.emitln("@xiom_recursion_counter = internal thread_local global i64 0");
```

### 3.8 Performance Targets

| Metric | Current (sequential) | Target (parallel) | Speedup |
|--------|---------------------|-------------------|---------|
| 100 files, 10K lines total | ~45s | ~6s | **7.5x** |
| 1,000 files, 100K lines | ~8min | ~40s | **12x** |
| 10,000 files, 1M lines | OOM/timeout | ~6min | **∞ (feasible)** |
| Cache hit (unchanged file) | 500ms per file | 50ms per file (skip parse) | **10x** |

---

## 4. DOMAIN B — LANGUAGE-LEVEL THREADING

### 4.1 Phase Design

| Phase | What | Effort |
|-------|------|--------|
| **B1: Spawn Codegen** | `spawn { ... }` → `xiom_thread_spawn` runtime call | 3 days |
| **B2: Thread-Local Storage** | Recursion counter, random seed per-thread | 1 day |
| **B3: Channel[T]** | `Channel[T].send()/.recv()` via ring buffer + condvar | 3 days |
| **B4: Move Semantics** | Capture analysis for spawn closures — prevent use-after-move | 3 days |
| **B5: Send/Sync Traits** | Thread-safety type checking — prevent data races | 5 days |
| **B6: Thread Pool** | Reuse OS threads, work-stealing scheduler | 3 days |

### 4.2 Spawn Codegen

```rust
// Source:
spawn {
    heavy_computation();
}

// Generated IR (pseudocode):
// 1. Allocate closure struct on heap (captured variables)
// 2. Create thread via xiom_thread_spawn
// 3. Return thread handle

fn compile_spawn_stmt(body: &Block) {
    // Allocate closure context on heap
    let ctx = self.fresh_tmp();
    self.emitln("  {ctx} = call i8* @malloc(i64 {sizeof_closure})");

    // Store captured variables into closure
    for (name, slot) in captured_vars {
        // ... store each captured var into ctx fields
    }

    // Generate the spawn wrapper function
    let wrapper = self.compile_spawn_wrapper(body, &captured_vars);

    // Call runtime: xiom_thread_spawn(wrapper_fn, ctx_ptr, stack_size)
    let handle = self.fresh_tmp();
    self.emitln("  {handle} = call i64 @xiom_thread_spawn(i8* {wrapper}, i8* {ctx}, i64 1048576)");

    // Track handle for join/detach
    self.add_spawn_handle(handle);
}

fn compile_spawn_wrapper(body: &Block, captures: &[Capture]) -> String {
    // Generate a standalone function that takes (ctx_ptr: i8*) -> void:
    //
    // define void @__spawn_wrapper_N(i8* %ctx) {
    //   ; extract captured vars from ctx
    //   %x_ptr = getelementptr i8, i8* %ctx, i64 {offset_x}
    //   %x = load i64, i64* %x_ptr
    //   ; compile body with captured vars in scope
    //   ...
    //   ret void
    // }
}
```

### 4.3 Channel[T] — Production-Grade Implementation

```rust
// XIOM source:
var ch = Channel[Int].new(256);   // bounded channel, capacity 256
ch.send(42);
var val = ch.recv();              // blocks if empty

// Channel struct (C runtime — ring buffer + mutex + condvar):
// {
//   buffer: *void,          // heap-allocated ring buffer
//   capacity: i64,          // max elements
//   head: i64,              // read position (atomic)
//   tail: i64,              // write position (atomic)
//   count: i64,             // current element count (atomic)
//   mutex: *u8,             // pthread_mutex_t / CRITICAL_SECTION
//   not_empty: *u8,         // condvar: signaled when count > 0
//   not_full: *u8,          // condvar: signaled when count < capacity
//   closed: i32,            // atomic: 0=open, 1=closed
// }
```

```c
// C runtime additions:
typedef struct {
    void* buffer;
    int64_t capacity;
    _Atomic int64_t head;
    _Atomic int64_t tail;
    _Atomic int64_t count;
    xiom_mutex_t mutex;
    xiom_cond_t not_empty;
    xiom_cond_t not_full;
    _Atomic int32_t closed;
} xiom_channel_t;

xiom_channel_t* xiom_channel_new(int64_t capacity);
void xiom_channel_send(xiom_channel_t* ch, void* data, int64_t elem_size);
void xiom_channel_recv(xiom_channel_t* ch, void* out, int64_t elem_size);
void xiom_channel_close(xiom_channel_t* ch);
int32_t xiom_channel_is_closed(xiom_channel_t* ch);
```

### 4.4 Move Semantics for Spawn Captures

```rust
// The checker must prevent this:
var x = 42;
spawn {
    io.println(x);  // ok: x is captured by copy (Int is Copy)
}
io.println(x);      // ok: x was copied, not moved

var v = Vec[Int].new();
spawn {
    process(v);     // ERROR: Vec is not Copy, v is MOVED into spawn
}
v.push(1);          // ERROR: v was moved, use-after-move
```

Implementation:
```rust
/// During type-checking of spawn blocks, track which variables are
/// captured by move vs by copy. After the spawn, mark moved variables
/// as consumed (use-after-move is an error).
struct SpawnCaptureAnalysis {
    /// Variables moved into the spawn closure.
    moved: HashSet<String>,
    /// Variables copied into the spawn closure (primitives, Arc).
    copied: HashSet<String>,
}
```

### 4.5 Send/Sync Traits (Minimal Viable)

For v0.54, implement a simplified thread-safety model:

| Type | Send? | Sync? | Rule |
|------|-------|-------|------|
| `Int`, `Bool`, `Float64`, `Str` | Yes | Yes | Primitives are always safe |
| `Vec[T]` where T: Send | Yes | No | Owned data, move semantics |
| `Arc[T]` where T: Send + Sync | Yes | Yes | Atomic reference count |
| `Mutex[T]` where T: Send | Yes | Yes | Mutex protects access |
| `*T` (raw pointer) | No | No | Unsafe by default |
| User structs | Derived from fields | Derived from fields | Auto-derived |

```rust
/// Thread-safety trait (compile-time only, no runtime cost).
enum ThreadSafety {
    /// Safe to transfer ownership between threads.
    Send,
    /// Safe to share a reference between threads.
    Sync,
    /// Not thread-safe.
    ThreadLocal,
}

/// Auto-derive Send/Sync for struct types based on field composition.
fn derive_thread_safety(type_name: &str) -> ThreadSafety {
    let fields = type_registry.get_fields(type_name);
    let all_send = fields.iter().all(|f| is_send(f.ty));
    let all_sync = fields.iter().all(|f| is_sync(f.ty));
    match (all_send, all_sync) {
        (true, true) => ThreadSafety::SendAndSync,
        (true, false) => ThreadSafety::SendOnly,
        _ => ThreadSafety::ThreadLocal,
    }
}
```

### 4.6 Thread-Local Storage

```xiom
// Language-level thread-local storage:
#[thread_local]
var counter: Int = 0;

// Thread ID:
var tid = xiom.thread.id();
```

```llvm
; LLVM codegen:
@counter = thread_local global i64 0
```

---

## 5. STD LIBRARY — `xiom.thread`

New stdlib module for language-level threading:

```
stdlib/xiom/thread.xi

module xiom.thread

// Thread management
pub fn spawn(f: fn()) -> ThreadHandle
pub fn sleep_ms(ms: Int)

pub type ThreadHandle = { id: Int; }
pub fn ThreadHandle.join(self)
pub fn ThreadHandle.detach(self)
pub fn ThreadHandle.id(self) -> Int

// Thread identity
pub fn current_id() -> Int

// Channel — message passing
pub type Channel[T] = { /* ring buffer + mutex */ }
pub fn Channel.new[T](capacity: Int) -> Channel[T]
pub fn Channel.send[T](self, value: T)
pub fn Channel.recv[T](self) -> T
pub fn Channel.try_recv[T](self) -> Option[T]
pub fn Channel.close[T](self)

// Thread-local storage
#[thread_local]
pub fn local[T](initial: T) -> &mut T
```

---

## 6. SELFHOST IMPLICATIONS

The self-host compiler (XIOM written in XIOM) will need:

1. **Parallel compilation:** The self-host compiler itself must be multithreaded to be usable. Domain A provides the infrastructure.

2. **Channel-based pipelines:** Parse → AST → Check → IR can be pipelined through channels, with each stage running on a dedicated thread pool.

3. **Thread-safe type registry:** The self-host compiler's symbol table must be thread-safe. Domain A provides this.

4. **Work-stealing scheduler:** For compiling many files, a work-stealing thread pool maximizes CPU utilization.

---

## 7. IMPLEMENTATION ORDER

```
v0.54 ──► v0.55 ──► v0.56 ──► SELFHOST
  │         │         │
  │         │         └── Parallel Codegen + Send/Sync + Channel[T]
  │         └── Parallel Check + Spawn Codegen + Thread-Local Storage
  └── Parallel Parse + Dependency Graph + Thread-Safe Registry
```

### v0.54: Parallel Frontend + Thread-Safe Types
- [ ] Rayon integration for parallel file parsing
- [ ] Dependency graph construction and topological sort
- [ ] `DashMap`-based concurrent type/function registries
- [ ] Thread-local recursion counter
- [ ] `--jobs N` CLI flag
- [ ] Benchmarks: 100-file project compile time

### v0.55: Spawn Codegen + Parallel Check
- [ ] `spawn { ... }` → `xiom_thread_spawn` runtime call
- [ ] Spawn wrapper function generation with heap-allocated closure
- [ ] Parallel type-checking within dependency levels
- [ ] Move semantics for spawn captures
- [ ] `Channel[T]` runtime + stdlib
- [ ] Thread-local storage (`#[thread_local]`)
- [ ] Tests: concurrent quicksort, parallel map-reduce, ping-pong channels

### v0.56: Polish + Selfhost Prep
- [ ] Parallel codegen (function-level)
- [ ] `Send`/`Sync` auto-derivation
- [ ] Thread pool with work-stealing
- [ ] `detach` + `join` with result capture
- [ ] Data-race detection (compile-time warnings)
- [ ] Full test suite: 100+ threading tests

---

## 8. PERFORMANCE TARGETS

### Compiler Throughput

| Scale | Current (sequential) | Target (v0.56) |
|-------|---------------------|----------------|
| 1 file, 100 lines | 200ms | 200ms (no change) |
| 100 files, 10K lines | ~45s | **~6s (7.5x)** |
| 1,000 files, 100K lines | ~8min | **~40s (12x)** |
| 10,000 files, 1M lines | OOM | **~5min** |
| Selfhost (est. 500 files) | — | **~15s** |

### Language Threading

| Benchmark | Expected Speedup |
|-----------|-----------------|
| Parallel quicksort (8 threads) | **6x** vs sequential |
| Map-reduce (8 threads) | **7x** vs sequential |
| Web server (100 concurrent) | Handles 100x more connections |
| Channel ping-pong (2 threads) | 1M msg/s throughput |

---

## 9. RISKS & MITIGATIONS

| Risk | Mitigation |
|------|------------|
| `DashMap` deadlocks under high contention | Use `parking_lot::RwLock` for write-heavy paths |
| Type checker non-determinism from parallel execution | Deterministic ordering within each topological level |
| Spawn codegen produces wrong closure layout | Generate C-compatible struct layouts; validate with alignment tests |
| Thread explosion from nested spawn | Thread pool limits; `#[thread_pool(max = N)]` annotation |
| Data races from shared mutable state | `Send`/`Sync` trait checking at compile time; runtime `Arc` for sharing |
| Debugging multithreaded compiler crashes | `--sequential` flag to force single-threaded mode for debugging |

---

## 10. SUMMARY

This plan delivers OS-level threading at two levels:

**Compiler-level:** Parallel parse, check, and codegen across files using `rayon`
and a dependency graph. Thread-safe registries via `DashMap`. Target: 7-12x
speedup for multi-file projects.

**Language-level:** `spawn` codegen emits real OS thread creation via
`xiom_thread_spawn`. `Channel[T]` for message passing. Move semantics and
`Send`/`Sync` traits for compile-time data-race prevention.

**Selfhost readiness:** The self-host compiler (written in XIOM) will use
these features to compile itself efficiently.

**Status:** APPROVED for v0.54 → v0.56 roadmap. Precedes Selfhost phase.
