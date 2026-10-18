# m365 -- Dependency-object cache: shared key, HIT existence, per-key lock

Status: QUEUED (design checklist; no fixer yet)
Owner: compiler lane (Stage 6 cache)
Sources: REL-20261017-1355-engine (m340 follow-ups), m340 implementation in
`crates/xiom/src/rtcache.rs`.

## Problem

The m340 dependency-object cache (`dependency_objects` ->
`ensure_objects`, `crates/xiom/src/rtcache.rs:112-229`) keys on:

- the clang path + version;
- **every** `compile_args` string passed by the caller;
- every dependency source path + content hash.

The engine reports that the key is **consumer-specific**: each of the 117
probes in one manifest computes a different key, so a cold 139-TU assimp set
is rebuilt per target (~438 s; three distinct key dirs under
`~/.xiom/jit/rtobj` within an hour: `f6abc...`, `a045...`, `108207...`).
A hello with no assimp imports took 456 s (`dep-cache MISS ... built in
438.24 s`). The documented `XIOM_NO_DEP_PORT_ARGS=1` workaround restores the
0.64.4 shape but is not the fix.

Addendum: a HIT does not verify object existence **atomically**. During the
0.64.6 gate a concurrent compile rebuilt runtime key `ab5c3608...`; a
parallel assimp compile linked a HIT list whose objects were momentarily
absent and clang failed with `no such file or directory:
...\ab5c3608...\01_fp128_helpers.o`. The key dir was complete afterwards.
Under concurrent compiles this is a hard link failure.

## Goals

1. One dependency key per (dependency identity, dependency flags) across all
   consumer targets in a project.
2. A HIT is only returned when the cache entry is **ready** and every object
   exists; a partially built/evicted entry becomes a MISS and is rebuilt.
3. Concurrent compiles of the same key do not expose a half-built entry to a
   linker, and do not delete each other's objects.
4. No change to dependency compilation semantics or the fallback contract
   (any cache error falls back to compiling in the main invocation).

## Plan

- [ ] Map the caller chain: find every `dependency_objects` call, the
      `compile_args` construction (port args, target, include dirs,
      optimization, sanitizer) and mark which args are **dependency** flags
      vs **consumer** flags. Write the split at the caller, not by string
      heuristics inside `ensure_objects`.
- [ ] Introduce an explicit `DepCacheKey` input: clang path/version, target
      triple, dependency source paths + content hashes, dependency include
      dirs/defines, dependency optimization/ABI flags. Exclude consumer
      source/output paths, consumer include dirs, probe-specific defines and
      consumer target names. Keep the runtime-object key behavior unchanged
      (`runtime_objects` still keys on the full runtime compile args).
- [ ] Add a ready marker written **last**: build into staging, rename every
      object, then create `dir/.ready`. A HIT requires the marker plus every
      object. Remove/ignore entries without the marker (treat as MISS and
      rebuild); never link a partially populated dir.
- [ ] Take a per-key build lock (lock file with PID/start time, bounded wait;
      stale-lock takeover after a timeout). The winner builds and writes the
      marker; waiters re-check the marker and reuse. On timeout/failure,
      return `Err` so the caller falls back to the main invocation.
- [ ] Add unit tests:
      - two consumer arg sets for the same dependency set produce the same
        key (hello + assimp probe), while a changed dependency flag changes
        the key;
      - a dir with one missing object or no `.ready` marker is a MISS and is
        rebuilt;
      - the m340 same-stem test (`one/util.c` + `two/util.c`) stays green;
      - a synthetic concurrent build of one key produces one complete entry
        and no missing-object list.
- [ ] Add `XIOM_TIMINGS` fields for `dep-cache key`, `HIT/MISS`, `ready`,
      `lock-wait` and the rebuild duration; use them in the acceptance run.

## Acceptance

- One cold build, then a hello and an assimp-importing probe in the same
  project **HIT the same dep key** (assert the key string equality).
- The whole-tree 117-target CI wall time is back near the 0.64.4 baseline
  (one 139-TU dependency build, not 117).
- No missing-object link failure under at least two concurrent compiles of
  the same key; a killed builder leaves a stale entry that the next compile
  detects and repairs (marker + lock).
- The m340 same-stem and runtime-object tests stay green; the engine's
  `XIOM_NO_DEP_PORT_ARGS` workaround can be retired after the acceptance run.

## Non-goals

- No dependency source/flag semantic changes; no release tag/re-cut.
- Do not delete the shared `~/.xiom/jit/rtobj` cache in tests; use a scratch
  cache root.
- No change to the runtime-object cache key (m340 behavior stays).
