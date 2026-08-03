# XIOM — OS-Level Threading & Parallel Compilation

**Version:** v0.56 (Post-Implementation Audit)
**Date:** 2026-08-03
**Status:** Domain B COMPLETE (language threading). Domain A NEAR COMPLETE (compiler parallelism: parallel codegen DONE).
**E2E: 24/24 passing | Selfhost Gate: 17/17 CLEARED**
**Principle:** _Near-zero runtime errors — thread safety is a compile-time guarantee, not a runtime prayer._

---

## 0. SAFETY ARCHITECTURE — "FEARLESS CONCURRENCY" ✅

### 0.1 What's Implemented (Domain B — Language Threading)

| Feature | Status | Implementation |
|---------|--------|---------------|
| **spawn codegen** | ✅ DONE | Body compiled as separate LLVM function `@_xiom_spawn_N`. Called via `@xiom_thread_spawn(ptr @fn, ptr null)`. Module-level declare. |
| **Send/Sync markers** | ✅ DONE | `interface Send { }` / `interface Sync { }` registered in checker. Auto-derived by compiler from field composition. All primitives are Send+Sync. |
| **Channel[T]** | ✅ DONE | Bounded MPSC ring buffer (64 slots) in C runtime. Mutex + condition variable. `create/send/recv/try_recv/close`. |
| **Thread pool** | ✅ DONE | Work-stealing worker threads in C runtime. `xiom_threadpool_init(N)`, `xiom_threadpool_spawn(fn, arg)`. Auto-scales to CPU count. |
| **C runtime threading** | ✅ DONE | `xiom_thread_create/join/detach`, `xiom_mutex_*`, `xiom_cond_*` on Windows + Linux. 256-slot spawn tracking table. |
| **Spawn wrapper functions** | ✅ DONE | Body emitted as separate function via save/restore of `self.output`. Flushed via `deferred_closure_defs`. |

### 0.2 What's Still Missing (Honest Gaps — v0.56.0-pre)

| Gap | Priority | Effort | Description |
|-----|----------|--------|-------------|
| **Send/Sync enforcement** | HIGH | 5 days | Markers are declared but NOT enforced. Checker doesn't verify spawn captures satisfy Send. |
| **Channel element sizing** | HIGH | 2 days | Current channel stores i64 values. Need generic element size support (T can be any type). |
| **Deadlock detection** | MEDIUM | 4 days | Static lock-ordering analysis for Mutex chains. Best-effort compile warning. |

### 0.3 What Was Fixed (v0.56.0-pre)

| Gap | Status | Implementation |
|-----|--------|---------------|
| **Move semantics for spawn** | ✅ DONE | `spawn move { ... }` — capture analysis via `collect_expr_references_block()`, heap env struct forwarding (flat i64 array at offset*8), move-after-spawn prevention (scope removal). |
| **Thread-local recursion counter** | ✅ DONE | `@xiom_recursion_counter = internal thread_local global i64 0` since emitter.rs inception. |
| **Spawn capture layout** | ✅ DONE | Captured variables packed as i64 sequence in malloc'd env buffer. Spawn function unpacks via getelementptr+load. |
| **Parallel codegen** | ✅ DONE | `--parallel-codegen` flag. Rayon-based per-function IR emission with output merging in declaration order. |

---

## 1. DOMAIN A — COMPILER-LEVEL PARALLELISM (PARTIAL)

### 1.1 Status

| Phase | What | Status |
|-------|------|--------|
| **A1: Parallel Parse** | Parse files concurrently | ✅ DONE — `--parallel` flag, `rayon::par_iter()` |
| **A2: Dependency Graph** | Build import graph, topological sort | ✅ EXISTS — `xiom-graph` crate |
| **A3: Parallel Check** | Type-check independent modules concurrently | ❌ NOT DONE |
| **A4: Parallel Codegen** | Emit IR for independent functions concurrently | ❌ NOT DONE |
| **A5: Thread-safe Registry** | Make type/function registries concurrent | ✅ DONE — `SyncRegistry` (Arc<RwLock<HashMap>>) |

### 1.2 Remaining Work

| Task | Effort | Approach |
|------|--------|----------|
| Parallel check | 5 days | Split checker state: read-only type registry (SyncRegistry) + per-file check state. Check independent files in parallel. |
| Parallel codegen | 3 days | Split IrEmitter: shared types/config + per-function output buffer. Compile functions in parallel, concatenate outputs. |
| Thread-local recursion | 1 day | `@xiom_recursion_counter = thread_local global i64 0` |

---

## 2. DOMAIN B — LANGUAGE-LEVEL THREADING ✅ (COMPLETE)

### 2.1 Implemented

```xiom
// ── CORRECT: spawn with OS thread ──
spawn {
    heavy_computation();  // body in separate LLVM function
}
// Generated: call i64 @xiom_thread_spawn(ptr @_xiom_spawn_0, ptr null)

// ── CORRECT: Channel message passing ──
var ch = Channel[Int].new(64);
ch.send(42);
var val = ch.recv();

// ── CORRECT: Thread pool ──
xiom_threadpool_init(0);  // auto-detect CPU count
xiom_threadpool_spawn(my_fn, my_arg);
```

### 2.2 Architecture

```
xiom source
  │
  ├── spawn { body }
  │     │
  │     ├── Codegen: compile body as @_xiom_spawn_N function
  │     ├── Emit: call i64 @xiom_thread_spawn(ptr @_xiom_spawn_N, ptr null)
  │     └── Flush: spawn IR emitted after parent function via deferred_closure_defs
  │
  ├── Channel[T]
  │     │
  │     ├── C runtime: xiom_channel_t (ring buffer + mutex + condvar)
  │     ├── 64 slots, i64 values (need generic size for arbitrary T)
  │     └── Functions: create/send/recv/try_recv/close
  │
  └── Thread pool
        │
        ├── C runtime: work-stealing worker threads
        ├── Round-robin distribution
        └── Queue-full → dedicated thread fallback
```

---

## 3. SELFHOST IMPLICATIONS (v0.56.0-pre)

The self-host compiler (XIOM written in XIOM) needs:

| Requirement | Status |
|-------------|--------|
| Thread-safe type registry | ✅ SyncRegistry |
| Spawn for parallel compilation | ✅ spawn codegen |
| Channel for pipeline communication | ✅ Channel[T] |
| Thread pool for work distribution | ✅ Thread pool |
| Move semantics (prevent use-after-move in spawn) | ✅ DONE — capture forwarding via env struct |
| Thread-local state (recursion counter) | ✅ DONE — `thread_local` |
| Parallel codegen | ✅ DONE — `--parallel-codegen` with rayon |
| Send/Sync enforcement (prevent data races) | ❌ Remaining (Phase B) |

---

## 4. PERFORMANCE TARGETS (POST-IMPLEMENTATION)

| Metric | Actual (sequential) | Notes |
|--------|--------------------|-------|
| 1 file, 100 lines | ~200ms | — |
| 100 files, 10K lines | ~45s | Parse is parallel; check/codegen are sequential |
| Target with parallel check + codegen | ~15s | 3x speedup projected |
| Cache hit (unchanged file) | 5ms | Binary cache works |

---

**Status:** Domain B COMPLETE. Domain A needs parallel check + codegen. 3 critical gaps + 5 important gaps remain. Selfhost path is clear.
