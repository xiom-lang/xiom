# Release gate -- v0.64.4

Six-batch correctness release + release-gate regression fixes. Scope:
m254-m271 (resolver/isolation, fn-value ABI, imports/patterns, home/paths,
pulse startup AV, Float32 slot stores) + m272/m273 (fn-value ABI
regressions found by this gate's full-suite run; m219 lock update). Pin:
moves to `stdlib-v0.64.4` at the pin commit (pending the stdlib tag per
REL-20261011-1812-compiler).

## Preconditions (before the tag)

- [x] Workspace version 0.64.4 (`Cargo.toml`) and `selfhost/src/codegen.xi`
      `SELFHOST_VERSION` = 0.64.4 (`b1ee2f4f`).
- [ ] `STDLIB_VERSION` = <stdlib-v0.64.4 ref>; repo-local `stdlib/`
      refreshed to that ref; stdlib suites green on it. Pending: the stdlib
      lane runs the wave-106 metro revert on the new archive, lands it,
      re-verifies the numeric witnesses, then tags.
- [ ] `release-notes/v0.64.4.json` committed; convert+verify green. Dry run
      against the stdlib main fragment: 6 highlights (4 compiler + 2
      stdlib), 0 breaking, 2 docs. The JSON is generated at the pin from a
      clean checkout of the tag ref.
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
- The v0.64.3 m247/m250/m251 gaps remain open (unchanged; not part of this
  scope).
- m247-m262 are already on origin; `b1ee2f4f..c44d9fae` (+ this record)
  are staged for the doorbell push.

## Gate D -- tag + publish

- [ ] pending: pin commit (STDLIB_VERSION + vendored refresh + notes JSON)
      -> doorbell push -> tag v0.64.4 on the release commit -> release run
      -> assets + notes live.
