<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Release gate: v0.62.2

**Status:** PLANNED -- gate defined, tag NOT cut, dry-run NOT yet run.
**Decision:** v0.62.2 is plannable now. Two external dependencies remain:
the stdlib lane's final sign-off + pin decision (owner relays), and the
owner's tag push (release.yml is tag-gated).

## What v0.62.2 carries

- **PERF-1 (owner-required): t2-queue trampoline fix.** Compiler half
  (m166: parser attribute order + injected-catalog `#[unsafe_direct]`
  trust) + the stdlib atomics annotation + pin. The benchmark lane's
  public-facing t2-queue must land in the ms range, not seconds.
- Earlier queued work: tier-2, R-2/R-2c, `byte_at`.
- Since v0.62.1: m162 (same-leaf catalog poison), m163 (method field
  `Vec[Str]` element concat), m164 (module-const table materialization),
  m165 (Vec growth ceiling 2^24 -> 2^32), C22 (`xiom run` sibling-module
  resolution).
- Infrastructure only (NOT shipped as a compiler): selfhost Phase 0
  harness + skeleton + progress tracker; the bootstrap binary ships only
  at 100% per the owner policy.

**Explicitly OUT of v0.62.2:** the selfhost binary (100% bootstrap policy),
the `XIOM_STRICT_BRACKETS` default flip (held for the stdlib wave), Stage 6
lint tier 3 (parked on the noise-budget decision), and the general
confinement fast-path for untrusted blocks (Stage 6 follow-up; not needed
once the stdlib annotation lands).

## Version + notes preconditions (must be committed BEFORE the tag)

- [ ] `Cargo.toml` `[workspace.package] version` = `0.62.2` (release guard
      fails the tag otherwise; all nine tools self-report this value).
- [ ] `selfhost/src/codegen.xi` `SELFHOST_VERSION` const -> `0.62.2`
      (mirrors the Rust `--emit-ir` header).
- [x] `release-notes/v0.62.2.md` + `v0.62.2.json` committed (drafted
      2026-09-29; `convert` + `verify` green: 5 highlights, schema-1).
- [ ] RE-CONVERT if the pinned stdlib ships `release-notes/v0.62.2.md`:
      the release job's `verify` regenerates WITH the stdlib fragment and
      fails on a stale JSON. Re-run
      `cargo run -p xiom-release-notes -- convert --tag v0.62.2
      --stdlib <stdlib checkout>` after the pin lands.
- [ ] `docs/COMPILER_VERSIONS.md` entry for 0.62.2.
- [ ] `STDLIB_VERSION` pinned to the stdlib lane's final tag (currently
      `stdlib-v0.62.0`; the lane decides the candidate tag -- the release
      checks the stdlib out at this exact ref).

## Gate A -- compiler-lane suites (all green locally, one run each)

Commands run from the repo root on this box. Expected counts as of the last
green runs (2026-09-29); re-run at the release commit and update if the
suite grew.

- [ ] `cargo test -p xiom-codegen --test e2e_tests -- --test-threads 12`
      -> 2393 passed / 0 failed / 4 ignored (last: 1813 s).
- [ ] `cargo test -p xiom-codegen --test feature_regression_tests`
      -> 513 passed.
- [ ] `cargo test -p xiom-codegen --test full_diff_tests`
      -> 2 passed (selfhost T1 gate stays green; T2/T3 untouched).
- [ ] `cargo test -p xiom --test checker_locks --test borrow_e001
      --test doctor_cli --test run_script_cli` -> 23 + rest green
      (run_script_cli tolerates the local Defender os-error-225 execution
      block; CI Linux takes the full path).
- [ ] `cargo test -p xiom-codegen` unit/lib targets green.
- [ ] `cargo check --workspace --all-targets` clean (2 known pre-existing
      `xiom-codegen` dead_code warnings only).
- [ ] CI PR job green on the release PR/commit (ci.yml: fuzz, robustness,
      perf budgets, stdlib API freeze, selected e2e).
- [ ] `python tools/ascii_guard.py check` clean; tree clean.

## Gate B -- stdlib lane sign-off (external)

- [ ] Stdlib suites green at the pinned tag (their harness: stdlib_tests,
      smoke battery; last known: 85/85 stdlib-exec + all smokes, with the
      concurrent-battery caveat recorded in SESSION).
- [ ] **PERF-1 stdlib annotation**: every fn in `stdlib/xiom/sync/atomics.xi`
      whose body contains an `unsafe` block carries `#[unsafe_direct]`
      (verified locally: atomics loop 8000 ms -> 0 ms; see COMPILER_BUGS
      m166). Requires compiler >= the m166 commit; tag for this wave.
- [ ] Stdlib lane confirms the pin tag for `STDLIB_VERSION`.
- [ ] Release-notes stdlib fragment present in the stdlib checkout (the
      verify step reads it when `release-notes/v0.62.2.md` exists there;
      our committed JSON must then be re-converted -- see the
      preconditions).

## Gate P -- PERF-1 acceptance (owner-required)

- [x] Compiler half (m166) landed: parser accepts `#[attr] pub fn`; codegen
      trusts injected catalog fn keys. Locks: parser unit test +
      `regress_m166_unsafe_direct_pub_fn_trusted`. Gates: full e2e
      2393/2393 (+4 ignored), feature-reg 514/514, parser 107/107,
      checker_locks 23/23 + CLI locks, selfhost diff 2 passed.
- [x] Local measurement with the annotation applied LOCALLY: 4M atomic
      pairs 8000 ms -> 0 ms; IR shows no trampoline/guard in the wrappers
      (`docs/repro/perf-1-atomic-trampoline/`, edit reverted after).
- [ ] Stdlib annotation landed + tagged (Gate B) and `STDLIB_VERSION`
      updated to that tag.
- [ ] Benchmark lane re-run on v0.62.2: t2-queue in the ms range and no
      regression on t1/t3/t4/t5/t8. THIS IS THE RELEASE ACCEPTANCE for
      PERF-1 -- the public benchmark must not ship with XIOM in seconds.

## Gate C -- packages + consumers rehearsal (external, non-blocking)

- [ ] Packages lane re-pin re-run against the v0.62.2 tag candidate
      (known-good pattern: a `xiom --check` pass on their suites; they
      move the pin and re-run everything).
- [ ] Playground/website: wasm + wasm-bindgen glue assets are packaged by
      the release job (C8/C8b guards already verify they ride the
      artifacts); docs dispatch fires post-release via
      `XIOM_RELEASE_TOKEN` (warn-only).

## Gate D -- release dry run (before the tag)

- [ ] `gh workflow run release.yml --repo xiom-lang/xiom` (workflow_dispatch
      builds artifacts only, no release). Confirm:
      both platform artifacts built; every staged tool `--version` matches;
      z3 bundled + runs; wasm magic checked; VSIX packaged and NOT
      published (extension 0.12.1 already exists market-side --
      `vscode-publish.yml` is manual and untouched).
- [ ] Download and inspect one artifact: tools run, `lib/xiom` +
      `lib/runtime` + `package.xi` + `AI_CONTEXT.md` present, SHA256SUMS
      complete.

## Gate E -- tag + publish (owner action)

- [ ] `git tag v0.62.2` on the release commit (ancestor of `main`; guard
      checks both) and push the tag.
- [ ] Watch the release job: guard -> build (win/linux; macos only when
      `RELEASE_BUILD_MACOS=true`) -> vscode -> release -> docs dispatch.
- [ ] Verify the GitHub Release page: 7+ assets, SHA256SUMS covers every
      shipped file (C8 recurrence guard), attests present.

## Gate F -- post-release

- [ ] Packages lane re-pins to v0.62.2 and re-runs (stdlib user relay).
- [ ] Stdlib lane tags/bumps per their policy; the repo-local stdlib
      checkout refreshes to the tag (was 80e767b -> final tag).
- [ ] Website downloads page / mirror refresh (ops lane; dl-verify).
- [ ] Benchmark lane may re-run against v0.62.2 (PERF-1 is NOT expected to
      change: the trampoline overhead fix is not in this release).
- [ ] SELFHOST_PROGRESS meter unchanged (9%); release stays Rust-hosted.

## Notes / risks

- The release guard requires the tag to equal the workspace version and to
  be an ancestor of `main`; do the version bump + notes in a single commit
  on `main` BEFORE tagging.
- Marketplace publishing is skipped automatically when the extension
  version already exists; toolchain-only releases do not touch the VSIX
  version (0.12.1 stays).
- `XIOM_RELEASE_TOKEN` (docs dispatch) and `XIOM_CROSS_REPO_TOKEN` (stdlib
  checkout fallback) are optional: missing secrets warn, never fail the
  release, but docs would not publish.
