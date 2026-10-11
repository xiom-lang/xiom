<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-11 (compiler lane, v0.64.4 batch program)

This file captures state for continuing the 2026-10-11 batch program in a
fresh session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Status: all six batches LANDED (m261-m271); verdicts pending

Landed (local main; origin/main = `677d65a7` after the doorbell push; the
six later commits are staged for the next-batch push -- NEVER push yourself):

- `677d65a7` m261/m262 module-local call precedence + module-private const
  identity. Closes REL-20261011-1124-orbitdb, REL-20261011-1052-stdlib.
- `fde5548e` m263-m265 fn-value ABI: C-boundary marshaling (B-13/1904),
  Vec[fn] element-call arity + unit returns (B-14/B-15), module-scope
  fn-ptr storage (0317-pulse-7). CLI lock `crates/xiom/tests/ffi_fnptr_abi.rs`
  (real C host) + e2e m264/m265.
- `0b1c6374` m266/m267 unresolved imports are hard single errors (2124 +
  1802 gate) + bare pattern variants prefer the scrutinee's enum (twin).
  Locks: `checker_locks::m266_unresolved_import_one_clear_error`, e2e m267.
- `c86e2f2d` m268/m269 install-target validation + deterministic tar (2100),
  packages-bearing home preference (2102).
- `0219c5e3` m270 pulse site-mode AV root-caused to the m261-class bare-call
  hijack of a module-local struct-returning helper; lock e2e m270 + fixture;
  the 1854 COMPILER_BUGS section flipped to FIXED. Closes 1854 + 2122-pulse.
- `f69b542e` m271 float Vec slot stores narrow numeric sources (XVC-C-15):
  closes REL-20261011-1134-xvector. Lock e2e m271 + fixture; IR witness
  (fptrunc/sitofp); relay item filed `fixed`, release-check row appended.

Local vs origin at the m271 checkpoint: ahead 6 (`fde5548e`..`f69b542e`);
the docs commit carrying this line rides on top.

## 2. Batch 6 (LAST) -- FLOAT32 SLOT STORES -- DONE (m271)

- Fix (`f69b542e`): `stmt.rs` Vec indexed/field store converts numeric
  sources to the element float width before the bit-level elem store
  (float sources via coerce_value -- fptrunc/fpext; Int via val_to_i64 +
  sitofp); the same branch widens Int literal stores into Vec[Float64].
- Witnesses: pre-fix exit 3 (A `a[1]=0`, B `b[1]=9.80908925027372e-45`,
  E `e.data[1]=0`; C/F/G green) -> post-fix exit 0; IR `fptrunc double 7.0`
  at the A site, `sitofp i64 7` at the B site. The xvector probe needs the
  lane-local `Vector` type; the local-struct matrix reproduces the same
  values (scratch recipe: `%TEMP%\kilo\m271\repro.xi`).
- Lock: `e2e_m271_f32_slot_store` + `tests/regression/m271_f32_slot_store/`
  (fails Some(4) pre-fix), ci.yml group 2.
- Relay: `REL-20261011-1134-xvector` acked -> fixed with evidence; to-all
  status item updated; release-check row `m271 | f69b542e` appended
  (relay commits `a3e9508` + `6386301`). Remaining gate: reporters re-run
  their probes at the archive and set `verified` or bounce.

## 3. Selfhost track (parallel)

- Worktree: `.kilo/worktrees/selfhost-phase-4-codegen` (branch
  `selfhost-phase-4-codegen`); session `ses_eded8c4d8ffeMNzBsYzBYIgAO8`;
  sibling session `ses_ef8e3a27cffeKY1Sz63tGGas2t` must stay IDLE.
- State: phase 6 at **T6 4/64** (P0-P1e + P5a landed; all five gates re-run
  green). Remaining: P1 remainder (method receivers: self alloca + field
  GEPs + `x.method()` calls) -> P2 tuples / P5 stack arrays -> phase-6
  meter flip (4 of 11).
- Cadence: the session works a chunk then idles. Check with agent_manager
  `list`; if idle, prompt: continue the next unchecked stage per
  `docs/checklists/selfhost-phase6.md`, rebase onto the newest LOCAL main
  before full parity runs, sole writer, same commit/doc/relay discipline.

## 4. RELEASE COORDINATION -- DONE

Filed and pushed from the relay clone (`E:\xiom-lang\xiom-relays`):

- `REL-20261011-1731-compiler` (to all): six-batch status. Updated at m271:
  batches 1-6 complete, stack staged for the doorbell push.
- `REL-20261011-1731-compiler-2` (to stdlib): v0.64.4 coordination -- pin
  candidate per the candidate-tag protocol, clean `release-notes/v0.64.4.md`
  fragment (v0.64.3 '>'-/'&'/'->' validator lessons), wave-106 `_METRO_K*`
  revert with the numeric witness re-verify, and their reporter rows.
- `release-checks/compiler-compiler-v0.64.3.md`: NEXT-BATCH fix refs,
  including the m271 row; all six batches listed.

Relay commits: `a3e9508` (filing) + `6386301` (m271 fix + status update).
Nothing pending here; watch for reporter verdicts/bounces at the archive.

## 5. Verification command set (copy/paste)

- `cargo build -p xiom`
- e2e groups from ci.yml (PowerShell; the `-split` matters):
  `$l1=(Get-Content ".github\workflows\ci.yml")[152]; $fa1=($l1.Substring($l1.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa1`
  `$l2=(Get-Content ".github\workflows\ci.yml")[153]; $fa2=($l2.Substring($l2.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa2`
  (currently 16 + 161 green)
- front-end locks: `cargo test -p xiom --test doctor_cli --test borrow_e001 --test checker_locks --test run_script_cli --test cxx_standard_cli --test ffi_fnptr_abi`
- units: `cargo test -p xiom-check --lib` (199, incl. full catalog corpus
  gate), `-p xiom-codegen --lib` (12), `-p xiom-graph --lib` (36),
  `cargo test -p xiom-pkg` (79).
- A/B baseline switch: `git checkout <sha> -- crates/` then build; restore
  with `git checkout HEAD -- crates/`. Attribution trick that worked
  repeatedly: disable ONE fix's effect (early `return None`) + rebuild +
  retest.

## 6. Ops notes for the fresh session (learned this session)

- NEVER pipe compiler output into `Select-Object -First N` directly -- the
  broken pipe kills the compiler (exit -1). Capture with `*> file` then
  grep the file.
- `%TEMP%\kilo` holds many stale toolchain checkouts -> W001 flood noise
  (harmless; the chosen stdlib is the pin). Suppress with `2>$null` when
  only the exit code matters.
- A/B runs: use the PINNED stdlib `E:\xiom-lang\xiom\stdlib` (unset
  XIOM_STDLIB); `E:\xiom-lang\stdlib` is the stdlib lane's moving working
  repo (commits while you work).
- Pulse 1854 repro: `git -C E:\xiom-projects\xiom-pulse archive 01dcd1b`
  into a fresh temp dir + append the unused `out_text` helper to
  `tests/test_app.xi`. A stale `a.exe` in the tree breaks 8 resource tests
  on ANY compiler (delete it, or compile with `-o` to a path outside the
  tree); fresh extracts are clean.
- bash tool: `workdir` must EXIST before the call (create the dir from its
  parent first).
- `m211_listdir_probe/` at repo root is another lane's untracked scratch --
  leave it. `.kilo/worktrees/*` belong to their sessions -- do not touch.
- relay.py: `new --body-file` for item bodies; `fix --note` inline; `commit
  --lane compiler [-m] <ids> --path <repo-file>` when a non-item file (e.g.
  a release-check) changed too.
- Diagnostics for unresolved imports: env `XIOM_TRACE_IMPORT` prints
  `[imp]` traces (kept in-tree, env-gated).

## 7. m-numbering

- m261-m271 LANDED; **next free is m272**.

## 8. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), continuing the 2026-10-11
v0.64.4 batch program. Read docs/SESSION-HANDOFF.md FIRST (it has the status,
release-coordination steps, verification commands and ops rules); consult
docs/RUSTC-STUDY.md + docs/rustc-study/ for reference patterns; rustc source
is E:\repos\rust (READ-ONLY; no git writes there).

STATE: all six batches landed (m261-m271; local main ahead 6 at the m271
checkpoint -- the doorbell pushes, NEVER push). The v0.64.4 release
coordination is filed (relay a3e9508 + 6386301); the v0.64.3 release-check
rows await reporter verdicts at the archive. Nothing is in flight on the
compiler side.

NEXT: stand by for the doorbell/archive wave. When reporters file verdicts
or bounces on the release-check rows: reproduce a bounce from its evidence,
fix it as a new m-batch (ONE commit: fix + fixtures + docs; relay fix-note
with numeric/IR witnesses and the commit ref), keep the stack unpushed for
the doorbell, and update the m-numbering line here. The v0.64.4 stdlib pin
(candidate-tag protocol) is with the stdlib lane -- watch
REL-20261011-1731-compiler-2 responses. The xvector F32-slot row
(REL-20261011-1134) is filed `fixed` at m271; xvector re-runs their probe
at the archive.

PARALLEL SELFHOST: phase 6 in .kilo/worktrees/selfhost-phase-4-codegen
(session ses_eded8c4d8ffeMNzBsYzBYIgAO8, T6 4/64, all gates green). Check
with agent_manager list; if idle, prompt it to continue the next unchecked
stage (P1 remainder -> P2 tuples / P5 stack arrays -> meter flip), rebase
onto the newest LOCAL main before full parity runs; it is the sole writer
of that worktree.

RULES: workarounds stay until each fix ships and is re-verified from its
witness; ONE commit per batch (fix + fixtures + docs); keep the stack
unpushed for the doorbell; house voice; ASCII only; no destructive
toolchain tests; compiler output -> capture to a file, never pipe into
Select-Object -First; use the PINNED stdlib for A/B; filtered test runs
only (the local lib-suite spin is a known box condition).
```
