# m373 -- Parallel compile knobs + product profiling

Status: CONFIG PLUMBING LANDED on main (manifest fields + driver mapping);
profiling and default selection remain. Not part of v0.64.8.
Owner: compiler lane (Stage 6 compile-time track)
Sources: `docs/SESSION-HANDOFF.md` 2026-10-19 next-wave decision;
7C parallel lex/parse (`--parallel`, `--jobs`), I2 parallel codegen
(`--parallel-codegen`); m365 dependency-object cache.

## Problem

The 7C/I2 parallel paths already exist, but projects can only enable them
from the command line. `xiom.toml [compiler]` parses
target/release/incremental/max-depth/timeout-secs only, so a product build
(engine, xvector, orbitdb) cannot persist its preferred concurrency.

## Goal

Make `[compiler]` the project default source for the existing paths, with
CLI flags still winning:

- `parallel = true` -> 7C lex+parse.
- `jobs = N` -> worker count for 7C/I2 (`0` keeps the all-CPUs meaning).
- `parallel-codegen = true` -> I2 per-function IR emission.
- `codegen-units` is deferred to m374; do not expose a knob without a
  backend that honors it.

## Plan

- [x] Add `parallel`, `jobs`, `parallel-codegen` to `CompilerToml` and
      `CompilerConfig` (`crates/xiom-graph/src/manifest.rs`).
- [x] Map them into `CompileConfig` in the driver
      (`crates/xiom/src/main.rs`), with `--sequential` keeping both paths
      sequential and an explicit `--jobs N` keeping the CLI count.
- [x] Manifest parse lock: `parse_compiler_parallel_knobs`.
- [x] `XIOM_VERBOSE_CONFIG=1` prints the applied parallel/jobs/codegen
      values.
- [ ] Profile engine/xvector/orbitdb with the released v0.64.8 archive and
      `XIOM_TIMINGS=1`: cold/warm lex+parse, check, codegen, link; identify
      the largest phase and whether 7C/I2 actually reduce it on the product
      trees. Record the commands and numbers in this checklist or a
      `docs/perf/` note.
- [ ] Choose product defaults from the measured data. Keep the compiler's
      own selfhost parity builds sequential by default; do not change the
      historical CLI defaults.
- [ ] Add an end-to-end config test that a project `xiom.toml` enables the
      paths and `--sequential` overrides them (use the staged debug
      compiler; `XIOM_VERBOSE_CONFIG=1` makes the assertion cheap).
- [ ] Decide whether `jobs` should be clamped to available parallelism for
      CI reproducibility; document the answer.
- [ ] Only after the profile is in: wire parallel C dependency builds via
      the m365 remainder (shared dependency key per project).

## Acceptance

- `xiom build` in a project with `[compiler] parallel = true, jobs = 4`
  runs 7C with four jobs and the same output as the sequential build.
- `--sequential` overrides both `parallel` and `parallel-codegen` from the
  manifest; `--jobs N` overrides the manifest `jobs`.
- The manifest default does not change behavior for projects without a
  `[compiler]` table.
- Profiling numbers for engine/xvector/orbitdb are recorded with the exact
  release archive and `XIOM_TIMINGS` output.

## Non-goals

- Parallel checker/module-DAG or codegen-unit partitioning (m374; after
  selfhost Phase 6 parity).
- Changing the checker or codegen algorithms. This milestone only plumbs
  the existing 7C/I2 paths and measures them.
- A `codegen-units` knob before m374 provides the partitioning.
