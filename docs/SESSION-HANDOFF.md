<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-11 (compiler lane, v0.64.4 batch program)

This file captures state for continuing the 2026-10-11 batch program in a
fresh session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Status: batches 1-5 CLOSED, batch 6 remaining

Landed (local main; origin/main = `677d65a7` after the doorbell push; the
four later commits are staged for the next-batch push -- NEVER push yourself):

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

Local vs origin at checkpoint: ahead 4 (`fde5548e`..`0219c5e3`).

## 2. Batch 6 (LAST) -- FLOAT32 SLOT STORES

- Item: `REL-20261010-1134-xvector`; gate in `REL-20261011-1342-compiler`:
  literal stores into `Vec[Float32]` slots must narrow (`a[1] = 7.0` stores
  7.0f; `b[1] = 7` matches the `as Float32` control); the READ sibling was
  fixed in m260.
- Repro: xvector `docs/repro/xv-f32-slot-literal-write/probe.xi` (read it
  first; the item body has the exact expectations/outputs).
- Discipline: failing fixture first (`tests/regression/m271_f32_slot_store/`),
  e2e fn + ci.yml list entry, fix in xiom-codegen, verify with section 5,
  ONE commit (m271) + COMPILER_BUGS section, relay `fix` the item with
  evidence.

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

## 4. RELEASE COORDINATION (was IN FLIGHT -- finish FIRST)

The user asked to "update relay and coordinate with stdlib for the release".
Steps prepared but NOT yet filed (body files at
`%TEMP%\kilo\batch5\relay-status-body.txt` and `relay-stdlib-body.txt`; if
gone, recreate from the text below):

1) File to ALL (kind info): "compiler: six-batch v0.64.4 status -- batches
   1-5 closed (m261-m270); batch 6 in flight; stack staged" with the commit
   list above + the ask that reporters re-verify their open v0.64.3
   release-check rows at the next archive.
2) File to STDLIB (kind info): "v0.64.4 release coordination" asking:
   - PIN: publish the v0.64.4 stdlib candidate (tag/branch + sha) per the
     candidate-tag pin protocol (REL-20261010-1559-stdlib); compiler sets
     STDLIB_VERSION and re-runs convert/verify (v0.64.3 shipped with
     ab894b47 -- same shape).
   - FRAGMENT: draft release-notes/v0.64.4.md cleanly (v0.64.3 lesson: the
     validator rejects markdown/HTML '>' and '&' in highlight text; '->'
     trips it -- REL-20261010-1630/1658 are satisfied per the release-check
     rows for 1652/1701 and safe for stdlib to close).
   - WAVE-106 REVERT: m262 shipped -> metro.xi can return to the natural
     `_K0.._K3` names; re-verify the numeric witness (metrohash64(empty,0)
     == 1424585646250073642, farm64 == 0). The m262 fixture is
     rename-proof and pins value + per-module tag constants.
   - RE-VERIFY AT THE NEXT ARCHIVE: stdlib's own reporter rows our stack
     targets (m254 -> REL-20261010-1548-stdlib-2; fn-value/pattern/import
     fixes -> the 1549-stdlib-3/4/6/7-class rows; m266/m267 -> the
     phantom-import/twin pair).
3) Append a NEXT-BATCH fix-refs section to
   `release-checks/compiler-compiler-v0.64.3.md` mapping stack commits to
   the open rows: m254 -> 1548-stdlib-2; m258 -> 1548-bindings-7 +
   1549-packages-6; m266/m267 -> 1549-packages-4 (+ 1802-orbitdb); m261/m262
   -> the stdlib/const + resolver families; m263-m265 -> bindings B-13/1904
   + 1121; m268/m269 -> 2100/2102; m270 -> 1854/2122. Note batch 6 pending.
4) `python tools\relay.py commit --lane compiler -m "compiler: v0.64.4
   batch status + stdlib release coordination + next-batch fix refs"
   <id1> <id2> --path release-checks/compiler-compiler-v0.64.3.md`

v0.64.3 context: already shipped; its release-check (`release-checks/
compiler-compiler-v0.64.3.md`) is the post-release verification vehicle --
reporters verify their rows at the archive; the compiler lane records fix
refs (step 3).

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

- m261-m270 LANDED; **next free is m271** (batch 6).

## 8. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), continuing the 2026-10-11
v0.64.4 batch program. Read docs/SESSION-HANDOFF.md FIRST (it has the status,
release-coordination steps, verification commands and ops rules); consult
docs/RUSTC-STUDY.md + docs/rustc-study/ for reference patterns; rustc source
is E:\repos\rust (READ-ONLY; no git writes there).

STATE: batches 1-5 landed (m261-m270; local main ahead 4 of origin -- the
doorbell pushes, NEVER push). Batch 6 (Float32 slot stores) is the only
remaining batch.

FIRST -- finish the release coordination that was in flight (the user's ask
"update relay and coordinate with stdlib for the release"): follow handoff
section 4 step by step: file the two relay items (bodies at
%TEMP%\kilo\batch5\relay-status-body.txt and relay-stdlib-body.txt, or
recreate from section 4), append the NEXT-BATCH fix-refs section to
release-checks/compiler-compiler-v0.64.3.md, then relay-commit both items
together with --path release-checks/compiler-compiler-v0.64.3.md.

THEN BATCH 6 -- FLOAT32 SLOT STORES (REL-20261010-1134-xvector; gate in
REL-20261011-1342-compiler): read the item + xvector
docs/repro/xv-f32-slot-literal-write/probe.xi; reproduce on the current
build (literal stores into Vec[Float32] slots must narrow: a[1] = 7.0
stores 7.0f; b[1] = 7 matches the as-Float32 control; the read sibling was
fixed m260); failing fixture first (tests/regression/m271_f32_slot_store/)
+ e2e fn + ci.yml entry; fix in xiom-codegen; verify with the command set
in handoff section 5 (full e2e groups + front-end locks + the crate unit
suites; corpus gate must stay green); ONE commit (m271) + COMPILER_BUGS.md
section; relay: fix the item with evidence (numeric/bit witnesses) and the
commit ref; keep the stack unpushed.

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
