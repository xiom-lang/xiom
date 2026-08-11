# XIOM Session Handoff — 2026-08-11 16:4x (release v0.58.0 + Linux target + SELFHOST READY)

## ⚠️ CRITICAL CONSTRAINTS
1. **NEVER commit/modify `xiom-benchmark-chaos/`** — owner works in a PARALLEL session (it has uncommitted changes). It COPIES `stdlib/` directly into its build (stdlib-pin deleted). Stage only: `crates/`, `stdlib/`, `docs/`, `tests/`, `examples/`, `selfhost/` (except `_diff_*`), `packages/`.
2. **Parallel session rebuilds `target/debug/xiom.exe` frequently** — verify failures by recompiling the specific smoke manually before assuming regression. The e2e harness now RETRIES raced compiles (`a092cc35`), so full runs are reliable again; still re-verify manually if a single test fails once.
3. **Selfhost tests are IGNORED**: `test_selfhost_bootstrap_v050` + all `full_diff_tests.rs` — `#[ignore]`. Do not touch UNTIL the selfhost plan (docs/SELFHOST_PLAN.md) reaches the relevant phase — then un-ignore deliberately.
4. **Models**: ALL agents/subagents run DeepSeek V4 FLASH only. NEVER pro. `subagent_variant_overrides`: flash → high.
5. **Monorepo split DEFERRED**. stdlib becomes its own repo later (backlog).
6. **`test_diff_test_produces_correct_ir` is a PRE-EXISTING failure** on the clean tree (handoff: IGNORE) — the ONLY red left anywhere.
7. **`.xiom_ai.json` is a parallel-session artifact — never stage it.**
8. **Parallel stdlib session owns `stdlib/xiom/*.xi`** — do NOT modify unless a compiler bug needs a stdlib-side fix; log in `docs/COMPILER_BUGS.md` instead. They are mid BigFloat Phase D/transcendentals; their file states are in flight.

## CURRENT BRANCH
`feat/architect` — HEAD `4565e3af` (release v0.58.0 + Linux/WSL target committed; tree clean except `.xiom_ai.json` + parallel-session probes)

## NEXT SESSION FOCUS: SELFHOSTING — start with docs/SELFHOST_PLAN.md Phase 0
The compiler is **SELFHOST READY** (see status below). The next session's job is
**Phase 0 of the selfhost plan**: harness upgrade (3-tier diff gate), module
skeleton, runtime_ffi.xi, archive xiomc_v050.xi. Checklist:
`docs/checklists/selfhost-phase0.md`. Do NOT start Phase 1 (lexer) until Phase 0's gate is green.

## RELEASE v0.58.0 — BUILT + INSTALLED (2026-08-11)
- Package: `release\xiom-v0.58.0\` (bin/lib/runtime/mcp + install.bat/install.sh) and
  `release\xiom-v0.58.0-windows-x64.zip`.
- Installed: `%LOCALAPPDATA%\xiom\bin\xiom.exe` (PATH + desktop shortcut + .xi association).
  Verify: `xiom --version` → "XIOM Compiler v0.58.0 Production - 1112 fast-suite / 2240 e2e, zero warnings".
- Installer notes: `install.ps1 -BinaryPath <pkg>\bin -Unattended -Shortcut` (encoding fixed to
  UTF-8-BOM; `$Shortcut` shadowing fixed; binaries live in the package's `bin\` subdir).
- Release banner stats live in `package.ps1` (`XIOM_RELEASE_STATS`) — update before next release.

## LINUX/WSL TARGET — WORKING (2026-08-11, commit `4565e3af`)
- **How to build/test** (Ubuntu WSL): copy repo to ext4 (`~/axiom-linux`, exclude
  `.git target release .testlogs`), `cargo build --release -p xiom` (~40 s), then
  `target/release/xiom -o /tmp/x.exe file.xi && /tmp/x.exe`.
- Fixed for Linux: runtime C `__declspec(thread)` → `XIOM_TLS` macro; POSIX
  `xiom_trap_enter/leave` (sigsetjmp/siglongjmp + sigaction — the fault-guard works:
  smoke_guard_fault FAULT-OK, smoke_guard_retry RETRY-OK); `-lm` on POSIX native links.
- Verified on WSL: diff_test (42), all m37 regressions, smoke_bigint/complex/bigfloat/
  guard_heap — exit 0.
- Caveat: `--parallel-codegen` on Linux untested (the unsafe-ctx counter fix is
  platform-neutral); guard smokes use the POSIX trap path (Windows uses SEH).

## TEST SUITE STATE (verified 2026-08-11 15:4x–16:3x)

| Gate | Result |
|------|--------|
| `cargo build --workspace` + `cargo test --workspace --no-run` warnings | **0 / 0** |
| fast suite (`./test_summary.ps1 -Fast`) | **1112 passed / 1 failed / 1 ignored** (only `test_diff_test_produces_correct_ir`, handoff IGNORE) |
| stdlib-exec | **72/72** |
| stdlib-compile | 40/40 |
| checker | 178/178 |
| e2e full run (15:57) | **2239/2240** — the 1 failure (`e2e_p1_contract_methods`) was a harness race, now hardened (`a092cc35`); expect **2240/2240** next run |
| release binary | v0.58.0 installed, `--version` OK, compiles smokes |
| Linux binary | builds + all smokes pass in WSL |

## COMPILER-HARDENING SESSION (this session) — SELFHOST READY
- **GOAL 1 (zero warnings): DONE.** Fixed all 12 `cargo build --workspace` warnings + 5 additional test-target warnings surfaced by `cargo test --workspace --no-run`:
  - xiom-check: removed dead `enforce_unsafe_tail_type` (T005 is enforced at the FUNCTION boundary via `check_fn_decl` T003 — block-level tail check was intentionally not wired; documented in a comment) + dropped unused `ifaces`/`refs` locals.
  - xiom-doc: removed dead `xiom_version()`.
  - xiom-dbg: removed never-called `DebuggerBackend::name()` (trait + GDB/MI + CDB impls).
  - xiom-pkg: removed unused `Write` + registry imports; dropped unread `RegistryPackage.name`.
  - test targets: feature_regression_tests (unused `CacheEntry` x2, dead `src`/`result`), xiom-ctfe lib tests (unused import, dead `call_expr`, needless `mut`), scripting_tests (unused `Write`, dead `run_script` gated behind newly-declared `full-e2e` feature).
  - Commits: `4403a118`, `b82a7cc6`, `fb8405b7`.
- **GOAL 3 (hardening sweep): DONE.** Guard smokes verified manually (FAULT-OK / RETRY-OK / heap exit 0), `xiomc_v10.xi` compiles clean, runtime C warning-free (`#ifndef`-guarded `_CRT_SECURE_NO_WARNINGS`, `-Wno-deprecated-declarations` on Windows native). Commit `f99da928`.
- **GOAL 2 (no new failures):** Fast suite = 1109 passed / 4 failed / 1 ignored. 3 failures are the documented pre-existing baseline (diff `test_diff_test_produces_correct_ir`, stdlib-exec complex, stdlib-exec net). The 4th (`stdlib_exec_bigint_runs`) is a **parallel-session in-flight** stdlib failure (bigint.xi/smoke_bigint.xi have +788 uncommitted parallel edits) — NOT a compiler regression; logged as docs/COMPILER_BUGS.md NOTE 6.
- **Commits:** `refactor(xiom-check)` `4403a118` · `chore(tools)` `b82a7cc6` · `chore(test)` `fb8405b7` · `fix(runtime)` `f99da928` · docs commits below.

## BUG-1 FIX SESSION (2026-08-10 late) — UNBLOCKED the parallel BigInt/BigFloat session
- **BUG 1 (tuple-of-struct codegen) FIXED** — commit `d22068f8` (crates/xiom-codegen):
  - fn signature/return/param tuple names now match the expression-level registration (module-qualified element names); `resolve_type_key` skips generated aggregate keys (no self-nesting); `parse_struct_field_types` uses real type_meta instead of `_`-splitting. Tuple-of-struct returns (2- and 3-element, 40-byte structs) compile and run correctly.
  - **Struct `&T` params now pass the ADDRESS** (`%struct.X*`) instead of a by-value copy — `_trim(&result)`-style mutations (digits.pop()) write through to the caller (previously silently lost → phantom limbs → wrong eq/compare). Scalar `&T` unchanged; `&Vec/&Slice/&Map/&Set` keep the by-value ABI.
  - **clang -O2 hang fixed** — size-based inline policy (alwaysinline only ≤10 stmts, inlinehint ≤48) replaces alwaysinline-everything; the 735KB extended-bigint module compiles in ~14s (was >300s hang).
  - Verified: probe_tuple/probe_big/probe_big2/probe_mut/probe_ref/probe_dig/probe_cmp/probe_dm all pass; `bigint_div_mod` no longer crashes; fast suite re-run = 1109/4/1, **no new failures**; stdlib-compile 40/40; checker 178/178.
- **NOTE 7 (stdlib, parallel session owns it):** `bigint_div_mod` returns a WRONG quotient for multi-limb dividends (e.g. `1000000005/2` → q=2 r=1000000001; `987654321987654321/12345` → r ≥ b). Root cause in `_estimate_q_digit`: single-top-limb estimate, no upward correction, `est<=0 → return 1`. `stdlib_exec_bigint_runs` fails at assertion 8 until the stdlib algorithm is fixed (the crash/hang causes are gone). See docs/COMPILER_BUGS.md.
- **Commits:** `fix(codegen)` `d22068f8` · `docs(compiler-bugs)` `df8b918b`.

## M37 HARDENING SESSION (2026-08-11) — e2e failures chased, 4 compiler fixes + 5 e2e tests
- **Fix 1 (parser, `40441ca7`):** `bits[L - 1]` — the explicit-generic-call
  heuristic mis-parsed index expressions starting with an UPPERCASE ident
  ("expected ']', found -"). Speculative bracket-depth scan; commits to
  generic-args only when `]` is followed by `(`. Unblocked the parallel
  session's bigint.xi/bigfloat.xi (both use `bits[L-1]`).
- **Fix 2 (check/catalog, `1e982ebf`):** import lookups walked the ENTIRE
  source tree per failed leaf lookup (`use xiom.math` minutes-to-hang;
  probe_bit compile 138-160s). Strategy-b scan is now index-gated (skipped
  when build_index ran) and skips build/VCS/package dirs. `use xiom.math`
  ~20s→9.6s; probe_bit 138-160s→9.5s; stdlib-compile 108.7s→23.7s.
- **Fix 3 (codegen, `384d5666`):** reverted the d22068f8 non-ident `&expr`
  fallback in coerce_arg_for_param — it re-compiled the inner of
  `&mut arr[i]`, discarding the element ADDRESS (m33_z14 regression).
- **Fix 4 (verified):** BUG 8 (catalog `&Vec[Int]` empty signatures) is
  RESOLVED on the current tree (probe_bit 12&10=8); the empty-signature
  stub came from the earlier uncommitted coerce.rs state.
- **e2e tests added (`2836dfd7`):** m37_tuple_struct (BUG 1), m37_ref_mut
  (&T mutation), m37_index_arith (parser), catfix vecmod/main (BUG 8
  catalog &Vec[Int] + &struct), catfix circ_* (circular imports).
- **Circular imports verified safe:** A↔B module cycles terminate
  (cached_loaded guard), check + compile succeed, symbols resolve both
  ways. No true cycle in the current stdlib.
- **Final fast suite (02:23): 1110 passed / 3 failed / 1 ignored — back to
  the documented baseline** (diff `test_diff_test_produces_correct_ir`,
  stdlib-exec complex, stdlib-exec net). `stdlib_exec_bigint_runs` PASSES
  (parallel session's dc1dd8e4 landed the div_mod estimator fix).
  stdlib-compile 40/40, checker 178/178, stdlib-exec 70/72 in 43.7s.

## M37 BATCH 2 (2026-08-11 02:5x) — BUG 9/10/11 FIXED — fast suite 1112/1/1
- **BUG 11 (`7f7b7b54`):** unsafe-block round-trip family — block-fn
  struct-tail returns now val_to_i64 round-trip (Option/struct tails no
  longer extract field-1 → AV), ret_from_enclosing no longer pollutes the
  block value (icmp ptr,i64), fault path returns `null` for pointers
  (`ret i64* 0` clang rejection). Fixed m33_u13, m34_d01..d20, m34_y04 AND
  the last two suite failures: **stdlib-exec is now 72/72** (complex +
  net_folder pass — BUG 11 unsafe-extern doubles were this family).
- **BUG 10 (`a2aafa26`):** float literals emitted full precision
  (`{:.17e}`, was `{:.6}` — 0.123456789 truncated to 0.123457); removed a
  leftover CG02 debug print. e2e: m37_float_precision.
- **BUG 9 (`3ccd004c`):** private catalog struct types referenced by pub fn
  signatures are now injected (were i64-degraded — ABI garbage). e2e:
  catfix b9mod/b9main.
- **Test migrations:** m21_ffi_unsafe_001..009 (T007 `requires:`),
  m35_z12/z30 (T003 unsafe wrapper), m33_u13 (well-defined rewrite —
  original was dangling-&local UB).
- **FINAL fast suite (02:53): 1112 passed / 1 failed / 1 ignored — the
  only remaining failure is the documented pre-existing
  `test_diff_test_produces_correct_ir`.** stdlib-exec 72/72, stdlib-compile
  40/40, checker 178/178. All 43 previously-failing e2e tests verified
  passing with the current binary.

## M37 BATCH 3 (2026-08-11 15:4x) — last 3 e2e failures fixed; selfhost plan written
- **e2e_i2_parallel_codegen (`e0f96fef`):** `--parallel-codegen` offset
  tmp/block/str counters per function but NOT `unsafe_block_counter` —
  every fn with an unsafe block emitted `%struct.__unsafe_ctx_0` → clang
  "redefinition of type" (t2-queue). Each parallel function now gets 1000
  unsafe-block slots. All 5 ecosystem tasks pass with `--parallel-codegen`.
- **e2e_m19_read_file_content (`e0f96fef`):** `collect_ident_names` had no
  `Expr::Struct` arm — free vars inside struct literals in unsafe blocks
  (io.read_file's `Err(IOError{ message: "..." + path })`) were never
  captured → the block fn referenced the enclosing fn's register ("use of
  undefined value '%tmp3'"). Added Struct/Array/Tuple/Some/Ok/Err/Try/
  AtPre/ConstBlock/Imply/Is/Match/Closure arms.
- **e2e_safety_probe (`e0f96fef`):** t8-safety-probe called extern `free`
  outside unsafe (predates the D2 extern-call rule) — migrated to
  `unsafe { free(p); }`.
- **Verified:** all 3 e2e tests pass via the harness; stdlib-exec 72/72
  standalone (one suite run had a racy misc failure during a parallel
  rebuild — passes 4/4 manually); fast suite 1111/2/1 (2 = documented diff
  test + the racy misc).
- **Selfhost plan (`090ed5d1`):** docs/SELFHOST_PLAN.md — byte-identical
  selfhost plan (Phases 0-8: foundations → lexer/parser/checker/codegen
  parity → self-compile sha256 gate → full green; O1 selfhost code-quality
  pass after Phase 4, O2 bootstrap-chain performance pass after Phase 7;
  three-tier diff gate counts→normalized→exact bytes; risks incl.
  register-number determinism). docs/checklists/selfhost-phase0.md checklist.

## M37 BATCH 3.5 (2026-08-11 16:1x) — e2e_p1_contract_methods flake root-caused + harness hardened
- The 15:57 full run: **2239/2240** — the single failure
  (`e2e_p1_contract_methods`) was NOT a compiler bug: it passed 4/4 via
  the harness and manually, and the compiler is unchanged for that path
  (xiom_is_sorted/xiom_contains are real runtime impls). It was the known
  parallel-session binary-swap race (target/debug/xiom.exe rebuilt
  mid-suite).
- **Fix (`a092cc35`):** hardened `compile_and_run` in e2e_tests.rs with the
  stdlib-exec suite's disambiguation — 50ms flush delay before spawning the
  produced exe + up to 3 attempts where a non-zero exit is recompiled fresh
  and only a REPEAT of the same code is accepted as real. Genuine compile
  failures are never retried (no real bug can be masked).
- Verified: e2e_p1_contract_methods + representative m37/m19/i2 tests pass
  via the harness.

## WASM + PLAYGROUND (2026-08-11 17:2x) — in-browser compiler v0.58.0 shipped
- **Built:** `crates/xiom-wasm` (0.58.0, wasm-bindgen) →
  `xiom-playground/xiom_wasm.js` + `xiom_wasm_bg.wasm` (2.6 MB, now TRACKED
  in git via a `.gitignore` negation). Rebuild: `cargo build -p xiom-wasm
  --target wasm32-unknown-unknown --release && wasm-bindgen --target web
  --out-dir xiom-playground <wasm>`.
- **Playground wired:** `js/wasm-loader.js` loads it (dynamic import + init,
  server fallback); `compiler.js` compiles PURE programs fully in-browser
  (diagnostics + LLVM IR instant/offline); stdlib programs and program
  OUTPUT go through the server; version text updated to v0.58.0.
- **Compiler fix found en route:** `find_runtime_c`/`find_runtime_c_files`
  resolved the runtime relative to cwd or exe.parent().parent() — from a
  non-repo cwd (playground server) `xiom run` linked without the runtime C
  ("undefined symbol: xiom_set_args"). Now walk UP from the exe (8 levels)
  + XIOM_STDLIB sibling runtime.
- **Verified live (playwright vs localhost:3000):** pure program (IR +
  "clean ✓ (WASM)" + server run), stdlib Hello World ("Hello, XIOM!"),
  broken program ([T001] diagnostics), 0 console errors/warnings, status
  "Ready. v0.58.0 — WASM compiler loaded".
- **Release:** v0.58.0 repackaged — now includes `wasm/` (in-browser
  compiler). Commits: `a62eaf8b`-era batch + wasm/playground commits.

## KNOWN LIMITATIONS (documented, not blockers)
- **diff suite** `test_diff_test_produces_correct_ir` fails (documented pre-existing; handoff says IGNORE).
- **Full selfhost diff tests + bootstrap e2e** remain `#[ignore]`d / `XIOM_SELFHOST`-gated by design — the selfhost plan (docs/SELFHOST_PLAN.md) defines the phased path to un-gate them.
- **e2e (16min) full run at 15:09: 2237/2240** — the 3 failures (m19_read_file, safety_probe, i2_parallel_codegen) are FIXED and verified via the harness; a clean full-suite re-run is the next boundary action (expect 2240/2240).
- **BUG 2/3 (globals):** module-global struct field writes lost / fn-call initializers zero — advisory (stdlib design avoids them); tracked for a later session.
- **`to_str()` method on Float64** dispatches to the Display-interface stub (no impl registered) — stdlib uses `Str.from`/`float_to_string`; interface-dispatch gap tracked for a later session.
- **HardwareFault/ContractViolation** types exist in stdlib/xiom/error.xi; the fault path returns a type-correct zero (recoverable indicator) rather than a full `Result[T, HardwareFault]` wrapper (plan §2.8's wrapper is a future refinement).

## NEXT SESSION — START HERE
1. Run the full e2e suite at a phase boundary (fast gates are green).
2. Optionally: selfhost bootstrap milestone (re-enable full_diff_tests) — deferred by constraints.
3. Optional refinements: Result[T, HardwareFault] wrapper (plan §2.8), transaction batching audit for perf, Promotion (zero-copy Copy-Out).

## DOC UPDATES (2026-08-10, docs-only commit)
- `docs/AI_CONTEXT.md` → v0.57.0: Unsafe Confinement model (§4.4, req a–j), `#[unsafe_no_retry]`/`#[unsafe_direct]` attributes, HardwareFault/ContractViolation (§8.17), `--enable-unsafe-direct` CLI flag (§11), FFI ownership conversions (C FFI), 60-module stdlib count.
- `docs/SCALING_ARCHITECTURE.md` → v0.2: pre-selfhost review incorporated (§12 — Sealed Generics/Pre-Mono Table, Layout Hash, Compiler Daemon) + revised migration path (~15 weeks).
- `docs/CTFE_PLAN.md` → §6 integration audit (CTFE+Confinement contracts; CTFE cache → SyncRegistry at Scaling Phase 5, NOT before).
- `docs/ORCJIT_PLAN.md` → §7 integration audit (JIT inherits Confinement automatically; `JitModule::get_function_ptr()` for Live Patching).
- `docs/LIVE_PATCHING_PLAN.md` → NEW v0.61 design spec (patchable ABI, JIT sandbox, atomic swap, rollback, AI/spacecraft integration).
- `docs/BIGINT_BIGFLOAT_SESSION.md` → NEW parallel-work session spec: production BigInt extension + BigFloat build, full API/contracts/test plan, ALL stdlib libs categorized with 12 parallel session slots.
