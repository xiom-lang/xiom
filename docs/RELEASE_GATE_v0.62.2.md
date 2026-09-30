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
      2026-09-29; RE-CONVERTED with the stdlib fragment: 6 highlights,
      `convert` + `verify --stdlib stdlib` green).
- [x] `STDLIB_VERSION` = `stdlib-perf1` (tag on stdlib commit `06d0ee7`,
      PERF-1 annotations; repo-local checkout refreshed to the pin).
- [x] Version bumps on the release commit: workspace `Cargo.toml` ->
      `0.62.2`, `selfhost/src/codegen.xi` `SELFHOST_VERSION` -> `0.62.2`.
- [x] Version history: `docs/COMPILER_VERSIONS.md` is a stale roadmap doc
      (v0.33 era, not maintained per release); the record is
      `release-notes/` + the GitHub release. No entry needed there.
- [ ] `STDLIB_VERSION` pinned to the stdlib lane's final tag (currently
      `stdlib-v0.62.0`; the lane decides the candidate tag -- the release
      checks the stdlib out at this exact ref).

## Gate A -- compiler-lane suites (all green locally, one run each)

Commands run from the repo root on this box. Expected counts as of the last
green runs (2026-09-29); re-run at the release commit and update if the
suite grew.

- [x] e2e at the release state: 2393 passed / 0 failed / **4 ignored**
      (1589 s, `--test-threads 8`; box unstable at higher parallelism --
      suites re-run in smaller chunks).
- [x] feature-reg 514/514; parser 107/107; full_diff 2/2 (selfhost T1
      gate green; T2/T3 untouched).
- [x] CLI locks: checker_locks 23/23, borrow_e001 2/2, doctor_cli 2/2,
      run_script_cli 4/4 (run test tolerates the local Defender block;
      compile+resolution asserted).
- [x] Stdlib suites against the pin (`stdlib-perf1`): stdlib_tests
      40/40 (2 threads), stdlib_execution_tests 85 passed + 2 ignored,
      stdlib_api_freeze_tests 2/2.
- [ ] `cargo check --workspace --all-targets` clean (2 known pre-existing
      `xiom-codegen` dead_code warnings only) -- not re-run this wave;
      `cargo build -p xiom` clean at 0.62.2.
- [ ] CI PR job: N/A for this flow -- pushes to main bypass PRs (owner
      policy); the tag-driven release job is the CI gate. ci.yml did not
      run on the release commits.
- [x] `python tools/ascii_guard.py check` clean; tree clean at commit.

## Gate B -- stdlib lane sign-off (external)

- [x] Stdlib suites green at the pinned tag (their harness: check_modules
      509/509, corpus 951/951 0 fail, probes 203/203, barename 0/509,
      coverage floors + doc/module-smoke ratchets OK; committed `86c5a48`,
      battery on the wave commit, tag `stdlib-perf1` -> `06d0ee7`).
- [x] **PERF-1 stdlib annotation**: all 16 pub fns in
      `stdlib/xiom/sync/atomics.xi` carry `#[unsafe_direct]` (verified in
      the refreshed checkout: 16 occurrences); pre-m166 compilers drop the
      attribute silently (reverified by the stdlib lane -- no break).
- [x] Stdlib lane confirmed the pin tag: **`stdlib-perf1`** (commit
      `06d0ee7`); `STDLIB_VERSION` updated; repo-local checkout refreshed
      to the pin.
- [x] Release-notes stdlib fragment present (`release-notes/v0.62.2.md`
      in the pin: "Standard-library atomics run at native speed"); our
      committed JSON re-converted -- 6 highlights, verify green.

## Gate P -- PERF-1 acceptance (owner-required)

- [x] Compiler half (m166) landed: parser accepts `#[attr] pub fn`; codegen
      trusts injected catalog fn keys. Locks: parser unit test +
      `regress_m166_unsafe_direct_pub_fn_trusted`. Gates: full e2e
      2393/2393 (+4 ignored), feature-reg 514/514, parser 107/107,
      checker_locks 23/23 + CLI locks, selfhost diff 2 passed.
- [x] Local measurement with the annotation applied LOCALLY: 4M atomic
      pairs 8000 ms -> 0 ms; IR shows no trampoline/guard in the wrappers
      (`docs/repro/perf-1-atomic-trampoline/`, edit reverted after).
- [x] Stdlib annotation landed + tagged (Gate B) and `STDLIB_VERSION`
      updated to `stdlib-perf1`.
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

- [x] First dry run (run 36705942509) caught a release-only blocker:
      `crates/xiom-mcp/src/main.rs` built a `CompileConfig` literal without
      the new `extra_source_dirs` field (the local `-p xiom` debug build
      never compiled xiom-mcp). Fixed; the full nine-package release build
      is green locally and every tool self-reports v0.62.2.
- [x] Second dry run (run 36707354989) GREEN on every leg: windows-x64
      2m36s, linux-x64 3m26s, macos-arm64 2m22s, macos-x64 3m40s, VSIX
      23s; the publish job correctly skipped on dispatch. macOS legs pass
      with `RELEASE_BUILD_MACOS=true` (z3 bundled per platform).
- [x] Packaging asserts ran inside every leg (CRB-3b: all nine tools
      `--version` match; z3 runs; wasm magic checked on linux; per-leg
      SHA256SUMS written). No artifact download needed for the gate --
      the in-workflow asserts + the release job's checksum guard cover
      the same surface.

## Gate E -- tag + publish (owner action)

- [x] `v0.62.2` tagged on `801d888f` and pushed (release run **36745108730
      GREEN**: guard 9 s; windows 3m18s; linux 3m22s; macos-arm64 1m20s;
      macos-x64 3m7s; VSIX 28s; publish 31s). Tag is LIGHTWEIGHT (prior
      tags are annotated); the guard and `gh release create --verify-tag`
      accept it. Do not retag -- the release is already attached.
- [x] Release page verified:
      https://github.com/xiom-lang/xiom/releases/tag/v0.62.2 with
      SHA256SUMS, `xiom-0.62.2-{linux-x64.tar.gz,macos-arm64.tar.gz,
      macos-x64.tar.gz,windows-x64.zip}`, `xiom-vscode-0.12.1.vsix`,
      `xiom-wasm-0.62.2.wasm` + glue (`xiom-wasm.js`, `.d.ts`,
      `_bg.wasm`).

## Gate F -- post-release

- [x] Docs dispatch: `compiler-release` delivered to
      `xiom-lang/website` (tag=v0.62.2, stdlib_ref=stdlib-perf1,
      compiler_ref=v0.62.2) -- the website publishes release notes/docs
      from there.
- [ ] Packages lane re-pins to v0.62.2 and re-runs (their relay).
- [ ] Benchmark lane re-run = Gate P acceptance (t2-queue ms range; no
      regression on t1/t3/t4/t5/t8).
- [ ] Website downloads page / mirror refresh (ops lane; dl-verify).
- [x] Repo-local stdlib checkout already at the pin (`stdlib-perf1`,
      `06d0ee7`).

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
