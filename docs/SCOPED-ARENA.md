# Scoped arena for escaped/per-frame allocations

Status: LANDED on compiler main (REL-20261019-1343-engine); no release
re-cut. The official v0.64.7 archive predates the allocation routing.

## Why

m368 (`5a530cf7e`) reclaims **owned, non-escaping** Vec locals only. A
frame-shaped struct returned across function boundaries is not owned by that
pass, so the editor's `UiFrame`/`EdShellOut`/panel Vecs leaked at ~220 MB/s
(`build/xiom-editor-fixed.exe`: 2204 MB WS / 2635 MB private in 600 frames
under a 2 GB cap). A heap-header refcount / full ownership transfer is a
larger runtime change; a documented scoped arena is the accepted alternative.

The runtime already has a per-thread guard arena used by `unsafe` blocks
(`stdlib/runtime/xiom_runtime.c`: `xiom_guard_heap_enter/exit`,
`xiom_guard_alloc`, `xiom_guard_realloc`, `xiom_guard_free`). The compiler
now routes **every emitted allocation** through `xiom_guard_alloc` and every
Vec growth through `xiom_guard_realloc`; when no arena is active they fall
back to the main heap (`xiom_alloc` / `realloc`). The guard heap therefore
becomes a documented, safe scoped-arena API.

## API

```xiom
extern "C" {
  fn xiom_guard_heap_enter();
  fn xiom_guard_heap_exit();
}

xiom_guard_heap_enter();
// ... allocate/use frame data: Vec.new/push, struct returns, Str, boxes ...
xiom_guard_heap_exit();
```

- The two functions are **explicitly safe externs** (the checker allowlist);
  no `unsafe` block is required. `unsafe` in application source is not needed.
- Per-thread and nested: each thread has its own arena; enter/exit are
  reference-counted, and only the outer `exit` discards the slabs.
- `exit` discards **all** arena slabs wholesale. Vec drops inside/after the
  scope call `xiom_guard_free`, which no-ops for arena pointers and frees
  main-heap pointers normally.
- A main-heap Vec created before `enter` and grown inside keeps its main-heap
  buffer (runtime R4 guard in `xiom_guard_realloc`), so it does not migrate
  into the scope.
- The arena is zeroed, 16-byte aligned, and uses 64 KB slabs plus oversized
  slabs for large allocations.

## Contract

1. All values allocated between `enter` and `exit` must be consumed or
   copied out **before** `exit`.
2. No arena-backed pointer (Vec data, Str data, box, raw pointer) may escape
   the scope or be used after `exit`. Copy persistent values to the main heap
   first (`xiom_guard_copy_out` / `xiom_guard_copy_str` for C payloads, or an
   ordinary allocation/struct assignment).
3. Keep the scope coarse enough to cover a whole frame (all helpers that
   allocate frame data), but not so coarse that persistent state is captured.
4. Do not rely on per-object `free` inside the scope; the arena is reclaimed
   wholesale at `exit`.

## Example

`tests/regression/escaped_frame_arena/main.xi` returns a struct with three
Vecs from `make_frame`, consumes it, and wraps each iteration:

```xiom
while i < 200 {
  xiom_guard_heap_enter();
  let f = make_frame(i);
  sink = sink + consume(f);
  xiom_guard_heap_exit();
  i = i + 1;
}
```

The e2e lock `e2e_escaped_frame_arena` verifies the values/link; the memory
acceptance is the soak below and the engine editor soak.

## Evidence (2026-10-19, release compiler)

- Editor-shaped leak witness (`tests/soak/escaped_frame_vec_leak.xi`, no
  arena): **4555 MB WS / 4765 MB private**, completes in 3.2 s.
- Same shape inside the scoped arena
  (`tests/soak/escaped_frame_arena_soak.xi`): **51.7-53.3 MB WS /
  82.7 MB private**, completes in 8.5-9.9 s (flat).
- m368 3000 x 1M Float64 loop after the allocation routing: **15.0 MB WS /
  47.5 MB private**, `leak loop done`.
- `e2e_escaped_frame_arena` 1/1; m368 1/1; m37 sweep 50/50; m336 3/3;
  C-ORBIT-13 1/1; B-34 1/1; XVC-C-20 1/1; m347 3/3; `cargo test -p
  xiom-check` green.

## Owned Vecs and `#[owns_return]` (m375, 2026-10-26)

The arena is for data the compiler cannot prove owned. Owned Vecs are
main-heap and dropped at their lexical scope:

- Fresh `Vec.with_len`/`with_capacity`/`new`/`clone`/`map`/`filter`
  bindings; loop-body and match-arm scopes; break/continue paths.
- A function returning a `Vec`, `Result`/`Option<Vec>`, or an aggregate
  may carry `#[owns_return]` when every returned Vec buffer is freshly
  allocated on the MAIN heap and uniquely owned by the caller. The
  attribute is a contract (like `unsafe`): never annotate an alias, field,
  global, or persistent `st.*` buffer.
- `#[owns_return]` functions allocate with `@xiom_alloc` and bypass an
  active guard arena; the caller's drop frees the buffer. Aggregate
  frames drop each direct Vec field at the caller's scope exit.
- `io.read_at` / `io.read_file_bytes` use the contract. Engine frame
  builders should too.
- The arena remains the escape hatch for unowned/aliased block data; it
  is not a substitute for ownership when a block wraps a long-running
  loop.

## Limitations / follow-ups

- The arena is **opt-in**; m368 still handles ordinary non-escaping Vec
  locals without an arena, and the engine must wrap its per-frame scopes.
- Escaped arena data is a use-after-free by contract; the checker cannot
  enforce the scope. Keep persistent state on the main heap.
- A refcount / ownership-transfer Vec ABI (sharing escaped values safely)
  remains future work; it would let escaped Vecs be dropped without a scope.
- Strings allocated inside runtime C helpers that call `malloc` directly
  (rather than the compiled allocator) are not captured; the editor's
  per-frame Vec/compiled allocations are the dominant path.

## Reference

- m368 non-escaping drop: `docs/COMPILER_BUGS.md` 2026-10-18 m368.
- Runtime guard heap: `stdlib/runtime/xiom_runtime.c` (guard heap section).
- Engine item: `REL-20261019-1343-engine`; release archive request:
  `REL-20261019-1312-engine` (still blocked on owner GO + lane verification).
