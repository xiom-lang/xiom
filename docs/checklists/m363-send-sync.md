# m363 -- Send/Sync enforcement (THREADING_PLAN Phase B)

Status: QUEUED (design checklist; no fixer yet)
Owner: compiler lane (checker; THREADING_PLAN Phase B)
Sources: `docs/THREADING_PLAN.md` (Phase B, HIGH), the I1 infrastructure in
`crates/xiom-check/src/lib.rs`, m365/m369 sibling notes.

## Current state (verified in the tree)

- `interface Send {}` / `interface Sync {}` are registered; primitives are in
  `send_sync_types` (`lib.rs:1159-1170`).
- `is_send` exists (`lib.rs:3041`) and is enforced **only** for `spawn`
  captures (`lib.rs:7497-7513`).
- `is_send` is still loose:
  - structs/enums recurse over fields, but
  - **any** `Option/Result/Vec/Map/Set/Deque/HashMap/HashSet/BTreeMap/
    PriorityQueue/Channel/Arc/Atomic*/Mutex/RwLock` is assumed Send without
    checking its type arguments (the comment says "full generic analysis
    deferred"). `Vec[*UInt8]`, `Channel[FfiHandle]`, `Mutex[*UInt8]` are
    therefore accepted today.
- `is_sync` does not exist; `&T` captures are not checked at all.
- `Channel.send`, `Mutex.new`, `RwLock.new`, `Arc.new` and threadpool spawn
  arguments are not checked.
- `spawn` marks every capture as moved even for non-move spawns; the
  by-value vs reference distinction is not modelled.

## Goal

Make the markers sound at the compile boundary: a value that crosses a
thread boundary must be `Send`; a shared reference must point at `Sync`.
No runtime changes. Keep the existing positive programs working, with a
feature flag for the first wave to catch false positives before default-on.

## Plan

- [ ] Replace the "container == true" shortcut in `is_send` with a recursive
  check over `container_parts` arguments (`Vec[T]`/`Option[T]`/`Result[T,E]`
  require T/E Send; `Map[K,V]`/`Set[T]` require K/V Send; `Channel[T]`/
  `Mutex[T]`/`RwLock[T]`/`Arc[T]` require T Send; atomics stay Send).
  Struct/enum field recursion stays; unresolved/FFI/raw-pointer names stay
  NOT Send (conservative).
- [ ] Add `is_sync`: primitives true; structs/enums all fields Sync; `Vec[T]`
  Sync iff T Sync; `Mutex[T]` Sync iff T Send; `RwLock[T]` Sync iff T Send +
  Sync; `Channel[T]` Sync iff T Send; `Arc[T]` Sync iff T Send + Sync;
  pointers/FFI false.
- [ ] Model spawn captures by kind: by-value capture requires Send;
  `&T` capture requires T Sync; `&mut T` capture requires T Send (and the
  borrow must not outlive the spawn). Keep the existing move diagnostics.
- [ ] Enforce the API boundaries:
  - `Channel.send(v)` / `try_send(v)`: T Send;
  - `Mutex.new(v)` / `RwLock.new(v)`: T Send (RwLock: Send + Sync if the
    design requires shared readers);
  - `Arc.new(v)`: T Send + Sync when the value is shared;
  - `threadpool.spawn(fn, arg)` / `thread.spawn(fn, arg)`: arg Send.
- [ ] Diagnostics carry the offender path (`Vec[*UInt8]`, `Mutex[FfiHandle]`)
  and a hint: move it, clone it, wrap in `Mutex`/`Channel`, or use an
  explicit marker impl.
- [ ] Escape hatch: check whether the parser supports `unsafe impl Send for
  T` / `unsafe impl Sync for T`. If not, add a narrow, documented
  `@thread_safe` annotation or an allowlist for known thread-safe FFI
  handles; do not silently mark raw pointers Send.
- [ ] Ship behind `--enforce-send-sync` for one wave; default on only after
  the ecosystem suites are clean. Record the flag in `docs/THREADING_PLAN.md`.
- [ ] Tests: `spawn move` `Vec[Int]` green; `spawn` of `*UInt8` red;
  `Vec[*UInt8]` red (today it is wrongly accepted); `&Vec[Int]` green
  (Sync); `&Cell`/`&mut` non-Sync red; `Channel[*UInt8]` red;
  `Mutex[*UInt8]` red; `Channel[Int]` green; `Option[Vec[Int]]` green;
  `Mutex[Vec[Int]]` green; an FFI handle in a thread arg red. Run the
  checker's Send/Sync tests plus the stdlib/engine/orbitdb/pulse/xvector
  suites under the flag.

## Acceptance

- The negative probes fail at check time with the offender path; the
  positive probes and the ecosystem suites pass under the flag.
- No codegen/runtime change; the existing spawn/channel programs behave
  identically when the flag is off.
- The flag/default decision is recorded; no release tag/re-cut in this task.

## Non-goals

- No new thread primitives, runtime changes, or scheduler work.
- No change to move/capture codegen (the checks are compile-time only).
- Do not edit `E:\xiom-lang\stdlib`; file stdlib-visible changes via the
  relay/wishlist.
