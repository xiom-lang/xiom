<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-13 (compiler lane; v0.64.4 SHIPPED; v0.64.5 batches m276-m280 landed)

This file captures state for continuing the compiler lane in a fresh
session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Status: v0.64.4 RELEASED; stack caught up; v0.64.5 plan in section 1c

v0.64.4 shipped 2026-10-12T13:37Z (tag on `df493530`; release run
`38256358845` GREEN; assets live). The commit map that produced it
(m254-m275; all pushed) is kept below for reference:

CURRENT STATE (2026-10-13T16:40Z; read this before anything below):

- origin/main = `91860f11` (no doorbell push as of this refresh; fetch
  before trusting). Local main = the m281 handoff commit (this file), AHEAD by 10:
  `633ee65b`+`dc3a0102` (v0.64.5 plan/refresh docs) + the six v0.64.5
  batch commits m276-m281 + the m281 handoff follow-up:
  - `0268c7d9` m276 env-unwrap veto on concretely-typed names (T1: kv
    empty-store startup AV; REL-20261012-1418-pulse FIXED+relayed).
  - `29e35ef8` m277 cast-call C handoff keeps the RAW fn code (two-word
    env: 0=thunk, 1=raw) + nested fn-type markers parse (T2: B-22 +
    REL-20261012-1403-packages FIXED+relayed).
  - `484d9f21` m278 same-named types keep module identity (T3: engine
    module-root AV; REL-20261011-2137-engine FIXED+relayed; the R39/R44
    `colliding` triage drives the injection rename).
  - `1a8141fb` m279 as_ptr/as_mut_ptr inference -- `*T + Int` was Str
    concat (B-17 + B-16 FIXED+relayed; lock `ffi_ptr_arith`).
  - `e1207485` m280 unsafe Copy-Out pointer gate + temporary Vec[Str]
    element concat (REL-20261011-2230-bindings FIXED+relayed; lock
    `e2e_m280_unsafe_ptr_temp_str`).
  - `77827766` m281 raw `p[i]` was the UTF-8 `xiom_char_at` decoder
    (B-24 stale-zero reads at low-byte-0x00 rounds; B-20/B-11 readback
    zeros) -> scalar-pointee pointer-index path; bare `null` at a
    pointer arg materialized as a stack byte (B-23) -> `inttoptr i64 0`;
    `base + off` confined tails Str-copied (B-20 helper) ->
    `binary_ptr_arith` gate on all three Copy-Out sites. Locks
    `ffi_ptr_slot_readback` + `null_literal_marshal` (ci.yml
    front-locks) + fixtures. Also re-probed
    REL-20261010-1549-packages-3 GREEN (m280 fixed it; `whole=77
    stmt=88`). Relay: item fixes 2051-bindings-2 / 0304-bindings /
    1548-bindings-2 / 1549-packages-3 + check rows (relay commit
    `438b714`).
  Do NOT push without owner authorization; the doorbell cron may push
  staged work -- check origin first.
- Relay bus SHAs since the refresh: 82151ff (B-17/B-16), 0fd83fc (2230),
  `438b714` (m281 fixes + rows);
  NEXT-BATCH FIX REFS rows + history bullets in
  release-checks/compiler-compiler-v0.64.4.md are current.
- Triage T1-T3 CLOSED (section 1c bullets carry the root causes).
- FFI-pointer batch: DONE (m279/m280/m281). B-11/0304-bindings +
  1548-bindings-2: the out-param recycling double-read (u32 vs byte)
  is the B-24/B-20 `p[i]` char_at family -- fix shipped m281; the
  SDL3 addendum shape reproduces as a readback through the same path
  (locked in `ffi_ptr_slot_readback`); reporters re-verify at the next
  archive. B-23/B-24 REL-20261012-2051-bindings-2 FIXED+relayed.
- Selfhost: phase-4 worktree (ses_eded8c4d8ffe, sole writer) prompted
  2026-10-13T13:00Z to rebase on e1207485 + continue
  docs/checklists/selfhost-phase6.md; sibling ses_ef8e3a27cffe stays idle.
- Verification: the full battery = e2e g1/g2 filters (ci.yml lines 153/154)
  + the front-locks CLI list (now incl. `ffi_ptr_arith`,
  `ffi_ptr_slot_readback`, `null_literal_marshal`) + xiom-check/
  xiom-codegen/xiom-graph libs + xiom-pkg; capture compiler output to files.
  m281 battery: e2e 16 + 171 green, front-locks green, libs 199/12/36,
  xiom-pkg 79.

Landed in the m254-m275 release wave (historical, already pushed):

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
- `b1ee2f4f` chore(release): v0.64.4 prep -- workspace + SELFHOST_VERSION
  bump, `release-notes/v0.64.4.md` draft (4 compiler highlights; dry-run
  convert 6 total), VSIX README pin.
- `745ff47e` m272 fn-value ABI regressions found by the gate's full e2e:
  C-boundary-only `callee_is_extern` unwrap gate + module-qualified
  `fn_typed_params` keys; locks = the six tests added to ci.yml.
- `c44d9fae` m273: m219 IR-shape lock follows the m263 `i64 ()**` spelling.
- `b873c6d6` m274: B-18 fix (bare fn NAME to an extern fn-typed param
  passes the raw code address; the four call-site wrap paths are gated by
  `callee_is_extern`); m263 CLI lock gained the bare-name arm. B-19
  resolved `wontfix` -- package-side `UInt8` vs `Uint32` declaration for
  SDL_WasInit (see COMPILER_BUGS m274; probe/package evidence).
- `4c42fc0a` m275: stdlib wave-regression fix (REL-20261011-2045-stdlib):
  the As-cast env unwrap now requires the compiled value to be the i64 env
  slot, so a fn-typed param name leaked into closure_locals can no longer
  hijack a float's cast (`Float32.to_float(v) { return v as Float64; }`);
  collision lock added to the m263 CLI fixture.
- `df493530` chore(release): v0.64.4 pin + notes JSON + gate record (the
  release commit); `91860f11` released-record docs. Gate record:
  `docs/RELEASE_GATE_v0.64.4.md`.

## 1b. Release wave state (next actions)

- PIN DONE (2026-10-11T22:38Z): stdlib tagged `stdlib-v0.64.4` = `3df0843`
  and verified (witness green on the v0.64.4 archive from the `87c6ca3c`
  push; metro revert `4e8b696`; refresh run 38177738188: corpus 954/954,
  probes 288/288, modules 509/509, barename 0/509;
  REL-20261011-1812-compiler verified + closed).
  - `STDLIB_VERSION` = `3df0843b2c384f9768867d96cfd4a611bed3034b`; vendored
    `E:\xiom-lang\xiom\stdlib` is a worktree pinned at the tag;
  - `release-notes/v0.64.4.json` converted + verified from the clean tag
    checkout (6 highlights = 4 compiler + 2 stdlib, 0 breaking, 2 docs);
  - origin/main = `df493530` (the release commit; do NOT push again until
    the tag run settles).
- RELEASED (2026-10-12T13:37Z): tag `v0.64.4` on `df493530`; release run
  `38256358845` GREEN; assets live (linux-x64, macos-x64/arm64,
  windows-x64, VSIX 0.12.2, wasm bundle, SHA256SUMS).
- WAVE LANDED (2026-10-12 evening): `release-checks/compiler-compiler-v0.64.4.md`
  (created 15:01Z) carries the reporter verdicts -- bindings, packages,
  orbitdb, playground, stdlib filed; successes and the ~35 still-red rows
  are distilled in section 1c. The to-all item `REL-20261011-1731-compiler`
  stays open until every row has a verdict (no compiler action needed for
  that bookkeeping).

## 1c. Next program (v0.64.5) -- plan from the v0.64.4 release-check

The v0.64.4 verdict wave is landing (`release-checks/compiler-compiler-v0.64.4.md`,
created 15:01Z). Wave closed: B-13/1904/B-14/B-15/B-18/B-19; m260 casts
PART B; C-ORBIT-06/07; 2100-pkg WSL paths; 1548-stdlib-2/5; packages -4/-6;
1900-engine + 1803-selfhost (fixed in the wave); 2045 (m275). Still red /
open: ~35 rows. Plan:

TRIAGE FIRST (P0):
- T1 `REL-20261012-1418-pulse`: DONE (m276, this commit). Not a v0.64.4
  wave regression but the m263 env-unwrap class on the EMPTY-store path:
  kv_open -> _create_segment -> stdlib write_u32_le's `var x = v as Int`
  unwrapped a leaked closure name (`v`); m275's i64 gate missed the
  zext'ed UInt32. Fix = type veto in `unwrap_fn_env_for_c`. Witness:
  tests/regression/m276_env_name_leak_cast (pre-fix 0xC0000005, post-fix
  exit 0) + PULSE empty-store serves + creates seg-1. (The reporter's
  "exit 0" was a %ERRORLEVEL% parse-time artifact; it AVs at 0xC0000005.)
- T2 `REL-20261012-1630-bindings` B-22: DONE (m277, this commit). Root
  causes: (a) nested fn-type markers split at `find(") -> ")` +
  `split(',')` (the `unknown type 'Int32, Int32) -> Int32'` family,
  1403-packages) -- now depth-aware `split_fn_marker_body`/
  `split_top_level_type_params`; (b) cast-call fn-typed args kept the
  closure env, and the env field 0 is the env-first THUNK (handing it to
  C shifts every arg) -- `wrap_fn_ref_env` envs are now two words
  (0 = thunk, 1 = RAW code) and every C boundary (`f as Int`, extern
  fn-typed params, cast-call args) loads field 1. Witness:
  tests/regression/m263_fn_value_c_abi gained via_cast_param /
  via_cast2_param; probe9 cb-abi green (42/42/42); pre-fix exit 9 + the
  warning, post-fix exit 0, no warnings. Closure-to-C stays on the
  workaround APIs (closures were never C-callable).
- T3 `REL-20261011-2137-engine`: DONE (m278, this commit). Not symbol
  mangling but TYPE identity: the catalog injection deduped same-named
  types by BARE name, so `xiom.<root>.render.mesh.Mesh` vs stdlib
  `xiom.geom.geometry_3d.Mesh` were decided by ASCII module order
  (`Runtime` < `geom` < `runtime`/`rt` < `zz`) -- the loser compiled
  against the winner's layout (field 2 vs 0) -> AV. Fix: collision
  pre-scan + module-qualified injection of colliding types,
  `pick_deterministic` proximity (current module, then entry `use`
  imports via the new `primary_imports`), and injected flat fns derive
  module context from their leaf-qualified names. Witness: engine
  scratch `engine` and `zz` roots both exit 0 (zz was 0xC0000005);
  lock `e2e_m278_type_name_collision` + fixture
  `tests/regression/m278_type_name_collision/`.

THEMED BATCHES (m-numbering assigned as they land; one commit per batch):
- FFI pointer: B-16/B-17 DONE (m279) and 2230 DONE (m280: unsafe Copy-Out
  Str-copied raw-pointer returns -> gated on !expr_is_pointer at all three
  sites; temporary Vec[Str] element concat -> vec_value_xiom_type Call arm;
  lock e2e_m280_unsafe_ptr_temp_str). Remaining in this batch:
  B-11/0304/1548-bindings-2 (out-param recycling / foreign buffer -- not yet
  triaged) and B-23/B-24 (REL-20261012-2051-bindings-2: null literal to C;
  ffi out-slot reads after a C write).
- Thread/spawn runtime: 1627/1402-xvector + 2053-engine (fn-value runtime
  entrypoint AV; marshal half fixed at v0.64.4).
- Allocator/runtime: 2032-engine (Vec drop leak), 2320#1 (with_len HBO,
  ASAN), 1548-bindings-5 (confined alloc/free spin).
- Borrow/codegen: B-21 (undefined IR temp on &mut of struct-returning
  call), 1659-orbitdb (copy elem-assign aliasing), 1549-packages-9 (bare
  &mut Int reads pointer).
- Resolver/checker: 1548-bindings-4, 1457-engine (bare sqrt), 2320#3
  (4-segment const), 1548-stdlib-6, 1550-orbitdb-2 + 1358-orbitdb
  (--check T001 + Linux rc), 1709-xvector, XVC-C-13, 1053/1456 (hash /
  Mutex shadow), 1626.
- Contracts/clauses: 1549-packages-8 (recursion AV), 1549-packages-3
  (unsafe return cast null), 1549-packages-2, 1411-engine, 1548-stdlib-4/7,
  1549-stdlib-2..7 (payload reads, polyhedra, cross-type callbacks,
  shape-mismatch silent).
- Tooling/registry: registry B1/B2/trust/resolve, 1549-packages-5/7,
  1500-benchmark cache ask, website-7/8/9, benchmark trampoline row, pulse
  RSS, playground rows (coercion/buffering/drvfs).
- macOS leg: 1549-pulse/1609-pulse typed `@llvm.memset.p0i8.i64` emission.

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
- State (last confirmed 2026-10-11 ~19:40Z; re-check with agent_manager
  before relying on it): phase 6 at **T6 4/64** (P0-P1e + P5a landed; all
  five gates re-run green; the relay item `REL-20261011-1525-selfhost`
  records P1a/P1b/P1c/P5a + T3 meter 3/64). Remaining: P1 remainder
  (method receivers: self alloca + field GEPs + `x.method()` calls) ->
  P2 tuples / P5 stack arrays -> phase-6 meter flip (4 of 11).
- Cadence: the session works a chunk then idles. Check with agent_manager
  `list`; if idle, prompt: continue the next unchecked stage per
  `docs/checklists/selfhost-phase6.md`, rebase onto the newest LOCAL main
  before full parity runs, sole writer, same commit/doc/relay discipline.

## 4. Release coordination -- DONE (historical)

v0.64.4 coordination complete: `REL-20261011-1731-compiler` (to all;
status + updates 1-6), `1731-compiler-2` (to stdlib), `1812` (pin
decision; verified + closed), `2014` (push nudge; fixed). The only
follow-up is bookkeeping: close `1731` once every release-check row has a
verdict (the check file is the source of truth). Nothing else pending.

## 5. Verification command set (copy/paste)

- `cargo build -p xiom`
- e2e groups from ci.yml (PowerShell; the `-split` matters):
  `$l1=(Get-Content ".github\workflows\ci.yml")[152]; $fa1=($l1.Substring($l1.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa1`
  `$l2=(Get-Content ".github\workflows\ci.yml")[153]; $fa2=($l2.Substring($l2.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa2`
  (currently 16 + 167 green; the m272 six joined group 2)
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
- `gh` CLI IS available: `gh run list -L 8`, `gh run view <id>`, `gh run
  watch <id> --exit-status`, `gh release view v0.64.4 --json assets`.
- The vendored stdlib is now a proper git WORKTREE pinned at `3df0843`
  (not the old stale checkout). To re-pin later: `git -C
  E:\xiom-lang\stdlib worktree prune`; rename the old dir aside; `git -C
  E:\xiom-lang\stdlib worktree add --detach E:\xiom-lang\xiom\stdlib
  <sha>`; delete the old dir after verifying.
- Push regime: NEVER push without the owner's authorization. The v0.64.4
  pushes (20:21Z, 21:10Z, tag, release-record) were owner-authorized; the
  doorbell cron may also push staged work -- ALWAYS `git fetch` and check
  `origin/main` first.
- The scheduled "XIOM Heavy Suites" workflow is PRE-EXISTING broken
  (rustup/cargo-fuzz setup; ~45s failure on every run since 2026-09-21) --
  ignore it for release purposes.
- Lesson from m275/2045: `local.closure_locals` (and the other
  name-keyed maps) are NOT function-scoped -- when adding a new consumer,
  gate on the compiled LLVM type where possible (an env is always the
  `i64` slot) instead of trusting the name set.

NEW (2026-10-13, v0.64.5 triage + FFI batches):

- Reporter "exit 0" from `cmd /c "... & echo EXIT=%ERRORLEVEL%"` is a
  PARSE-TIME artifact (expands before the app runs); capture exit codes
  with `cmd /v /c "... & echo EXITCODE=!ERRORLEVEL!"` or PowerShell
  `$LASTEXITCODE`. T1's "silent exit 0" was really 0xC0000005; WER can
  hold the process so a short watchdog misses it -- check the Application
  Error event log.
- Control the PRECONDITION state in repros (T1: the kv AV only fired on
  an EMPTY store dir; once one run created seg-1 every later build
  served, silently invalidating a whole bisect round).
- The loader has TWO module paths: EAGER nested Module decls (small
  single-file/dir fixtures -> qualified type keys) vs the LAZY catalog
  injection (`collect_external_decls` -> flat, bare-deduped). A
  single-file fixture often CANNOT reproduce injection-path bugs; use a
  real project tree A/B (the engine scratch copies) as the witness.
- xiom-check's R39/R44 `colliding` triage (shape rules; identical-layout
  facade duplicates like the two stdlib AtomicInt decls stay bare) is THE
  collision authority -- derive new injection-side logic from that map; a
  hand-rolled bare-name pre-scan broke t2-queue (AtomicInt).
- Builtins (as_ptr/as_mut_ptr, null, ...) need an INFERENCE counterpart:
  call.rs lowers them, but bindings stay untyped unless
  infer_call_return_xiom mirrors them -- and `i8*` is ambiguous (Str vs
  *UInt8), so gate operator lowering on AST-level pointer-ness
  (`expr_is_pointer`), never on the LLVM type alone (Copy-Out Str-copied
  a *UInt8 return; `base + N` became str_concat).
- relay.py notes: PowerShell passes argv to python; EMBEDDED DOUBLE
  QUOTES break the argument (the note split at `->`). Use quote-free
  notes.
- cargo from the wrong cwd fails: use
  `cargo build --manifest-path E:\xiom-lang\xiom\Cargo.toml -p xiom`
  when the probe shell sits in another repo.
- Fixture conventions: pure-XIOM checks -> e2e fixture + ci e2e group 2
  filter; anything needing `--c-source` -> CLI test under
  crates/xiom/tests/ (ffi_fnptr_abi.rs pattern) + the front-locks list.

## 7. m-numbering

- m261-m281 LANDED; **next free is m282**.

## 8. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), starting the v0.64.5
program after the v0.64.4 release. Read docs/SESSION-HANDOFF.md FIRST
(state, the v0.64.5 plan in section 1c, verification commands, ops rules);
consult docs/RUSTC-STUDY.md + docs/rustc-study/ for reference patterns;
rustc source is E:\repos\rust (READ-ONLY; no git writes there).

STATE: v0.64.4 is RELEASED (tag v0.64.4 on df493530; release run
38256358845 GREEN; assets live) and the verdict wave landed
(release-checks/compiler-compiler-v0.64.4.md in E:\xiom-lang\xiom-relays;
~35 rows still open). Git: origin/main = 91860f11; local main has ONE docs
commit on top (633ee65b, the v0.64.5 plan). Do NOT push without the
owner's authorization; fetch and check origin first (the doorbell cron
may push staged work).

NEXT -- v0.64.5 per handoff section 1c. TRIAGE FIRST:
T1 REL-20261012-1418-pulse: kv-mode PULSE exits 0 silently on v0.64.4
(jsonl fine; v0.64.3 control green; repro docs/repro/kv-hang-0644/ in
E:\xiom-projects\xiom-pulse) -- treat as a wave regression, bisect the
diff if needed;
T2 REL-20261012-1630-bindings B-22: fn-typed PARAM at a cast-call
argument position marshals to 0 -- fire the env->code unwrap at cast-call
arg positions + parse nested fn types (REL-20261012-1403-packages);
unblocks the glfw 0.6.0 direct handoff; completes the B-13 family;
T3 REL-20261011-2137-engine: module-root rename AV (rt/ zz/ runtime/
Runtime/ crash; engine/ passes) -- name-dependent codegen.
Then the themed batches (section 1c): FFI pointer; thread/spawn runtime;
allocator/runtime; borrow/codegen; resolver/checker; contracts/clauses;
tooling/registry; macOS typed-memset. Number from m276; ONE commit per
batch (fix + fixtures + docs + COMPILER_BUGS section) + a relay fix-note
with numeric/IR witnesses; reproduce from the item's witness before
touching code; run the full battery for gate-class changes.

PARALLEL SELFHOST: phase 6 in .kilo/worktrees/selfhost-phase-4-codegen
(session ses_eded8c4d8ffeMNzBsYzBYIgAO8; last known T6 4/64, P1 remainder
next). Check with agent_manager list; if idle, prompt it to continue per
docs/checklists/selfhost-phase6.md, rebase onto the newest LOCAL main
before full parity runs; it is the sole writer of that worktree (sibling
ses_ef8e3a27cffeKY1Sz63tGGas2t stays idle).

RULES: workarounds stay until each fix ships and is re-verified from its
witness; ONE commit per batch; never push without owner authorization;
house voice; ASCII only; no destructive toolchain tests; compiler output
-> capture to a file, never pipe into Select-Object -First; A/B with the
PINNED stdlib (E:\xiom-lang\xiom\stdlib, a git worktree pinned at 3df0843;
unset XIOM_STDLIB); gh CLI is available for run/release inspection; the
heavy-suites scheduled workflow is pre-existing broken (ignore it).
```
