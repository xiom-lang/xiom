# Release gate -- v0.64.4

Six-batch correctness release + release-gate regression fixes. Scope:
m254-m271 (resolver/isolation, fn-value ABI, imports/patterns, home/paths,
pulse startup AV, Float32 slot stores) + m272-m275 (fn-value ABI
regressions found by this gate's full-suite run, the wave-2 bindings
verdicts, and the stdlib archive-battery bounce; m273 lock update). Pin:
`stdlib-v0.64.4` (commit `3df0843`; stdlib witness + full battery green on
the refreshed archive, run 38177738188 -- REL-20261011-1812-compiler
filed fixed at 22:11Z).

## Preconditions (before the tag)

- [x] Workspace version 0.64.4 (`Cargo.toml`) and `selfhost/src/codegen.xi`
      `SELFHOST_VERSION` = 0.64.4 (`b1ee2f4f`).
- [x] `STDLIB_VERSION` = `3df0843b2c384f9768867d96cfd4a611bed3034b`
      (stdlib-v0.64.4); repo-local `stdlib/` refreshed to that ref (a
      worktree pinned at the tag). Stdlib evidence: witness green on the
      v0.64.4 archive from the 87c6ca3c push, metro revert landed
      (`4e8b696`), m275 verified on the refreshed archive (run
      38177738188): corpus 954/954, probes 288/288, modules 509/509,
      barename 0/509, ratchets OK.
- [x] `release-notes/v0.64.4.json` committed; convert+verify green from the
      clean tag checkout: 6 highlights (4 compiler + 2 stdlib), 0
      breaking, 2 docs.
- [x] VSIX: toolchain-only release -- `editors/vscode/package.json` stays
      0.12.2 (marketplace publish skips cleanly); README pin text ->
      v0.64.4 (`b1ee2f4f`).

## Gate A -- suites at the release state

State: `c44d9fae` (m273). All runs WITHOUT `XIOM_STDLIB` (vendored tree).

- [x] full e2e 2480 passed / 0 failed / 4 ignored (910 s). The FIRST run of
      this gate found six long-standing full-suite tests RED
      (m35_z07/z23, m37_bug47_ref_params_leak, m48_writeback_aggregates,
      m49_fnfloat_constarrays, m81_fn_ref_same_leaf; all exit 0xC0000005);
      fixed by `m272` (`745ff47e`, see docs/COMPILER_BUGS.md) and re-run
      clean.
- [x] full e2e RE-RUN at `b873c6d6` (after the wave-2 `m274` B-18 wrap
      gate): 2480 passed / 0 failed / 4 ignored (890 s). Post-m274 re-runs:
      feature-reg 549/549; stdlib_tests 40/40 + stdlib_execution 85/85
      (+2 ignored) + api_freeze 2/2; checker 199/199; parser 108/108;
      locks 2/32/2/4/1/8; verify 9+37; graph 36; codegen 12;
      release-notes 6; pkg 79; `cargo check --workspace --all-targets`
      clean; encoding guard clean; nine-tool release build + v0.64.4
      reports (unchanged by m274).
- [x] full e2e RE-RUN at `4c42fc0a` (after the wave-3 `m275` As-cast env
      gate): 2480 passed / 0 failed / 4 ignored (1051 s); feature-reg
      549/549 (477 s); stdlib_tests 40/40 + stdlib_execution 85/85
      (+2 ignored) + api_freeze 2/2. The stdlib repros gt_coop / trim_d /
      p_wave102_shapes / p_wave91_shapes all rc 0; `ffi_fnptr_abi` + the
      12-lock e2e set green (the m263 fixture now carries the collision
      arm).
- [x] feature-reg 549/549 (`m273`, `c44d9fae`, updates the m219 IR-shape
      lock to the m263 fn-pointer spelling `i64 ()**` + bitcast; runtime
      unchanged, e2e m219 green before and after).
- [x] parser 108/108.
- [x] checker 199/199 (incl. the un-ignored `catalog_corpus_is_clean`).
- [x] checker_locks 2/2; borrow_e001 32/32; doctor_cli 2/2;
      run_script_cli 4/4; cxx_standard_cli 1/1; ffi_fnptr_abi 8/8 (the m263
      C-boundary lock); scripting_tests 34/34; diff_tests 15/15;
      cli_args 5/5.
- [x] xiom-verify 9/9 + 37/37; xiom-graph 36/36 (lib); xiom-codegen 12/12
      (lib); xiom-release-notes 6/6; xiom-pkg 79/79.
- [x] stdlib_tests 40/40; stdlib_execution_tests 85/85 (+2 ignored);
      stdlib_api_freeze_tests 2/2.
- [x] `cargo check --workspace --all-targets` clean (29 warnings, 0 errors).
- [x] nine-tool release build green; all nine tools report v0.64.4
      (xiom, xiom-pkg, xiom-fmt, xiom-doc, xiom-lsp, xiom-dbg, xiom-mcp,
      xiom-verify, xiom-ffigen).
- [x] encoding guard (`tools/ascii_guard.py check`) clean.
- [!] full_diff_tests `diff_check` red -- selfhost checker-parity drift,
      PRE-EXISTING (same as the v0.64.3 gate; tracked by the selfhost lane).
      The other four (tokens/ast/corpus/runtime_ffi_selfcheck) pass.

## Blockers / notes

- Release-gate round 1 (2026-10-11): the full-suite run surfaced two
  fn-value ABI regressions from the stack -- the B-13 C-boundary unwrap was
  firing at shared internal call sites (internal callees take closure
  envs), and m261's module-qualified keys missed the fn-typed param table
  (m81). Both fixed in `m272`; the six tests are now in the ci.yml e2e
  list. `m273` updates the m219 IR-shape lock.
- Release-gate round 2 (wave-2 bindings verdicts on fde5548e): `m274`
  (`b873c6d6`) fixes B-18 -- the call-site B-007 WRAP paths now hand an
  extern callee the bare fn name's raw code address (pre-m274 the wrap
  passed a closure env; C executed the heap data, the unsafe-block
  trampoline caught the fault and the enclosing fn returned 0 silently).
  B-19 resolved `wontfix` as NOT a compiler regression: the sdl3 package
  declares SDL_WasInit (a `Uint32`-returning C fn) as `fn(Int) -> UInt8`;
  m263 honors declared return widths, so the i8 read truncates 16384
  (0x4000) to 0. Same build with `fn(Int) -> UInt32` prints the v0.64.3
  contract (`elapsed=100 init=1 w_all=16384`); one-line package-side fix
  suggested to bindings.

- Release-gate round 3 (stdlib archive battery, 2026-10-11T20:45Z):
  the v0.64.4 dry-run archive battery bounced one wave regression
  (REL-20261011-2045-stdlib): any cross-module call passing a fn value to
  a module fn with a fn-typed param (game_theory, fuzzy, bench) died at
  clang with `inttoptr i64` on a `float` (corpus 953/954, probes 286/288).
  Root cause: m263's As-cast env unwrap keyed on the name-leaked
  `closure_locals` (`cooperative_game(v: fn(...))` vs
  `Float32.to_float(v)`). Fixed `m275` (`4c42fc0a`): the unwrap requires
  the i64 env slot type. gt_coop/trim_d/p_wave102/p_wave91 all rc 0
  post-fix; collision lock in the m263 CLI fixture. Pushed; stdlib
  re-runs the corpus on the refreshed archive and tags on green (954/954
  + 288/288 expected).
- No open v0.64.4 regressions remain known.
- The v0.64.3 m247/m250/m251 gaps remain open (unchanged; not part of this
  scope).
- m247-m262 are already on origin; `b1ee2f4f..c44d9fae` (+ this record)
  are staged for the doorbell push.

## Gate D -- tag + publish

- [x] pin commit (`STDLIB_VERSION` + vendored refresh + notes JSON + this
      record) -> pushed to origin main (`4c42fc0a..<pin>`).
- [ ] tag `v0.64.4` on the pin commit -> `git push origin v0.64.4` ->
      release run -> assets + notes live; then this record completes with
      the run id and asset list.
