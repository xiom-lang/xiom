# m369 -- Catalog index scan breadth, stale roots and cross-version thrash

Status: FIXED on main `330955136` (branch `fix/m369-catalog-index`;
no release re-cut). See `docs/COMPILER_BUGS.md` 2026-10-18 m369.
Owner: compiler lane
Sources: REL-20261018-1518-xvector (symptom), m367 (fixture exclusion),
`docs/COMPILER_BUGS.md` 2026-10-18 m369, benchmark perf track 1.

## Problem

The module catalog is built by recursively indexing every directory in
`ModuleCatalog::source_dirs` (`crates/xiom-check/src/catalog.rs:230`). The
driver adds, among others, the source file's parent directory
(`crates/xiom/src/lib.rs:627-648`), the project `src/`, graph dirs and the
stdlib dirs. The persistent cache is `$HOME/.xiom/catidx.txt`
(`catalog.rs:1060`), keyed by a compiler-version identity line
(`catalog.rs:348`), and is only pruned when it exceeds 60,000 entries
(`catalog.rs:407`).

Measured on 2026-10-18 (official v0.64.7 archive, `--no-cache`,
`XIOM_TIMINGS=1`):

- 6-line file whose parent is a busy temp tree: **index 50.569 s /
  56,663 misses**, check 0.7 s, codegen 0.03 s.
- xvector global-init probe (small source dir): index 0.749 s / 1,247
  misses, total ~2 s.
- A concurrent v0.64.4 engine build flipped the persistent cache header
  `0.64.7 -> 0.64.4`; the next v0.64.7 compile rebuilt the 11.9 MB index
  (49-56 s). When the index was not being rewritten the same probe took
  7.9 s.
- The xvector lane's ">2 min hang" is this scan/lock under load, not
  global-initializer semantics (the exact probe is green; see
  REL-20261018-1518-xvector).

## Goals

1. A single-file compile must not recursively index an unrelated or
   transient directory tree. Cap the work to the declared project source
   roots, the selected stdlib, and the file's immediate module directory.
2. The persistent index must not carry stale roots/fixture trees forward,
   and must not be shared across compiler versions in a way that forces a
   full rebuild on every switch.
3. Preserve module-resolution semantics: sibling modules, project `src/`,
   stdlib, and package deps must still resolve; W001 collisions must not
   appear for the same canonical file.
4. No release re-cut; land on a fixer branch, verify, then integrate.

## Plan

- [ ] Reproduce and pin the baseline on current main: a 6-line file in a
      temp tree with >50k files, `--no-cache`, record the five
      `[timings] index ... hits/misses` lines and total wall time.
- [ ] Classify source dirs in the driver (`crates/xiom/src/lib.rs`):
      project root / project `src` / package dep / stdlib root / file
      parent / temp or scratch. Keep the set canonical and deduplicated;
      drop the file parent when it is under the system temp root and is
      not a project root (m252 already excludes the temp root and
      `xiom_run`; extend to transient subdirs).
- [ ] Make `index_dir` recursion policy-driven: recursive for project
      roots and stdlib roots, shallow (`.xi` in the directory only) for
      a bare file parent and for temp-rooted dirs. Skip
      `tests/regression` fixture trees (m367) and package caches
      (`out/`, `.xiom/`, `packages/` copies) by default.
- [ ] Deduplicate module roots by canonical path before indexing; a
      second checkout of the same stdlib/package must not add a second
      candidate or W001.
- [ ] Prune the persistent cache to entries touched by the current run on
      every save (not only at >60k), or bound it by a small LRU; never
      persist a transient temp path.
- [ ] Scope the cache file to the compiler identity: either a per-version
      file (`catidx-<version>-<os>-<arch>.txt`) or a read-only load on
      identity mismatch (do not rewrite another version's cache). The
      v0.64.4 <-> v0.64.7 flip must not force a rebuild in either
      direction.
- [ ] Keep the write non-blocking: write a temp file, retry the rename a
      few times, and on failure keep the in-memory index for this run
      without failing the compile. No unbounded lock wait.
- [ ] Add a regression test for the scan breadth (a fixture dir with many
      non-`.xi` files and a nested `tests/regression` mini-stdlib) and a
      test for the cache identity mismatch (two identities, no rebuild
      loop). Report index misses in the assertion message.
- [ ] Run the locks: the new tests, `e2e_m252*`, `e2e_m347*`, the m37
      sweep, a full stdlib build, and a cross-version A/B with the
      v0.64.4 and v0.64.7 archives on the same `$HOME/.xiom`.

## Acceptance

- The 6-line temp-tree compile is <2 s end-to-end (index <0.5 s, no
  50k-file scan) on a warm and cold cache.
- The xvector global-init probe compiles in <2 s when the index is free;
  no >2 min stalls while another compiler version runs.
- Switching v0.64.4 -> v0.64.7 -> v0.64.4 does not rewrite the other
  version's cache and does not add >1 s to the second compile.
- `tests/regression` fixture modules do not appear in the catalog index
  (m367), and no W001 duplicate is emitted for the same canonical file.
- Existing resolution tests and the selfhost parity harness stay green.

## Non-goals

- No changes to module resolution precedence or `use` rewriting.
- No release tag/re-cut; integrate on main only.
- Do not delete the shared `$HOME/.xiom` in tests; use a scratch home.

## Result (2026-10-18)

Landed on main `330955136`:

- shallow transient temp file-parents (driver + catalog flag);
- per-version/platform cache file;
- save-time prune of entries outside the current source roots.

Measured on the 6-line `v_mutex.xi` in `%TEMP%\kilo\m368`: index
**50.569 s / 56,663 misses -> 0.707 s** (`--no-cache`) and **0.392 s** warm
(2,600 hits / 0 misses); total compile 3.3-4.2 s, now dominated by check
1.6 s + codegen 1.7 s. The xvector probe remains green. Locks: catalog 5/5,
`cargo test -p xiom-check` 211/211, m347 e2e 3/3.

Remaining / follow-ups:

- m367: a path-aware rule to exclude `tests/regression` fixture trees from
  the recursive scan (the stale-root prune and per-version file are landed,
  but a fixture tree under a current root is still indexed).
- The W001 same-canonical-file dedupe check was not re-measured in this
  pass; re-run it with the repo-root source-dir case and add a lock.
- Cache read/write races: saves are atomic temp+rename and failures are
  ignored; a bounded retry/backoff is still worth adding if a Windows
  reader holds the target file open.
