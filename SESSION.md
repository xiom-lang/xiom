# XIOM Session Handoff -- 2026-08-18 01:32 (compiler session -- BUG 43-47 done, FULL SUITE GREEN)

> **Context-full handoff.** BUG 43-47 all FIXED and committed (`c2978476`,
> `aadbfc3d`, `f47c4d3b`, `2daae98d`, `6791576c` on `feat/architect`).
> **Full suite: 3914/3914 PASSED, 0 FAILED, 26 ignored** (16m45s run,
> `.testlogs/session_20260818_011502.txt`). Working tree clean except the
> pre-existing `.xiom_ai.json`.

## [OK] Full-suite result (2026-08-18 01:31)

| Suite | Result |
|-------|--------|
| e2e (all 2231) | **2268/2268** -- incl. 5 NEW regressions: `e2e_m37_bug43_result_f64_payload`, `e2e_m37_bug44_str_deref`, `e2e_m37_bug45_iface_method_generic`, `e2e_m37_bug46_generic_struct_ref`, `e2e_m37_bug47_ref_params_leak` |
| stdlib-exec | 70/70 (+2 ignores: smoke_core Eq-impl note, smoke_simd BUG-40 CRT) |
| stdlib-compile | 40/40 - feature-reg 510/510 - api_freeze 2/2 |
| scripting | **34/34** (the s_out.exe race flake FIXED -- was the only 23:04 failure) |
| checker / parser / lexer / ctfe / codegen-unit / verifier / jit | 178/178 - 96/96 - 18/18 - 96/96 - 10/10 - 27/27 - 5/5 |
| formatter / lsp / pkg / doc / ffi-gen / mcp / dbg / display | 79/79 - **38/38** - 39/39 - 4/4 - 33/33 - **39/39** - 29/29 - 5/5 (lsp+mcp green -- the stdlib layout restructure landed) |
| script-diff / integration / diff / fuzz / robustness | 15/15 - 128/128 - 24/24 (+1 ignore) - 24/24 - 63/63 |

## [OK] Completed this stretch (all committed)

| Item | Result |
|------|--------|
| scripting `test_standalone_simple` flake | **FIXED** -- both standalone tests raced on the shared `s_out.exe` output path (parallel threads, Windows sharing violation). Output path now derives from the unique script name. |
| BUG 43 -- Result[Float64, Str] payload read via sitofp (direct-call scrutinee) | **FIXED** -- `scrutinee_payload_xiom` resolves call-scrutinee payloads via `callee_return_xiom`; smoke_core_convert exit 0 |
| BUG 44 -- deref/coercion of `&Str` loads a byte | **FIXED** -- one-star strip, new `ref_locals` tracking for `var p = &s`, `coerce_ref_arg_to_pointee` for `&T`->T auto-coercion |
| BUG 45 -- method-form interface dispatch in generic fns -> stub | **FIXED** (root cause = BUG 46's mono param degradation) |
| BUG 46 -- generic `&UserStruct[T]` param field reads -> garbage | **FIXED** -- mono Ref arm missed module-qualified struct keys -> `i64*`; qualified-suffix check restores `%struct.{qualified}*` |
| BUG 47 -- `ref_params`/`param_locals` leak across fns -> AV | **FIXED** -- both sets now cleared in the per-fn reset block (a `&T` param named `b` in an earlier fn misclassified a later fn's value param `b` -> deref'd address 7) |
| BUG 41/42 follow-up -- mono Ref arm made `&Slice[T]`/`&[N]T` lowering unreachable | **FIXED** -- restored shape checks, deleted the dead arm; **zero-warning gate restored** |

**Commits:** `c2978476` (BUG 43-47 codegen) - `aadbfc3d` (scripting race) -
`f47c4d3b` (e2e regressions) - `2daae98d` + `6791576c` (docs).

## [WARN] Workflow rules (unchanged)
- **ALWAYS use the ISOLATED binary** for compiler verification:
  ```powershell
  $env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso"; cargo build -p xiom
  Remove-Item Env:CARGO_TARGET_DIR
  $xiom = "$env:TEMP\kilo\tgt_iso\debug\xiom.exe"
  ```
- The **parallel stdlib session** owns `stdlib/xiom/**`, examples/stdlib_smoke,
  and selfhost; they commit to the SAME `feat/architect` branch and rebuild
  `target/debug/xiom.exe` constantly. The e2e/stdlib harnesses hardcode
  `target/debug/xiom.exe`. Their full-sweep finished at 01:32; the canonical
  binary now contains this session's fixes.
- PowerShell note: `$LASTEXITCODE` after `& exe ... 2>$null` in loops is
  unreliable -- capture per-command with `; $c = $LASTEXITCODE` and verify
  suspicious failures individually (3 false "compile=1" reports this session
  were capture artifacts).

## [WIP] Remaining (low priority / next session)
1. **BUG 25 #10**: `xiom.crypto` -- `use of undefined value '@_pkcs7_pad'`
   (private fn body emission vs bare-symbol resolution). PRE-EXISTING;
   smoke_crypto exits 0. Needs a dedicated session.
2. **smoke_simd**: BUG-40-era latent CRT layout miscompile (0xC0000005) --
   documented #[ignore]; toolchain investigation.
3. **Checker gap (documented, NOT a codegen bug)**: `Eq5[T].eq(el, &value)`
   associated-form calls fail the checker ("expected Self, found Int" -- Self
   not substituted in interface method params). Tower pattern + method form
   work; stdlib uses method form. Candidate checker session.
4. **stdlib_tests.rs + stdlib_execution_tests.rs path sync** after the stdlib
   layout freezes (still owned by the parallel session).
5. **Stdlib-side Eq/Ord impls re-apply** (the stdlib session's conversion was
   reverted pending BUG 47 -- now fixed, so the tower-style impls + call-site
   conversions can land; the plan is in the COMPILER_BUGS.md BUG 47 entry).

## [TOOLING] Compiler behavior additions (this session, documented in COMPILER_BUGS.md)
- **ref_locals**: `var p = &s` / `var p: &Str = ...` now tracked (like
  `ref_params`) -- `*p` derefs correctly for Str and scalar pointees;
  `&T`->T auto-coercion derefs (Str params only; scalar `&T`->T stays ambiguous
  at the codegen layer and keeps address passthrough).
- **param_locals/ref_params cleared per fn** -- cross-fn name collisions no
  longer misclassify params (the BUG 47 AV).
- **Mono `&Slice[T]` -> by-value `%struct.Vec`** (restored) and
  `&[N]T` -> `elem_ty*` (restored); `&UserStruct[T]` -> `%struct.{qualified}*`.

## [OK] Completed this stretch (all committed)

| Item | Result |
|------|--------|
| scripting `test_standalone_simple` flake | **FIXED** -- both standalone tests raced on the shared `s_out.exe` output path (parallel threads, Windows sharing violation). Output path now derives from the unique script name. Verified 3/3 full scripting runs 34/34. |
| BUG 43 -- Result[Float64, Str] payload read via sitofp (direct-call scrutinee) | **FIXED** -- `scrutinee_payload_xiom` resolves call-scrutinee payloads via `callee_return_xiom`; smoke_core_convert exit 0 |
| BUG 44 -- deref/coercion of `&Str` loads a byte | **FIXED** -- one-star strip (`trim_end_matches` stripped both stars of `i8**`), new `ref_locals` tracking for `var p = &s`, `coerce_ref_arg_to_pointee` for `&T`->T auto-coercion |
| BUG 45 -- method-form interface dispatch in generic fns -> stub | **FIXED** (root cause = BUG 46's mono param degradation) |
| BUG 46 -- generic `&UserStruct[T]` param field reads -> garbage | **FIXED** -- mono Ref arm missed module-qualified struct keys (`eqt20.Box2`) -> `i64*`; qualified-suffix check restores `%struct.{qualified}*` |
| BUG 47 -- `ref_params`/`param_locals` leak across fns -> AV | **FIXED** -- both sets now cleared in the per-fn reset block (a `&T` param named `b` in an earlier fn misclassified a later fn's value param `b` -> deref'd address 7) |
| BUG 41/42 follow-up -- mono Ref arm made `&Slice[T]`/`&[N]T` lowering unreachable | **FIXED** -- restored shape checks in the live arm, deleted the dead arm; **zero-warning gate restored** |

**New e2e regressions (5):** `e2e_m37_bug43_result_f64_payload`,
`e2e_m37_bug44_str_deref`, `e2e_m37_bug45_iface_method_generic`,
`e2e_m37_bug46_generic_struct_ref`, `e2e_m37_bug47_ref_params_leak`
(tests/regression/m37_bug4*.xi).

**Commits:** `c2978476` (BUG 43-47 codegen) - `aadbfc3d` (scripting race) -
`f47c4d3b` (e2e regressions) - `2daae98d` (docs/COMPILER_BUGS.md).

## Verification state (isolated binary, `tgt_iso`)
- **Unit suites:** checker 178/178 - parser 96/96 - lexer 18/18 - ctfe 96/96 -
  codegen-unit 10/10 - jit 5/5 -- all green, zero build warnings.
- **stdlib-exec smoke sweep:** 70/70 pass (smoke_core, smoke_sort, smoke_cmp,
  smoke_iter, smoke_serialize, smoke_crypto, smoke_complex ... all exit 0) +
  1 documented ignore (smoke_simd -- BUG-40-era CRT layout, pre-existing) +
  smoke_math_core renamed -> smoke_math_tower (passes).
- **New regression files:** all 5 exit 0 via the isolated binary.
- **The 23:04 full suite** (before this session): 3908/3909 -- the ONLY failure
  was the scripting flake, now fixed. Full-suite rerun pending the parallel
  sweep's completion (their rebuilds lock target/debug/xiom.exe).

## [WARN] Workflow rules (unchanged)
- **ALWAYS use the ISOLATED binary** for compiler verification:
  ```powershell
  $env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso"; cargo build -p xiom
  Remove-Item Env:CARGO_TARGET_DIR
  $xiom = "$env:TEMP\kilo\tgt_iso\debug\xiom.exe"
  ```
- The **parallel stdlib session** owns `stdlib/xiom/**`, examples/stdlib_smoke,
  and selfhost; they commit to the SAME `feat/architect` branch and rebuild
  `target/debug/xiom.exe` constantly (their full-sweep was still running at
  handoff). The e2e/stdlib harnesses hardcode `target/debug/xiom.exe`.
- Note: PowerShell `$LASTEXITCODE` after `& exe ... 2>$null` in a loop is
  unreliable for smoke sweeps -- capture per-command with `; $c = $LASTEXITCODE`
  and verify suspicious failures individually (3 false "compile=1" reports
  this session were capture artifacts; direct reruns all passed).

## [WIP] Remaining (low priority / handed to next session)
1. **BUG 25 #10**: `xiom.crypto` -- `use of undefined value '@_pkcs7_pad'`
   (private fn body emission vs bare-symbol resolution). PRE-EXISTING;
   smoke_crypto exits 0, so not blocking. Needs a dedicated session.
2. **smoke_simd**: BUG-40-era latent CRT layout miscompile (0xC0000005) --
   documented #[ignore]; toolchain investigation, not this batch.
3. **Checker gap (documented, NOT a codegen bug)**: `Eq5[T].eq(el, &value)`
   associated-form calls fail the checker ("expected Self, found Int" -- Self
   not substituted in interface method params). Tower pattern + method form
   work; stdlib uses method form. Candidate checker session.
4. **stdlib_tests.rs + stdlib_execution_tests.rs path sync** after the stdlib
   layout freezes (still owned by the parallel session).
5. **Full-suite rerun** once the parallel sweep finishes (scripting flake fix
   should make it 3909/3909).

## [TOOLING] Compiler behavior additions (this session, documented in COMPILER_BUGS.md)
- **ref_locals**: `var p = &s` / `var p: &Str = ...` now tracked (like
  `ref_params`) -- `*p` derefs correctly for Str and scalar pointees;
  `&T`->T auto-coercion derefs (Str params only; scalar `&T`->T stays ambiguous
  at the codegen layer and keeps address passthrough).
- **param_locals/ref_params cleared per fn** -- cross-fn name collisions no
  longer misclassify params (the BUG 47 AV).
- **Mono `&Slice[T]` -> by-value `%struct.Vec`** (restored) and
  `&[N]T` -> `elem_ty*` (restored); `&UserStruct[T]` -> `%struct.{qualified}*`.
- The 5 new e2e tests run against `target/debug/xiom.exe` -- they will FAIL
  against a stale pre-fix binary; they pass against the rebuilt one.
