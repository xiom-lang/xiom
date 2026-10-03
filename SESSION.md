<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# CONTINUATION HANDOFF (2026-10-03 (13), v0.62.3 IN RELEASE HOLD -- two pre-existing defects; parallel lanes live)

> RELEASE STATE (2026-10-03, this session): **v0.62.3 PUBLISHED** --
> https://github.com/xiom-lang/xiom/releases/tag/v0.62.3 (release run
> 37142496836 green; nine-tool archives + SHA256SUMS + VSIX 0.12.2 + wasm;
> docs notified). Pin `stdlib-perf3` (2429ac3); both blockers fixed (m179
> Arc, m180 contracts AV) plus the m181 checker refinements. Gate A all
> green on the pin (full e2e 2411/2411 +4 ign, feature 518, parser 107,
> checker 195, checker_locks 28, borrow 2, doctor 4, run_script 2,
> full_diff 4, stdlib_tests 40, stdlib-exec 85 (+2 ign), api-freeze 2);
> Gate P ACCEPTED (t2-queue 24 ms vs 11,968 ms, no regressions); Gate D
> published. Batch e2e after C25+m182: 2410 passed / 2 failed (both
> compiler-process OOM under 8-thread load, `e2e_m21_result_option_012/013`
> -- green isolated; move `tmp/sprintc` out of the repo first for local
> full runs) / 4 ignored. Benchmark lane still to verify the official
> archive and refresh the version table. Post-release queue: R-8
> `tcp_stream_read`, contracts-arena verifier (SMT `self` + X7007),
> const-tables complex payloads, float bitcast. See
> `docs/RELEASE_GATE_v0.62.3.md`.
>
> NEXT RELEASE PLAN (owner request, 2026-10-03): candidate `v0.62.4`
> (patch, correctness/tooling). Contents in order:
> 1. lz4 context-dependent miscompile (P4c minimal repro; repeated-
>    callsite optimizer-sensitive UB) -- fix + smoke/IR lock + pinned-
>    archive verification.
> 2. iter.range `contains` C001 -- RESOLVED on the current tree: the real
>    smoke compiles/runs rc 0 (post m182/m184/m185; v0.62.3 archive still
>    red; m184 the likely fix). Registry promotes the smoke lock after the
>    candidate ships; the synthetic V8 variant's direct-form C001 stays a
>    separate latent case.
> 2b. packages findings: nested test-module import FIXED (m184, merge +
>    file-fallback lock); uninitialized-local-in-match-arm FIXED (m185,
>    literal-zero struct coercion no longer NULL-derefs; e2e lock); the
>    enum-payload Str corruption still needs a minimal repro (v0.62.4
>    target).
> 2c. stdlib finding: inline module-qualified UInt32 compare FIXED (m186,
>    deep-inference signedness fallback; e2e lock). Candidate contents so
>    far: C25, m182, m184, m185, m186, the timing instrumentation, and the
>    lz4/iter work once their audits land.
> 3. Registry polish: MOVED to v0.63.0 (registry lane, 2026-10-04): B1
>    publish packaging guard, B2 `xiom pkg yank`, `--resolve` outside a
>    workspace are minor-material; trust wording (wording-only) may ride
>    either, registry preference is all four together in v0.63.0.
> 4. Then the v0.62.x release mechanics: pin the then-current stdlib tag,
>    version bumps, notes, README/extension texts, Gate A + Gate P.
>
> BENCHMARK OPTIMIZATION for the candidate (owner request): phase timing
> shipped behind `XIOM_TIMINGS=1` (cumulative marks: start/parse/check+
> borrow/codegen). First measurement on the lz4 smoke: parse 0.003s,
> **check 10.16s (catalog+program), borrow 0.45s**, codegen 0.18s,
> clang+link ~6.6s. The checker phase is the target; a checked-stdlib-
> module cache is the scoped candidate for v0.63.0 (Stage 6).
>
> DEFERRED to `v0.63.0` (minor): Stage 6 compile-time work (per-module
> cache + separate compilation + parallel codegen; KPI compile_ms < 2 s),
> R-8 `tcp_stream_read`, contracts-arena verifier, i64<->f64 bitcast,
> `-e` fast path, **lz4** (owner-approved 2026-10-04: audit not closed;
> optimizer-sensitive v0.62.0..v0.62.2 regression), and the registry
> polish B1/B2/`--resolve`. Safety hardening after selfhost. Selfhost
> Phase 3 continues off-main and flips only at full parity. v0.62.4 stays
> the FIX patch: C25, m182, m184-m189 (+ timing instrumentation).

> KICKOFF (paste-ready): Continue the XIOM swarm compiler lane on `main`.
> Read this top section, then `docs/COMPILER_BUGS.md` (2026-10-03 entries)
> and `docs/SELFHOST_PROGRESS.md`. v0.62.3 is IN RELEASE HOLD: cut only
> after (1) the Arc `#[unsafe_direct]` size miscompile is fixed, (2) the
> `any_contracts()` AV is fixed, (3) suites are green, (4) Gate P runs on a
> FRESH stdlib tag. Method rules at the bottom of this block.
>
> 1. ARC DEFECT -- FIXED (m179, this session). Root cause (exact): the
>    parser lowers `size_of[ArcInner[T]]()` to a GenericCall whose explicit
>    type list is the reduced base `["ArcInner"]` (nested `[T]` dropped);
>    the size intrinsics never read that list, so `Arc.new_Int` took the
>    8-byte scalar fallback (malloc(8) for the 16-byte ArcInner store).
>    Fix: call.rs size intrinsics resolve the name from the generic-call
>    list first (mono-substituted), then the family fallback (Arc added to
>    the Rc/Weak list). Locks: `regress_m179_size_of_nested_generic_mono`
>    (IR, verified red-before), `e2e_m179_arc_strong_count` + fixture,
>    ci.yml line; era matrix HEAD + stdlib_pf2 rc 0 (was rc 1); IR
>    `malloc(16)`. Full e2e + stdlib-exec in the batch run with the
>    contracts fix. Era worktrees (rebuild if aged):
>    `tmp/sprintc/stdlib_v0620`, `stdlib_pf1`, `stdlib_pf2`.
> 2. CONTRACTS AV -- FIXED (m180, this session). Root cause (exact):
>    `any_contracts()` -> `!_get_index().none()`; the `.none()` receiver is a
>    CALL, `infer_struct_type_name` cannot type it, so the key degraded to
>    the bare leaf "none" and the keep-first alias bound the unrelated
>    generic `core.none[T](items, predicate)` (2 params) to the 0-arg method
>    call -> `core.none_ContractIndex` emitted with the free fn signature but
>    receiver-only args -> ABI mismatch -> 0xC0000005. Fix: call.rs derives
>    the receiver leaf via deep type inference when the shallow one fails
>    (`receiver_dispatch_leaf`), so the key resolves to
>    `contracts.ContractIndex.none`. Locks: `e2e_m180_contracts_any_av` +
>    fixture, ci.yml. Pre/post: rc 0xC0000005 -> 0. Era matrix: red on
>    v0.62.0 release and all stdlib era trees (v0620/pf1/pf2) -> compiler-side,
>    entered <= v0.62.0.
> 3. m178 FALSE POSITIVES -- FIXED (m181, this session). The m178
>    match-pattern rule landed without a full e2e pass; the batch rerun
>    surfaced 9 fixture reds, all false positives: (a) `type X = Option/
>    Result[...]` aliases not unwrapped (m21_type_edge_010/011, m36_c09);
>    (b) `Vec[Option[Int]].new()` rendered "Vec[Int]" so `g[0]` typed Int
>    (m65_vec_option_elem); (c) enums declaring their own None/Some
>    variants rejected (m32_e13/e15, m33_y14, m34_y13, m36_e04). Fixed in
>    xiom-check (`alias_base_head`, `enum_declares_variant`, recursive
>    ctor render); those fixtures are the locks, ci.yml extended. The
>    10th red, e2e_m17_zero_warnings, is a LOCAL artifact: era worktrees
>    inside the repo tree make the catalog index duplicate stdlib roots
>    (W001). Era trees moved to
>    `C:\Users\lefte\AppData\Local\Temp\kilo\sprintc` during suite runs;
>    move back (or keep out of the repo) before local full e2e. CI/clean
>    checkout unaffected.
> 4. SUITES -- run on the 3-fix batch (m179+m180+m181), pinned stdlib
>    unless noted: full e2e 2410/2411 (+4 ign), single red
>    `e2e_m90_stdlib_same_leaf_http` = the stale-pin xiom.net T001s (PASSES
>    with XIOM_STDLIB=waves); feature 518/518; checker 194/195 (1 =
>    `catalog_corpus_is_clean`, same net T001s, hardcodes the repo pin and
>    ignores XIOM_STDLIB); checker_locks 28/28; stdlib-exec 85/85 (+2 ign)
>    on the waves tree (81/85 on the pin: 4 net smokes); api-freeze 1/2 on
>    the pin (same net compile). ALL remaining reds close with the Gate P
>    STDLIB_VERSION bump. Local artifact: era worktrees under tmp/sprintc
>    make the catalog index duplicate stdlib roots (W001) ->
>    `e2e_m17_zero_warnings` red in local full runs; move tmp/sprintc OUT
>    of the repo (e.g. %TEMP%\kilo\sprintc) for those runs, restore after.
>    CI/clean checkout unaffected. Era matrix (this batch): HEAD +
>    stdlib_pf2 Arc rc 0 / contracts rc 0; both red on the v0.62.0 release
>    and all era trees pre-fix.
> 5. GATE P: ask the stdlib lane for a FRESH tag at their then-current main
>    (>= `8b23b79`; candidate `stdlib-perf3`), bump `STDLIB_VERSION`, regen
>    the api-freeze snapshot, run benchmark t2 (compiler half `185342f4`),
>    then tag v0.62.3. The pin policy: release ships the latest TAGGED
>    stdlib including waves 54-58; anything landing after the tag waits.
>
> RELAYS ALREADY PASSED: stdlib must KEEP the perf2 `#[unsafe_direct]` sync
> annotations (Arc failure is compiler-side; Gate P depends on them).
> Registry: staging index-key pin recorded (`xiom pkg trust --registry <url>
> --index-key 0f07f71a052e16f10c20c5f6198adf168e3a9f092f6bc11b6255e64e11746efb`,
> fp 0f:07:f7:1a:05:2e:16:f1); production pin unchanged.
>
> PARALLEL: selfhost Phase 3 (Agent Manager session
> `ses_f01f1e04affeAYkPxrlvgZQzMc`, branch `selfhost-phase-3-checker`) is
> resumed for the W000/W004/W006/W007 lints + borrow pass; diff_check is
> staged-green over the 83-file corpus + manifest; the meter (27%, 3/11)
> flips only at full parity. Phase 2 merged at `60731523`. Stdlib continues
> independent waves.
>
> QUEUE AFTER THE TWO BLOCKERS: mutation diagnostic (match-bound payload
> writes on `&mut` enums silently dropped; the aliasing experiment is
> documented and reverted -- recommended route is the checker diagnostic),
> `derive[Clone]` aggregates, `invariant:` placement, xiom-verify encodings,
> loop-return typing, arity symmetry, reserved `fn`, `!bool == 1`,
> type-laxness (`Vec[UInt8] = got.value` with a Str field), module-header
> nested-module compare, stdlib `Vec.push[T]` stride fix.
> PLUS (2026-10-03 relay): complex module-level const tables (Str/struct
> payloads) mis-read -- FIXED (m182): real constant globals for fully
> literal elements (`docs/repro/const-tables/const_tables.xi` rc 6 -> 0,
> NAMES str_len loop 14); IR test + e2e + ci line; feature 519/519.
> stdlib row 25 can retire complex shapes. i64<->f64 bitcast intrinsic for
> `xiom.num.float.float_bits/bits_to_float`
> (stdlib stubs documented with TODO(compiler); no stdlib change wanted).
> Arity contradiction retired (stale row); Vec[StructType] trap 10 not
> reproducible (retirement candidate).
> REGISTRY RELAY (2026-10-03): registry lane's priority = three v0.62.3-only
> regressions first. Reports received (README @ cfb624b) and all three
> reproduced on the official archive. TRIAGE: (1) iter C001 -- minimized
> (3 range-sums switch the receiver classification; amplifier =
> `intercept = !has_user_fn` in call.rs; state change still to pin),
> green on the stdlib tip; (2) cell/RefCell -- NOT compiler: the smokes
> never call release() and 6D.1 (592243a) made Ref pointer-based with
> explicit release documented; stdlib-side smoke fix; (3) lz4
> block-compress empty -- OPEN compiler-suspect (&mut Vec out-param pushes
> lost in context; emit-IR diff next). Then optional polish: B1 `xiom pkg
> publish` packaging guard (refuse artifact dirs; .xiomignore + --force),
> B2 `xiom pkg yank <pkg>@<ver>` (contract received: POST
> /packages/{name}/{version}/yank, publish-scope Bearer, optional reason,
> staging-first), install trust wording (exact copy received), `--resolve`
> outside a workspace (shape confirmed: `xiom pkg resolve <name>@<spec>`).
> Queue order: lz4 + iter amplifier, then B1, B2, wording/--resolve.
>
> DONE THIS CYCLE (all with fixture + e2e/checker + CI locks): m169 (C24-1
> same-leaf results), m170 (C24-2 thunk + Option-of-Vec), m168b (`&mut` arg
> to by-value param), C23 (driver double-opt fix; pack acceptance `C23
> present: no`), m175 (`io.read_line` Int handle inttoptr; `C24 fixed:
> yes`), m171/m172/m174 (selfhost findings (f)/(g) + literal field order),
> m176/m177 (qualified types + Result payload laxness) + scoping fixup, m178
> (ill-typed match T001 -- surfaced and fixed the stdlib `xiom.net`
> `str_slice`; stdlib retired its probe). Full e2e 2409/2409 (+4 ignored) at
> `f4734c07`. expat/nbt: NOT reproducible (port.ps1 `-1` is its watchdog
> sentinel; released v0.62.2 + both pins green, nbt 26/26).
>
> METHOD (non-negotiable): repro-first under `tmp/sprintc`; every fix gets a
> fixture + e2e and/or checker test + CI line; cargo commands sequentially;
> never rebuild `target/debug` while a suite/e2e runs (harness race produces
> empty-output failures); `python tools/ascii_guard.py check` before every
> commit (`repair --apply` on failure); commit atomically with
> SESSION/COMPILER_BUGS evidence; full e2e once per compiler batch; push
> only on the owner's ask.

# CONTINUATION HANDOFF (2026-10-02 (12), selfhost Phase 2 parser parity -- MERGED TO MAIN (`60731523`); gate 2 GREEN)

Supersedes the (11) handoff below (kept as history).

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM swarm compiler lane on `main` (Phase 2 rebased and
> fast-forward merged at `60731523`; meter **27% -- 3 of 11**). Read this top
> section, then `docs/checklists/selfhost-phase2.md`,
> `docs/SELFHOST_PROGRESS.md` and `docs/COMPILER_BUGS.md` (Phase 2 findings
> (e)-(h) + the struct-literal field-order relay). State: **v0.62.2**
> released; Phases 0-2 selfhost on main; Phase 3 (checker parity) runs in its
> own Agent Manager worktree on branch `selfhost-phase-3-checker`.
> Compiler batch landed (m171/m172/m174, `d1ab8ec4`): (f) qualified variant
> patterns now count as exhaustive, (g) aggregate-payload enum equality is a
> clean T001 instead of invalid IR, and out-of-order struct literals map
> fields by name (stdlib relay). Full e2e 2406/2406 (+4 ignored, 8 threads)
> plus feature 517, checker 195, checker_locks 26, stdlib-exec 85 (+2 ign),
> api-freeze 2/2.
> m175 landed (playground C24): `io.read_line()` now inttoptr's the Int FFI
> handle into `fgets` (was truncated to a byte and address-passed); pack
> acceptance `C24 fixed: yes` in run+compile modes on Windows and Linux;
> piped-stdin e2e + IR lock; full e2e 2408/2408 (+4 ignored, 8 threads).
> m176/m177 landed (packages): module-qualified type annotations/literals/
> params resolve to the registered struct (was silent zeros / T001), and
> `Result[Bool, Str]` no longer satisfies `Result[Int, Str]` (was a silent
> garbage read). Follow-up fixed module scoping vs the leaf fallback
> (m19/m37 regressions); full e2e 2409/2409 (+4 ignored, 8 threads) at
> `f4734c07`. Locks m176 fixture + e2e + CI, m177 checker lock.
> Registry relay: staging index digest is signed (public key
> 0f07f71a052e16f10c20c5f6198adf168e3a9f092f6bc11b6255e64e11746efb, fp
> 0f:07:f7:1a:05:2e:16:f1); staging pin is OPTIONAL for fail-closed tests;
> production pin unchanged. No action required.
> New packages findings queued (COMPILER_BUGS 2026-10-02 relay):
> match-bound `&mut` enum payload mutations dropped, `derive[Clone]`
> aggregate payloads corrupt, `invariant:` placement, xiom-verify encoding
> gaps (tooling); json legacy bugs are stdlib-side.
> RELEASE HOLD (2026-10-03): v0.62.3 is postponed until the two pre-existing
> v0.62.x defects are fixed -- contracts `any_contracts()` AV and
> `Arc.new(42).strong_count() != 1`; both confirmed on v0.62.2, green on
> v0.61.3, repros recorded in COMPILER_BUGS (2026-10-03 relay). Registry
> polish: the staging index digest is signed; for fail-closed tests pin the
> staging INDEX key with `xiom pkg trust --registry <url> --index-key
> 0f07f71a052e16f10c20c5f6198adf168e3a9f092f6bc11b6255e64e11746efb`
> (fp 0f:07:f7:1a:05:2e:16:f1); production pin unchanged, nothing else
> required. Stdlib continues independent waves; selfhost Phase 3 resumed
> (W-lints + borrow pass remaining).
> v0.62.3 SCOPE STATUS (2026-10-02): C23 fixed (`8abedb72`, pack> acceptance "C23 present: no"); C24 read_line fixed (`592c64d4`, "C24
> fixed: yes"); C24-1/C24-2 stdlib relays fixed (m169/m170); m168 residual
> fixed (565b9924); expat/nbt NOT reproducible with the exact harness on
> the released v0.62.2 + stdlib main/perf1 (nbt 26/26) -- awaiting the
> packages lane if it recurs. GATE P: stdlib-perf2 tag exists on stdlib
> main; at the cut bump STDLIB_VERSION to stdlib-perf2, regen the
> api-freeze snapshot, run benchmark t2 (compiler half `185342f4`). Extra
> hardening landed: m171/m172/m174, m176/m177 + scoping fixup. Open
> (not release-blocking): the Remaining list below.
> RELEASE DECISION (2026-10-02, recommendation): tag v0.62.3 after ONE more
> short compiler batch -- (1) match-bound payload mutations on `&mut` enums
> (json mutators, silent data loss), (2) the wave-57 alloca-dominance
> invalid IR (compile blocker for Result-match + Str-call shapes).
> Everything already in scope is GREEN. `derive[Clone]` aggregates,
> invariant placement, verify encodings, arity/loop/reserved-fn and (e)/(h)
> become v0.62.4 fast-follows. Stdlib continues wave 58 (HTTP family) in
> parallel; v0.62.3 pins the existing `stdlib-perf2` tag (no new pin
> needed), waves push/tag at the next release boundary or on request.
> Gate P executes at the cut: STDLIB_VERSION -> stdlib-perf2, freeze
> snapshot regen, benchmark t2.
> Remaining bugs: (e) recursive enum payload
> boxing, (h) large-function variant payload mapping, type-laxness intake
> (the remaining `let gb: Vec[UInt8] = got.value;` row), module-header
> nested-module compare, plus the packages rows (loop-return typing, arity
> symmetry, cross-module qualified call resolution, reserved `fn`
> identifier, spurious `!bool == 1` diagnostic; the crypto symbol is
> stdlib-side).
> Phase 2 deliverables (all committed):
> 1. Canonical `--dump-ast` on the Rust driver
>    (`crates/xiom/src/main.rs::dump_ast` + `AstDump`, clap flag, usage) and
>    `selfhost/src/main.xi` -> `selfhost_parser.dump_ast`.
> 2. Harness gate `full_diff_tests::diff_ast` un-ignored and GREEN: the
>    `--dump-ast` outputs are line-exact over the 83-file corpus
>    (`cargo test -p xiom-codegen --test full_diff_tests`, ~100 s for
>    diff_ast; T1/T2/diff_tokens also green in the same suite).
> 3. Parser port: `selfhost/src/ast.xi` (arena AST; recursive value enums
>    mis-lower -- finding (e)), `parser_state.xi` (error latch + `tk_tag`
>    kind identity -- finding (g)), `parser_expr.xi` (types/patterns/stmts/
>    exprs), `parser_core.xi` (program + top decls), `ast_dump.xi`
>    (canonical walker), facade `parser.xi`.
> 4. `docs/checklists/selfhost-phase2.md` (LANDED evidence);
>    docs/SELFHOST_PROGRESS.md gate 2 row + Phase 2 evidence; the meter
>    line updated to 27%.
> 5. Findings (e)-(h) in `docs/COMPILER_BUGS.md`; repros under
>    `tmp/sprintc/phase2_parser/`.
> Known follow-ups for the compiler lane (not blockers): (g) enum `==`
> with aggregate payloads emits invalid IR; (h) `NkExprGenericCall`
> destructure field mis-mapping in a large dispatch function; (e) recursive
> enum payload boxing; (f) qualified variant patterns false W000.
> Next work: Phase 3 checker parity (launched in its own worktree) plus the
> compiler queue above.
> Method: repro-first under tmp/sprintc; cargo commands sequentially on this
> box; never rebuild target/debug while an e2e runs; `python
> tools/ascii_guard.py check` before every commit; commit atomically with
> SESSION.md + COMPILER_BUGS.md evidence; push only on the owner's ask.

# PHASE 3 WORK LOG (branch `selfhost-phase-3-checker`, off `60731523`)

- Stage A (2026-10-02): canonical `--dump-check` on the RUST driver.
  `CompileConfig::dump_check` stops `compile_with_diagnostics` right after the
  checker (no borrow pass, no codegen); `crates/xiom/src/main.rs::dump_check`
  owns the format: `{kind} {code} {line}:{col} {message}` per diagnostic,
  `CHECK-OK` when clean, `PARSE-ERROR` on input/lex/parse failure; message
  escapes `\\ \n \r \t` only. Clap flag + usage + `cli.rs` surface test.
  Ground truth over the 83-file corpus (repro:
  `tmp/sprintc/phase3_checker/dump_check_recon.txt`): 81x `CHECK-OK`,
  `stdlib/tests/smoke/smoke_guard_fault.xi` 4x
  `warning W003 23:3/33:3/43:3/53:3`,
  `tests/regression/m37_short_circuit.xi` 1x `warning W008 10:11`. The
  positive corpus is NOT vacuous; the selfhost port must reproduce both
  lints plus keep the other 81 clean.
- Stage B (2026-10-02): selfhost checker stage-1 port + `diff_check` gate.
  Files: `selfhost/src/check_types.xi`, `check_state.xi`, `check_core.xi`,
  `check_expr.xi`, rewritten `checker.xi` (canonical dump + `check_count`
  shim that fails only on hard errors); `selfhost/tests/check_negative/`
  manifest (16 cases, `.expected` is the source of truth for BOTH drivers);
  `crates/xiom-codegen/tests/full_diff_tests.rs::diff_check`.
  Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_check` ->
  `83 files (5 diagnostic lines, non-vacuous) + 16 manifest cases`, 1 passed
  (279.1 s). Post-port regressions: `diff_tokens` ok (101.1 s), `diff_ast`
  ok (108.1 s), T1 `diff_corpus` ok (148.2 s). T2 remains unreachable
  (Phase 0 stub emitter; failed at IR line 3 on every file, pre-existing).
  Finding filed: `docs/COMPILER_BUGS.md` 2026-10-02 Phase 3 --
  `NkAssign(l, r)` destructure read `r` as pointer bits (crash 0xC0000005);
  one-arm accessor helpers work, so the port uses side-helper field access +
  Int-tag statement dispatch (same class as Phase 2 (h)).
  Checklist: `docs/checklists/selfhost-phase3.md`. Deferred sub-stages
  (catalog/imports, container method sets, unknown-method/struct-field
  validation, W000/W004/W006/W007 lints, borrow pass) keep permissive `_`
  fallbacks; the bootstrap meter stays 3 of 11 until full checker parity.
- Stage B verification (2026-10-02): branch rebased onto `5837cec7` (main
  landed m171/m172/m174 = Phase 2 findings (f) and (g); (e)/(h) still open;
  rebase conflict only in COMPILER_BUGS.md, resolved keeping main's newest
  sections above the Phase 3 finding). Post-rebase re-runs on this box:
  `diff_check` ok (83 files + 16 cases, 201.8 s), `diff_tokens` ok (99.3 s),
  `diff_ast` ok (88.8 s), T1 `diff_corpus` ok (128.7 s), `cargo test -p xiom
  --bin xiom` 6/6, `cargo test -p xiom --lib dump_check` 1/1 (142.3 s).
  T2 remains unreachable (pre-existing stub emitter).
- Stage B2 (2026-10-03): catalog/imports resolution (Phase 3 sub-stage 1).
  New `selfhost/src/check_modules.xi` loads imported module sources (stdlib
  shapes + local files + the 19-entry static relocation table for modules
  whose declared dotted name does not match their path, e.g.
  `xiom.path` -> `stdlib/xiom/os/path.xi`), registers pub fns/types/consts
  and extern fns under their dotted keys, binds `use` aliases only when the
  file's declared `module` matches (R49-1), and tracks in-program module
  names so `pipeline.fn(...)` resolves like Rust. `check_expr.xi` resolves
  module/member paths for calls, fields, receivers and the `xiom.` namespace
  root; bare lowercase unknowns in `use` files error like Rust (uppercase
  type-ish names stay permissive). 18 new manifest cases under
  `selfhost/tests/check_negative/catalog/` (fixtures dmod.xi/dmod2.xi).
  Gate: `diff_check` -> 83 corpus files (5 diagnostic lines, non-vacuous) +
  34 manifest cases, 1 passed (255.4 s). `diff_tokens`/`diff_ast`/T1 +
  `runtime_ffi_selfcheck` re-run green in the same session.
  Finding filed: `io.list_dir` returns pointer bits instead of names
  (COMPILER_BUGS 2026-10-03), blocking a dynamic stdlib header index; the
  static relocation table is the staged workaround. Also observed once:
  whole-program `-o` builds of the selfhost intermittently exit `-1` with no
  diagnostics while a stdlib smoke lane runs on the shared box (not
  reproducible in isolation afterwards; no COMPILER_BUGS entry without a
  clean repro).
- Stage B3 (2026-10-03): container method sets (Phase 3 sub-stage 2).
  `cm_register_builtin_fns` ports Rust's `register_builtins` table
  (Vec.new/with_capacity/push/len/as_ptr/as_mut_ptr/pop/sort/insert/remove/
  clear/is_empty, Slice pointers, Map.new/Set.new, sizeof/align_of/type_id/
  field_offset/is_signed/to_float/to_int/to_int_from_char/to_char/
  unreachable/panic). Instance dispatch now resolves builtin keys, catalog
  extension keys (`module.Type.method` suffix) and the R8 free-fn UFCS scan;
  the param-offset table matches Rust exactly (arity-direct
  receiver-passed-explicitly offset 0 -> `v.push(1,2)` yields Rust's
  `argument 1 type mismatch: expected Vec, found Int`). Container ctor
  results keep their type arguments (`Vec[Vec[Int]]`) and `ce_check_index`
  synthesizes them from the parsed type expression so nested indexing stays
  typed. 5 new manifest cases under
  `selfhost/tests/check_negative/containers/`. Gate: `diff_check` -> 83
  corpus files (5 diagnostic lines) + 39 manifest cases, 1 passed (219.4 s).
- Stage B4 (2026-10-03): unknown-method + struct-literal field validation
  (Phase 3 sub-stage 3). The Rust method-resolution machinery is now ported:
  the `methods` map (receiver-style fns register module-private too; impl
  members register from catalog files), unique-candidate wildcard capture
  (AUDIT #6: `PathBuf.join` -> `Path.join`), interface members
  (`interfaces` map with `want_of` arity + declared returns), the R8 free-fn
  UFCS scan, and Rust's primitive/erased-container method tables. Unknown
  methods now report `cannot call 'X' on this expression` for user types,
  containers, primitives, Option/Result and statics; struct literals report
  `type 'T' has no field 'f'` and `field 'f' type mismatch: expected E,
  found F`. Generic-param receivers stay permissive via per-function
  `cur_generics` (user single-letter types like `P` error like Rust). 11 new
  manifest cases under `selfhost/tests/check_negative/methods/`. Gate:
  `diff_check` -> 83 corpus files + 50 manifest cases, 1 passed (219.4 s).
- Stage B5 (2026-10-03): lint parity (Phase 3 sub-stage 4). W004
  (unreachable match arms: shadow keys for literals/None/Some/Ok/Err/
  variants, catch-all shadowing), W000 (non-exhaustive NAMED user enums;
  Option[..]/Result[..] carry args and Bool is a scalar, so neither warns;
  bare variant arms parse as catch-all Idents), W006 (literal shift amount
  outside the left type's bit width), W007 (self-comparison on non-float
  types) are ported with Rust's spans and messages. Rust's multi-missing
  W000 order is HashMap-random (filed COMPILER_BUGS 2026-10-03); the port
  uses registration order and gates single-missing cases. 8 new manifest
  cases under `selfhost/tests/check_negative/lints/`. Rebase onto main
  `95c7de1c` (m178 + wave-58 relay) was clean first. All five gates green:
  diff_corpus T1 ok, diff_tokens ok (19,245 tokens), diff_ast ok (12,877
  nodes), diff_check ok (83 files, 5 diagnostic lines, +58 manifest cases,
  279.3 s), runtime_ffi_selfcheck ok.
- Stage B6 (2026-10-03): borrow-pass parity (Phase 3 sub-stage 5). The Rust
  canonical `--dump-check` now runs `BorrowChecker` on the type-check success
  path (non-strict E001 warnings, exactly like `compile()`); the selfhost
  ports the lexical ownership walk + place model + loan set
  (`selfhost/src/check_borrow.xi`, flat-scope storage, places kept in
  `place_display` spelling). Corpus diagnostics grew from 5 to 11 lines
  (E001 on m37_bug45/46/f128) and both drivers agree line-exact with all 58
  manifest cases. `NkAssign` destructure accessor workaround reapplied in
  `bc_stmt_assign` (crash repro bf5.xi `p.y = 3;`, COMPILER_BUGS extended).
  All five gates green together: `cargo test -p xiom-codegen --test
  full_diff_tests -- --nocapture` -> 5 passed, 326.8s (diff_corpus T1,
  diff_tokens 19,245, diff_ast 12,877, diff_check 83 files 11 diagnostic
  lines + 58 manifest cases, runtime_ffi_selfcheck). Meter stays 3 of 11:
  full parity still needs catalog-BODY checking and uppercase bare-name
  resolution (both documented in the checklist).
- Stage B7 (2026-10-03): uppercase bare-name resolution (fallback b).
  Module types/enums/consts now register Rust's BARE fallback keys
  (`use xiom.collections;` makes `Map` resolvable bare), and unresolved
  names in `use` files error for every case instead of only lowercase
  (value: one error at the ident span; call: undefined + Unit-cascade).
  Interface-name receivers (`Eq[T].eq`) stay permissive for now
  (associated-form dispatch deferred, m37_bug48). 3 new manifest cases
  (`catalog/uppercase_*`); `diff_check` -> 83 files (11 diagnostic lines) +
  61 manifest cases, 417.3 s; all five gates re-run green. Remaining before
  the meter flip: catalog-BODY checking only.


# CONTINUATION HANDOFF (2026-10-02 (11), v0.62.2 shipped; m167/m168 fixed; phased queue)

Supersedes the (10) kickoff prompt below (kept as history).

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM swarm compiler lane. Read this top section, then
> `docs/COMPILER_BUGS.md` (newest entries first), `docs/SELFHOST_PROGRESS.md`,
> and `docs/RELEASE_GATE_v0.62.2.md`. Bootstrap meter on main is **18%**
> (Phase 1 lexer parity merged at `0831ee6f`; `diff_tokens` byte-equality
> gate green over the 83-file corpus). The merged driver changes
> (`crates/xiom/src/main.rs` --dump-tokens path) have now had a full e2e on
> the merged state (`aca6aaee`, 2026-10-02: e2e 2395/2395 +4 ignored at 8
> threads with the debug driver rebuilt from HEAD); the next compiler batch
> carries only its own changes.
> State: **v0.62.2 RELEASED** (tag on `801d888f`, run 36745108730, docs
> dispatched). Compiler lane fixes landed after it: m166 method trust key
> (`185342f4`), m168 `&mut T` write-through (`7b38fa87`), m167 global Vec
> fast path (+ locks; committed with this handoff). All gates green at the
> handoff commit unless noted.
> QUEUE, in order:
> 1. [DONE 2026-10-02] Merged the selfhost Phase 1 branch
>    (`selfhost-phase-1-lexer`, rebased to `0831ee6f` on main): token-dump
>    gate green, CLI locks green. The branch's SESSION.md edits were
>    dropped in the rebase (main's handoff kept).
> 2. [DONE 2026-10-02] C24-1 FIXED (m169): same-leaf qualified results
>    (`vector.lerp/clamp/hadamard`, `curves.b_spline`) lost the Vec element
>    type at module-qualified call sites; caller reads took the scalar i64
>    path (stored double bits surfaced via sitofp). Fix in codegen
>    `callee_return_xiom` (resolve the callee key exactly as call emission
>    does, then read the declared return type). Locks: m169 fixture + e2e +
>    IR + CI line. Full e2e runs once at the end of this compiler batch.
> 3. [DONE 2026-10-02] C24-2 FIXED (m170 + m170b): calls through fn-typed
>    params kept no return type on the binding (curve_length's sampled reads
>    took the scalar i64 path; `type_from_ast` also dropped the Vec args in
>    the non-mono registration), and the erased Option/Result literal payload
>    slot was widened by a stale type_meta type (Option-of-Vec `.unwrap()`
>    read garbage). Locks: m170 fixture + e2e + IR + CI line; feature-reg,
>    stdlib-exec and api-freeze green. Full e2e at batch end.
> 4. [NOT REPRODUCED 2026-10-02] expat/nbt silent exit: port.ps1's `-1` is
>    its own watchdog timeout sentinel (not a program exit code); the empty
>    stdout is the file-redirected child's unflushed buffer after the kill.
>    Exact harness re-run on this box with the deployed v0.62.2 (same 9/30
>    binary) against stdlib main AND stdlib-perf1 (clean worktree 06d0ee7):
>    expat 25/25 PASS exit 0 in 18.2 s; nbt 26/26 PASS (t5 included) exit 0;
>    2.8 KB file-redirect probe writes fully. If the packages lane still
>    reproduces: preserve %TEMP%\xiom-run-*.out/.err and report their
>    packages commit + duration (their repro likely predates wave-49).
> 5. [DONE 2026-10-02] m168 RESIDUAL FIXED (m168b): the deref helper was
>    only consulted inside the pointer-param branch and stripped only the
>    `*T` form, while `local_xiom_types` records `&mut T` (ref-preserving).
>    Hoisted the check for non-pointer params and accept `&mut `/`&`/`*`
>    prefixes. Probes rc 2 -> 0; locks m168_mut_ref_byval_arg fixture + e2e
>    + IR + CI line.
> 6. [DONE 2026-10-02] C23 FIXED: the driver optimized twice (`opt -O<level>`
>    then `clang -O<level>`); on LLVM 18 hosts the second stage miscompiled
>    the l6-15/l7-39/l8-09 lesson shapes. Kept `opt -passes=verify`, dropped
>    the redundant pre-optimization (clang is the only optimizer now).
>    Verified: pack acceptance `C23 present: no` with a HEAD Linux driver;
>    three fixtures (l6/l8 discriminate pre-fix on the compile path; l7 is
>    shape coverage) + e2e + CI line. Windows has no `opt` -> unaffected.
> 7. Registry polish (optional, post-release): B1 packaging guard, B2
>    `xiom pkg yank`, install trust wording, `--resolve` outside a
>    workspace (registry relay).
> 8. stdlib `Vec.push[T]` stride fix (relay in COMPILER_BUGS m167): the
>    generic body hardcodes 8-byte stride + unscaled `data + len`; the
>    compiler now avoids it for direct receivers, but fix it at the source.
> 9. Packages type-laxness intake (COMPILER_BUGS 2026-10-02 relay):
>    `let gb: Vec[UInt8] = got.value;` with a Str field compiles clean and
>    yields wrong bytes. Reproduce, then scope (post-batch, non-release).
> 10. Top-level `module` header breaks nested-module cross-type field
>    compares (COMPILER_BUGS 2026-10-02 relay): l6-15 + `module <name>` ->
>    one T001 at `note.category == cat`; without the header it compiles.
>    Post-batch intake.
> BATCH GATES (2026-10-02, m169+m170+m168b; commits 3aad01cd -> 4e084e0e
> -> 565b9924): full e2e 2401/2401 (+4 ignored, 8 threads), feature-reg
> 517/517, parser 107/107, checker_locks 23/23, fuzz 24/24, perf 3/3,
> robustness 63/63, stdlib-exec 85/85 (+2 ign), api-freeze 2/2 (now
> PID-unique temp), selfhost diff_tests 24/24 (+1 ign), doctor_cli 4/4,
> run_script_cli 2/2, borrow_e001 2/2.
> C23 BATCH (2026-10-02, `8abedb72`): full e2e 2404/2404 (+4 ignored, 8
> threads), feature-reg 517/517, checker 195/195, checker_locks 23/23,
> fuzz 24/24, perf 3/3, robustness 63/63, formatter 86/86, lsp 45/45;
> Linux acceptance `C23 present: no`; website gates line updated to the
> same snapshot. Next in queue: the registry polish / intake items (C23
> and m168b are done); Gate P is ready when `STDLIB_VERSION` bumps to
> `stdlib-perf2`.
> WAITING ON: none blocking. Stdlib `stdlib-perf2` (f011efe) is now on
> stdlib origin main -- it carries the `xiom/sync/sync.xi` `#[unsafe_direct]`
> annotation + `stdlib-perf2` tag, so Gate P is READY: bump
> `STDLIB_VERSION` to `stdlib-perf2`, regenerate the api-freeze snapshot,
> then benchmark t2 (the compiler half is in `185342f4`).
> Next release (v0.62.3) scope: C23 + C24-1/C24-2 + expat/nbt + m168
> residual + t2/sync.xi; gate = local suites at the release commit +
> playground run.sh "C23 present: no" + Gate P + stdlib-perf2 pin + dry-run.
> Method: repro-first (tmp/sprintc), locks (fixture + e2e + CI line), full
> e2e ONCE per compiler batch at 8 threads on this box, ascii_guard before
> commits, commit atomically with SESSION/COMPILER_BUGS, push only on the
> owner's ask. Shared machine: run cargo commands sequentially; never
> rebuild target/debug while e2e runs.

# CONTINUATION HANDOFF (2026-09-29 (10), selfhost Phase 0 landed + m163 fixed -- T1 harness green; m162 filed)

Supersedes the (9) handoff below (kept as history).

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM swarm compiler lane. Read the top section of SESSION.md,
> docs/SELFHOST_PLAN.md and docs/SELFHOST_PROGRESS.md (the bootstrap meter +
> per-phase gate table; currently 9% = Phase 0 of 11 gates) first
> (docs/STAGE6_LINT_WAVE.md for the lint wave state; tier 3 parked).
> State: compiler **v0.62.1 RELEASED**; extension 0.12.1 live on both
> marketplaces (thread closed); **selfhost Phase 0 LANDED** (2026-09-29):
> `cargo test -p xiom-codegen --test full_diff_tests` = 2 passed (T1 green
> over the 84-file corpus, 49.2 s; `runtime_ffi_selfcheck` ok);
> `selfhost/src/` skeleton (main/lexer/parser/checker/codegen/selfcheck/
> runtime_ffi) compiles with `xiom.exe` into `target/selfhost/xiomc-self.exe`
> and runs; v050 archived; 13 stale `_diff_*` temp files deleted. All in the
> handoff commit at the top of this section (previous pushed tips bc047d9c /
> c0c081df; local tip not pushed -- push only on the owner's ask).
> **NEW compiler finding (filed) + fix (landed):**
> (1) **m163 -- SILENT MISCOMPILE, FIXED 2026-09-29**: inside a struct
> METHOD, a `Vec[Str]` field element used as a `+` operand lowered to i64
> add + inttoptr (pointer-decimal garbage / AV). Root cause: the method
> prologue bound receiver fields as locals but never registered their Vec
> ELEMENT types; fix `record_receiver_field_vec_elems` registers them in
> both prologue branches. Locks: `m163_method_field_vec_concat` (e2e + CI
> line) + `regress_m163_field_vec_elem_concat` (IR). Gates: full e2e
> 2390/2390 (+4 ignored, 1517 s), feature-reg 511/511, checker_locks
> 22/22, selfhost diff 2 passed. Repro kept in
> `tmp/sprintc/m163_method_str_accum/` + `tmp/sprintc/m163_single.xi`.
> (2) **m162 FIXED 2026-09-29**: a user module exporting a fn whose LEAF
> matches a stdlib fn (`char_at`) poisoned catalog-body resolution of
> UNRELATED stdlib modules (bogus T001s in `[xiom.num]`) and codegen's
> binding (`@user_util.char_at` inside `@num.parse_int_radix` -> runtime
> "invalid index"). Checker prefers the body's explicit item import
> (recorded into `catalog_resolved_calls`); codegen's
> `resolve_catalog_call_bare` binds that target. Locks:
> `m162_sameleaf_catalog_poison` (e2e + CI line) + checker_locks
> `m162_sameleaf_fn_does_not_poison_catalog_bodies`. Repro:
> `tmp/sprintc/m162_sameleaf_catalog_poison/`.
> Next work, compiler track: **transient `program_exit=-1` capture
> batch**, then playground polish (Range `unknown type 'Iterator'`
> warning; W005 erased-dispatch stub), CI Heavy Suites triage.
> Landed since (10): m164 module-const table materialization, the C22
> driver fix (`xiom run` sibling modules), the m162 same-leaf
> catalog-poison fix (checker + codegen) and the m165 Vec growth-ceiling
> raise (2^24 -> 2^32 elements; >16 MB byte buffers). Held: `XIOM_STRICT_BRACKETS` default flip + the
> 3 stdlib mixed-bracket sites. Parked: loop-CSE retry (needs the porting
> session's pre-fix decoder). Release: **v0.62.2 gate defined in
> `docs/RELEASE_GATE_v0.62.2.md`** (version+notes preconditions, suites
> A-F + Gate P, dry-run before the tag; stdlib sign-off + pin decision and
> the owner's tag push are the external steps). **PERF-1 (t2-queue
> trampoline) is OWNER-REQUIRED for this release**: the compiler half
> landed as m166 (parser attribute order + injected-catalog trust); the
> stdlib atomics annotation, the pin and the benchmark re-run are Gate P.
> **RELEASED 2026-09-30** (tag `v0.62.2` on `801d888f`, run 36745108730
> green, docs dispatched); carries tier-2, R-2/R-2c, byte_at, m162-m166,
> C22; the selfhost binary stays out (100% policy). Post-release the
> packages lane re-verifies `byte_at` (docs/repro/byte-at-128), the
> benchmark lane re-verifies R-2/R-2c, and the repo-local stdlib checkout
> refreshes to
> the tag.
> Next work, selfhost track: **Phase 1 (lexer parity)** -- port
> `crates/xiom-lexer` to `selfhost/src/lexer.xi`, add `--dump-tokens` to
> both compilers, gate on byte-equal dumps over the corpus
> (docs/SELFHOST_PLAN.md section 5). Float `{:.17e}` formatting stays
> deferred to the emitter phase (documented in runtime_ffi.xi).
> Method: repro-first under tmp/sprintc/, locks (tests/regression fixture +
> checker_locks/e2e + CI line where e2e-able), full e2e ONCE per compiler
> batch, python tools/ascii_guard.py check before every commit, commit
> atomically with SESSION.md + COMPILER_BUGS.md evidence. Identity Lefteris
> Notas <lefterisnotas@gmail.com>; pushes only when the owner asks; never
> rebuild target/debug while e2e runs. Run long suites with reduced
> threads on this box: e2e `-- --test-threads 12`, stdlib_tests 8,
> `scripting_tests` 4.

## Cross-lane notes (2026-09-29)

- **Benchmark lane relay (2026-09-30, Gate P NOT accepted on t2)**:
  v0.62.2 t2 = 33,693 ms (full arena 42/42) / 11,968 ms (probe) vs
  rust 14/3 ms; their root cause: call sites still pay the trampoline
  around trusted stdlib fns and legacy `xiom.sync.AtomicInt` is
  unannotated. Compiler lane: confirmed and FIXED the method trust-key
  gap (receiver-qualified key; local proof: legacy path 7000 ms -> 0 ms
  with the two legacy methods annotated; locks + full e2e 2393/2393).
  RELAY to stdlib: annotate `stdlib/xiom/sync/sync.xi` unsafe-block fns
  (AtomicInt methods + standalone `atomic_*` helpers; consider
  `cdl_wait_spin`), tag `stdlib-perf2`; then the benchmark re-runs --
  acceptance moves to the follow-up release (v0.62.3).
- **Packages lane relay (2026-09-30, v0.62.2 re-pin)**: byte_at battery
  `bad=0` (workaround retired), 435 records re-pinned. Fleet sweep found 2
  REAL regressions among ready/published packages: `xiom.expat` and
  `xiom.nbt` exit `-1` with NO stdout; adding `io.flush_stdout()` after
  every println makes expat pass 25/25 and nbt 25/26 (one genuine nbt
  check failure remains: u16 length-prefix/UTF-8 names). Both were green
  on v0.62.1 and reproduce running the compiled a.exe directly -- a
  v0.62.2 codegen/exit regression, not the byte_at workaround (their
  `%TEMP%\kilo\sweep-v0622*` logs + COMPILER-FINDINGS.md 2026-09-30).
  QUEUED as the next bug batch: bisect the silent-exit/stdout-flush
  regression (candidates: m163/m164/m165 codegen; check exit + flush
  paths).
- **Playground relay (C23, OPEN -- partly reproduced)**: v0.62.1/0.62.2
  miscompile at -O2 (the default level) -- three lesson solutions give
  silently wrong answers at -O2, correct at -O0, host-LLVM-dependent.
  Repro pack landed: `E:\xiom-lang\playground\tools\compiler-repros\c23\`
  (byte-exact solutions + README + `run.sh` acceptance = "C23 present:
  no"). REPRODUCED EXACTLY in WSL Ubuntu (clang 18.1.3 (1ubuntu1), the
  reporter host) with the v0.62.2 linux toolchain extracted at
  `~/xiom-c23`: l6-15 2/3, l7-39 2/3, l8-09 2/0. Level bracket (l7-39):
  O0 2 / O1 2 / O2 3 / O3 3; instrumentation hides it; the reviewed IR
  (esz=24 Vec[TodoItem], 24-byte .get box + .set memcpy) is structurally
  correct so far. Full details + next steps in `COMPILER_BUGS.md`
  "2026-10-01 -- OPEN (C23)"; pass-level bisection is the next dedicated
  batch.
- **Packages lane relay (2026-10-01, repros delivered)**: both v0.62.2
  regressions have ready repros at their commit `2d91399`,
  `docs/repro/v0622-regressions/`:
  (a) `Vec[Str].push` trigger = a MODULE-LEVEL `var v: Vec[Str]` global
  (a local Vec with the same pushes is fine); REPRODUCED locally -- filed
  as m167 in COMPILER_BUGS (bad `store i8 <handle>, i8* dest` + scalar
  `v[0]` read -> clang rejects the IR; same class as m163's field gap,
  now for globals).
  (b) expat/nbt: the registry artifacts include full sources/tests
  (`.../packages/xiom.expat/0.1.1/download`, `.../xiom.nbt/...`; local
  `packages/xiom-expat/tests/test_conformance.xi`); run
  `scripts/port.ps1 -Package xiom.expat` -> silent exit -1, zero stdout;
  `io.flush_stdout()` after each println makes expat 25/25 and nbt 25/26
  (t5 UTF-8 strings is a genuine failure) -- suspected exit/flush path.
  (c) FOURTH v0.62.2 issue filed: `&mut Int` write-drop (from xiom.svm),
  silent wrong results; repro details requested.
  **m168 primary FIXED** (mut_ref_params + through-store in stmt.rs +
  coerce.rs arms): `set99`/`&mut Vec[Int]` whole-assign/`&mut Str` all
  correct; locks + IR lock + CI line; gates e2e 2394/2394 (+4 ignored),
  feature-reg 516, parser 107, CLI locks, selfhost diff 2. RESIDUAL:
  `&mut T` -> by-value param in a DIRECT call still ptrtoints (sibling
  `bump`/`byval` shape) -- direct-call arg pipeline fix queued
  (COMPILER_BUGS m168 RESIDUAL).
  Wave 46 published (eco-v0.1.27); wave-47 candidates in their SESSION.
- **Registry lane relay (optional polish, post-release)**: B1 packaging
  guard, B2 `xiom pkg yank`, install trust wording, `--resolve` outside a
  workspace. Queued after the release blockers.
- **Registry lane relay (post-v0.62.2, QUEUED)**: registry 2.7.0 is live;
  two xiom-pkg-only changes queued AFTER the release ships (registry is
  not touching this repo meanwhile). (C5) Pin + verify the registry index
  digest: TrustStore gains `index_keys: BTreeMap<String,String>`
  (serde-default so legacy trust files load; normalize like `keys`),
  `get_index_key`/`pin_index` (hex validation like `pin`); `trust
  --registry <URL> --index-key <hex>` (mutually exclusive with `--key`);
  `trusted` lists index keys; pure
  `verify_index_digest(index_bytes, digest_json, pinned_key)` with the
  four checks (sha256, byte length, public-key equality, ed25519 over
  "xiom-index-digest:v1\n" + lowercase sha256 hex) called in
  `fetch_registry_index` right after the fetch and BEFORE caching; fail
  closed when a key is pinned, skip the digest fetch entirely when not
  (staging is unsigned). Do NOT reuse the artifact `keys` map (official
  artifacts use per-run OIDC keys). Production pin: key
  `f76f5ff51538ce757454864494b74eae5424ce9ae6eb33689aa31ffe6d059673`,
  fingerprint `f7:6f:5f:f5:15:38:ce:75`. Tests: legacy trust file loads;
  accepts a KeyPair-signed payload; rejects wrong key / tampered bytes /
  missing signature. (B3) `publish --dry-run` posts the same multipart to
  POST `{registry}/validate` (no writes) and prints the JSON incl.
  warnings; documented in `PUBLISHING.md` (registry commit 33c3d56).
  Background: registry SESSION.md section 24.
- **Playground ack**: C22 closed; they delete `stageForRun` at the
  v0.62.2 pin bump and re-run the package tests; their production package
  runs are 5-7.5 s vs a 30 s timeout, so the temp-dir churn note does not
  apply in their container.
- **Release gate v0.62.2 (planned)**: `docs/RELEASE_GATE_v0.62.2.md` --
  version+notes preconditions (workspace 0.62.2, `SELFHOST_VERSION`,
  `release-notes/v0.62.2.{md,json}` BEFORE the tag, `STDLIB_VERSION`
  pin), compiler-lane suite gates (all green as of the last runs),
  stdlib-lane sign-off, packages rehearsal, a `workflow_dispatch`
  dry-run before tagging, and the tag/publish/post-release steps.
  **v0.62.2 RELEASED 2026-09-30.** stdlib delivered `stdlib-perf1`
  (`06d0ee7`, all 16 atomics wrappers annotated); `STDLIB_VERSION`
  updated; repo-local checkout refreshed to the pin; release notes
  re-converted WITH the stdlib fragment (6 highlights, verify green);
  versions bumped; all local gates green at the release state (e2e
  2393/2393 +4 ignored at 8 threads, stdlib_tests 40/40, stdlib_execution
  85 +2 ignored, api-freeze 2/2, feature-reg 514/514, parser 107/107, CLI
  locks, selfhost diff 2). Dry run 36707354989 green on all legs (the
  first, 36705942509, caught the `xiom-mcp` E0063 -- fixed `837f8e6f`).
  **Tag `v0.62.2` on `801d888f`; release run 36745108730 GREEN**; release
  page https://github.com/xiom-lang/xiom/releases/tag/v0.62.2 carries
  SHA256SUMS + lin/win/macos archives + VSIX 0.12.1 (marketplace skip
  expected) + wasm + glue; docs dispatch delivered to `xiom-lang/website`
  (tag=v0.62.2, stdlib_ref=stdlib-perf1). Tag is lightweight (prior tags
  annotated) -- do not retag. Post-release: benchmark re-run (Gate P:
  t2-queue ms range), packages re-pin, ops mirror refresh; registry lane
  C5/B3 unblocked (queued below).
- **m166 FIXED (PERF-1, owner-required for v0.62.2)**: `#[unsafe_direct]`
  written above `pub fn` was a P001 that error recovery absorbed -- the
  attribute was silently dropped -- AND stdlib fns compiled inside a user
  program were never trusted (the check keyed off the primary source path).
  Fixed in the parser (`pub` accepted after attributes) and codegen
  (`catalog_fn_keys` handed by the driver at injection). Local proof with
  the annotation applied to the two wrappers: 4M atomic pairs
  8000 ms -> 0 ms; IR shows no trampoline/guard arm in
  `@sync.atomic_load`/`atomic_store`. Locks: parser unit test
  `test_attribute_before_pub_fn` +
  `regress_m166_unsafe_direct_pub_fn_trusted`. Gates: e2e 2393/2393
  (+4 ignored), feature-reg 514/514, parser 107/107, checker_locks 23/23 +
  CLI locks, selfhost diff 2. **Relay to stdlib**: annotate every fn in
  `stdlib/xiom/sync/atomics.xi` whose body contains an `unsafe` block with
  `#[unsafe_direct]` (load/store verified locally; same single-intrinsic
  shape); requires compiler >= this commit; tag for v0.62.2 and update
  `STDLIB_VERSION`. Then the benchmark lane re-runs t2-queue -- the Gate P
  release acceptance.
- **Benchmark lane relay (PERF-1, in progress)**: t2-queue is ~1450x Rust
  (50,787 ms vs 35 ms) while XIOM is normal on every other task; isolation
  shows 4M `AtomicInt` load/store pairs at 18.5 s vs 4 ms plain, with every
  unsafe-block call paying `xiom_trampoline_call` + guard-page arm/disarm
  (~2.4 us/call). Full provenance (session `run_1790700620309`, exports,
  evidence dirs), root-cause hypothesis, candidate fixes and checklist are
  in `docs/STAGE6_PERF_PLAN.md` (PERF-1). **Local repro landed**
  (`docs/repro/perf-1-atomic-trampoline/`): plain ~0 ms vs atomic 8000 ms
  for 4M pairs (~1.0 us/call; same order as the container); IR shows the
  per-CALL shape (`xiom_trampoline_call(@__unsafe_block_N)` plus
  `guard_heap_enter/exit` + `guard_page_arm/disarm` + `trap_leave` around
  each `xiom_atomic_*`). Next: design decision (block-level arming vs
  callee classification) with safety-lane sign-off on retry semantics,
  then implement + perf-budget lock + benchmark re-run.
- **Website fetch**: `docs/SELFHOST_PROGRESS.md` is live on `main`
  (push `c0c081df..ad63eda0`; raw URL verified). A plain commit push
  triggers no workflows in this repo (CI is PR-only, `release.yml` is
  tag-only, `vscode-publish`/`heavy` are `workflow_dispatch`), so the
  tracker is fetchable without cutting any release.
- **Packages lane relay**: pin moved 0.61.3 -> 0.62.1 mid-batch; every
  suite re-run clean, no code changes needed for the new compiler. Their
  sectest caught two PACKAGE-side catalog bugs (obs-fold detection was
  masked by trimming -- leading SP/TAB is now strictly a folded header
  line; `max-age=abc` reported as "without max-age" instead of "is not a
  number"). They used `xiom.string.str_replace_all` to isolate one policy
  rule by rewriting a hardened baseline, and avoided `==` on Result
  values (no guaranteed Eq) via small typed helpers. No compiler/stdlib
  action items from this relay; if Result/Option structural equality is
  wanted, it is a stdlib design decision (not filed).

## Status snapshot (2026-09-29 (10))

- **Selfhost Phase 0 LANDED** (handoff commit): harness rewritten in
  `crates/xiom-codegen/tests/full_diff_tests.rs` -- deterministic 84-file
  corpus manifest (tests/regression m33_z14 + m34_d01 + m37_* x49;
  examples/phase1_* x16, catfix x7, diff_test, demo_float, stress_* x5,
  benchmark_selfhost, stress_body_parser; stdlib/tests/smoke/
  smoke_guard_fault.xi), T1/T2/T3 tiers selected by
  `XIOM_SELFHOST_DIFF_TIER=1|2|3` (default 1, tiers stack), runner
  compiles `selfhost/src/main.xi` once per test process into
  `target/selfhost/xiomc-self.exe` and passes the source path as argv[1]
  (replaces the v10 temp-source-patch runner; no repo-root temp files).
  Skeleton: `main.xi` driver (args -> read -> lexer -> parser -> checker ->
  codegen, `--selfcheck` mode), lexer/parser/checker stubs, codegen stub
  emitting a well-formed header + `@main` module. `runtime_ffi.xi` ports
  `rt_str_len`/`rt_char_at`/`rt_str_slice` (exact C semantics incl. UTF-8
  decode and clamps), `SymbolTable` (1-based intern/lookup), `FnTable`,
  `IrBuffer`; `--selfcheck` asserts the C outputs and prints SELFCHECK OK.
  v050 -> `selfhost/archive/xiomc_v050.xi`; 13 stale
  `selfhost/_diff_*_N.xi` temp files deleted. Checklist corrections:
  `examples/stdlib_smoke/` never existed (fixture is in the stdlib
  checkout); `selfhost/_diff_phase1_*` were v10 temp copies, not corpus.
  Gate: `cargo test -p xiom-codegen --test full_diff_tests` -> **2 passed**
  (`diff_corpus` T1 over 84 files in 49.2 s + `runtime_ffi_selfcheck`).
- **Selfhost progress tracker**: `docs/SELFHOST_PROGRESS.md` -- bootstrap
  meter (9%: 1/11 gates), per-phase gate table, Phase 0 evidence, open
  blockers (m162), gate commands. Update the meter line whenever a
  gate flips.
- **m163 FIXED (silent miscompile)**: struct-method field `Vec[Str]` element
  as a `+` operand became i64 add + inttoptr; fix registers field Vec
  element types in the method prologue (`record_receiver_field_vec_elems`,
  both branches). Locks: `tests/regression/m163_method_field_vec_concat/
  main.xi` + `e2e_m163_method_field_vec_concat` (CI line) +
  `regress_m163_field_vec_elem_concat` (IR). Gates: full e2e 2390/2390
  (+4 ignored), feature-reg 511/511, checker_locks 22/22, selfhost diff
  2 passed. COMPILER_BUGS entry flipped to FIXED; repro in
  `tmp/sprintc/m163_method_str_accum/`.
- **m162 FIXED (same-leaf catalog poison)**: a user module exporting a
  same-leaf fn (`char_at`) made `xiom.num`'s body resolve the USER's
  signature (bogus T001s) and codegen bind `@user_util.char_at` inside
  `@num.parse_int_radix` (runtime "invalid index"). Checker: catalog-body
  bare calls now prefer the body's EXPLICIT ITEM import (provenance:
  `local_module_paths[leaf]` ends with the leaf -- module-surface
  injections excluded, which keeps `xiom.math.rounding`'s `pow` overload)
  and record the dotted target into `catalog_resolved_calls` (R20 owner
  key). Codegen: `resolve_catalog_call_bare` binds `catalog_call_targets`
  before `bare_fn_aliases`. Locks: `m162_sameleaf_catalog_poison` e2e + CI
  line + checker_locks `m162_sameleaf_fn_does_not_poison_catalog_bodies`.
  Gates: full e2e 2392/2392 (+4 ignored, 2167 s under the smoke battery),
  feature-reg 512/512, checker_locks 23/23, selfhost diff 2 passed.
  `COMPILER_BUGS.md` entry added; repro kept in
  `tmp/sprintc/m162_sameleaf_catalog_poison/`. The selfhost `rt_` prefixes
  are no longer required (kept; rename in O1).
- **m164 LANDED (module-const table materialization)**: immutable
  all-literal integer const arrays now emit ONE `internal constant
  [N x i64]` global per table (`module_const_defs`) and index reads GEP it
  (`const_array_globals` interception in the Index arm, BEFORE container
  compilation), replacing the per-use alloca + N+1 stores that rebuilt the
  whole table at every reference (packages row 25). i64 slots preserve the
  old buffer read semantics bit-for-bit (UInt8 high-bit zext, Int8
  negatives). Scope: integer-like elements only; float/Str/struct and
  zero-length arrays keep the substitution path, and const-array `.len()`
  is a pre-existing limitation that did not change. Locks:
  `m164_const_table_global` (e2e + CI line) +
  `regress_m164_const_array_global` (IR: `internal constant [8 x i64]`,
  no N+1-slot alloca). Gates: full e2e 2391/2391 (+4 ignored, 1872 s),
  feature-reg 512/512, checker_locks 22/22, selfhost diff 2 passed.
- **m165 LANDED (Vec growth ceiling)**: the growth guard trapped when the
  next doubling exceeded 2^24 ELEMENTS (a `Vec[UInt8]` byte buffer died at
  16 MB -- a 20 MB file could not be buffered; the old comment mislabeled
  it 2^20). The ceiling is now 2^32 elements (`call.rs`), keeping
  `new_cap * esz` far below i64 overflow for sane element sizes; the
  realloc null-check remains the OOM trap. Locks:
  `m165_vec_byte_buffer_gt_16mb` (e2e + CI line; pushes 16,777,218 bytes
  and re-checks the boundary) + `regress_m165_vec_growth_ceiling` (IR: the
  guard uses 4294967296, the old constant is gone). Gates: full e2e
  2393/2393 (+4 ignored, 1813 s), feature-reg 513/513, checker_locks
  23/23, selfhost diff 2 passed.
- **C22 (playground relay) FIXED**: `xiom run <script>` now hands the
  checker the script's real directory (parent + guarded grandparent,
  mirroring the compile path) through `CompileConfig.extra_source_dirs`;
  the `%TEMP%/xiom_run` temp copy previously meant sibling modules never
  reached the catalog (`--check` worked, `run` failed with undefined
  variable). Wired for both pipelines + the `--watch` path. Lock:
  `tests/regression/c22_run_sibling_module/` + `run_script_cli` CLI suite
  (CI line). **Playground lane: the `stageForRun` bridge can be deleted.**
  Local note: this box's Defender blocks freshly built `%TEMP%/xiom_run`
  exes (os error 225); the lock asserts resolution+codegen (`compiled:`)
  and tolerates the execution block locally (CI Linux takes the full path).
  **Extended verification (packages shape, 2026-09-29 pm)**: the
  playground's exact repro -- `<dir>/main.xi` importing
  `<dir>/packages/xiom-hello/src/hello.xi` (module `xiom.hello`) -- now
  compiles AND runs on the fixed build (`compiled:` + exit code 0; manual
  probe `tmp/sprintc/c22_packages/`), while v0.62.1 still fails as
  reported. The permanent lock now covers a package layout too
  (`packages/c22-pkg/src/hello.xi` -> `c22pkg.hello`). Loaded runs can
  take minutes (catalog churn over stale temp stdlib copies, W001 noise);
  that is slowness, not a deadlock (completed <180 s with `--no-cache`
  under the smoke battery).
- **Coordination note (2026-09-29 evening)**: the compiler lane's
  `cargo test -p xiom` overlapped with the stdlib smoke battery
  (`%TEMP%\xiom-smokes-20260929-191201`, 8 `xiom_v0613` workers, started
  19:12). Three workers were stopped by mistake before the overlap was
  identified -- if that battery shows failures for smoke_net_http2 /
  smoke_test3 / smoke_stress_string_upper_lower / smoke_stress_io_rename /
  smoke_hash2 / smoke_cross_io_convert_fmt / smoke_cmp_reverse /
  smoke_stress_compress_gzip_levels, re-run those before trusting the
  result. The battery itself was left untouched afterwards; the compiler
  lane's suites should not be run concurrently with it on this box.
- **Git**: local `main` tips include Phase 0 `e813449b`, tracker
  `532bfa75`, m162-m166 (`d8a04678`, `d0189157`, `8258c400`, `ad63eda0`,
  `c2b15112`), C22 `563aaff2` + packages lock `d51b4ba6`, release notes
  `0db99183`, release commit `eec38f34`, mcp fix `837f8e6f`, gate doc
  `801d888f` -- all pushed to `origin/main`; tag `v0.62.2` pushed and
  released. Push only on the owner's ask.

# CONTINUATION HANDOFF (2026-09-29 (9), compiler lane -- v0.62.1; all batches pushed; extension 0.12.1 live; selfhost Phase 0 green-lit)

Supersedes the R-5 batch handoff below (kept as history).

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM swarm compiler lane. Read the top section of SESSION.md,
> docs/STAGE6_LINT_WAVE.md and docs/SELFHOST_PLAN.md first.
> State: compiler **v0.62.1 RELEASED**; all landed batches are pushed
> (main == origin; prior tips `bc047d9c`, `72e42ac2`, `19c611c0` plus this
> handoff commit); tree clean. The VS Code extension **0.12.1 is PUBLISHED
> and VERIFIED**: both marketplaces live (publish run 36578182481; Open VSX
> 2026-09-29T13:51:48Z, VS Code Marketplace 13:59:06Z), the v0.62.1
> release VSIX was swapped to `xiom-vscode-0.12.1.vsix` (72,534 B, sha256
> `ab1de7f967d615bfd45b3fdd39da2e178404cee900dbe99f60c3383137ad3430`;
> `SHA256SUMS` re-uploaded; download-back verified), and dl.xiom-lang.org
> now serves it (ops fixed `dl-deploy.sh` with an asset-list fingerprint;
> `dl-verify.sh` green) -- extension thread CLOSED, owner confirmed the
> sites and the VS Code auto-update. stdlib pin stays `stdlib-v0.62.0`
> (tag force-updated to `0e63101`; repo-local checkout still `80e767b` --
> refresh at the next release step). Run long suites with reduced threads
> on this box: e2e `-- --test-threads 12`, stdlib_tests 8,
> `scripting_tests` 4 (parallel JIT tests are the known load-flake class;
> never record a silent failure as a pass without a re-run; concurrent
> stdlib smoke batteries roughly double suite wall-times).
> Owner decisions (2026-09-29): (1) **selfhost is GREEN-LIT as a parallel
> track** -- lay the Phase 0 ground now while the compiler lane burns the
> backlog; Phases 0-2 (harness, lexer, parser) may proceed immediately,
> Phase 3+ (checker/codegen parity) should follow a behavior freeze;
> (2) **release policy: selfhost ships only at 100% bootstrap** (self1 ==
> self2 sha256 + T3 byte-identical IR + full suite) -- until then every
> release is the Rust-hosted compiler; (3) Stage 6 tier 3 stays parked
> pending the owner's noise-budget call.
> Landed since the R-5 handoff (do not redo): PR #4, C001, v0.62.1, Stage 6
> W004 + tier 2 (`75623b4f`: W008/W006/W007 + `warn_coded_at` dedupe),
> R-2d, R-2c + R-2 partial (`1814ac36`: `hint_is_vec_or_slice`,
> `local_opt_payload_xiom` fallback, inline match-payload aliasing +
> in-place Option/Result local matching), byte_at i8 widening
> (`f4af5f64`: zext-default i8, `widen_operand_to_i64`,
> `expr_int_signedness`), vscode 0.12.1 (`19c611c0`).
> Next work, compiler track (in order): module-level const table
> materialization (perf/code-size; packages row 25: merkle rebuilds its 64
> K constants per hash, l10n lowers its ISO table to comparison chains;
> IMMUTABLE consts substitute at each use site via `local.constants` while
> mutable module `var` already emits real globals -- emit `internal
> constant` globals for const ARRAYS / static-data + header for const
> Vecs + a GEP-the-global read path; needs a fresh batch with locks),
> same-name fn shadowing (no redefinition diagnostic), `Vec` ~2^24 cap,
> transient `program_exit=-1` capture batch, playground polish (Range
> `unknown type 'Iterator'` warning; W005 erased-dispatch stub), CI Heavy
> Suites triage; held: `XIOM_STRICT_BRACKETS` default flip + the 3 stdlib
> mixed-bracket sites with the next stdlib wave; parked: loop-CSE retry
> (all reductions clean on `f4af5f64`; needs the porting session's pre-fix
> decoder).
> Next work, selfhost track (Phase 0; checklist
> docs/checklists/selfhost-phase0.md): upgrade `full_diff_tests.rs` to
> T1/T2/T3 + a deterministic corpus manifest; create the `selfhost/src/`
> skeleton (main/lexer/parser/checker/codegen stubs) that compiles with
> `xiom.exe`; start `runtime_ffi.xi` (`str_len`/`char_at`/`str_slice` via
> stdlib ops); archive `xiomc_v050.xi`. Gate: T1 green on the corpus; NO
> release coupling. Keep the port 1:1 with the Rust crates' control flow
> -- the T2/T3 tiers exist to catch drift early.
> Release status: **v0.62.2 not cut yet** -- it should carry tier-2, R-2
> and byte_at plus the extension metadata. After it ships: the packages
> lane re-verifies `byte_at` (`docs/repro/byte-at-128`, expect `bad=0`),
> the benchmark lane re-verifies R-2/R-2c and can drop the t5
> `Vec[BTree]` workaround, and the repo-local stdlib checkout refreshes
> to the tag (`0e63101`; currently `80e767b`). STDLIB RELEASE COMPLETE:
> `stdlib-v0.62.0` published (run 36495200067; assets live; the registry
> lane re-dispatches the publish -- their item).
> Method: repro-first under tmp/sprintc/, locks
> (tests/regression fixture + checker_locks/e2e + CI line where
> e2e-able), full e2e ONCE per batch, python tools/ascii_guard.py check
> before every commit, commit atomically with SESSION.md +
> COMPILER_BUGS.md evidence. Identity Lefteris Notas
> <lefterisnotas@gmail.com>; pushes only when the owner asks; never
> rebuild target/debug while e2e runs.

## Status snapshot (2026-09-29 (9))

- **Owner decisions (2026-09-29)**: selfhost GREEN-LIT as a parallel track
  (Phase 0 ground-laying may start now; Phases 0-2 independent, Phase 3+
  after a behavior freeze); **release policy: selfhost ships only at 100%
  bootstrap** (self1 == self2 sha256 + T3 byte-identical IR + full suite)
  -- until then every release is the Rust-hosted compiler; Stage 6 tier 3
  parked pending the noise-budget call.
- **VS Code extension 0.12.1 LANDED + PUBLISHED + VERIFIED** (2026-09-29,
  workflow run 36578182481): README updated (tested through v0.62.1;
  Stage 6 diagnostic codes W002-W008 / T001 surface via `xiom-lsp`;
  publishing paragraph) + new `.github/workflows/vscode-publish.yml`
  (`workflow_dispatch` from main: node 20 universal VSIX,
  per-marketplace version-absent gate, dry-run, VSIX + SHA256SUMS
  artifact, step summary). The listings moved: Open VSX 0.12.1 at
  13:51:48Z, VS Code Marketplace 0.12.1 at 13:59:06Z (`release.yml` keeps
  publishing extension versions on `v*` tags). Root cause of the "sites
  didn't update" report: no extension changes since 2026-09-22
  (0.12.0 == repo HEAD; zero `xiom-lsp`/`xiom-dbg` diffs from v0.61.3 to
  v0.62.1), so the release gate skipped republishing an existing version
  -- correct, but there was no way to ship an extension-only refresh
  without a compiler tag until now. **Owner verified the sites and the
  VS Code auto-update; the v0.62.1 release VSIX was swapped to 0.12.1
  (`ab1de7f9...`) and dl.xiom-lang.org now serves it (ops fingerprint fix
  in `dl-deploy.sh`; `dl-verify.sh` green) -- thread CLOSED.**
- **packages `byte_at >= 128` LANDED**: direct comparisons of
  `string.byte_at(s, i)` against >=128 constants were sext'ing the i8
  call result (195 -> -61) while local binds worked. `widen_to_i64` now
  matches its documented default (zext i1/i8, sext i16/i32; reg_signed
  overrides still win), and the new `widen_operand_to_i64` +
  `expr_int_signedness` tri-state (Ident/As/Paren/shift/Call) resolve
  Int* -> sext, UInt* -> zext, unknown -> type default; `expr_is_unsigned`
  also resolves call returns for Shr. Probe `bad=3` -> `bad=0`. Lock
  `m161_byte_at_direct_compare` + 1 checker_locks (22/22). Gates: e2e
  2389/2389 (+4 ignored, 1223s), checker 195, stdlib-exec 85 (+2),
  modules 40/40, feature-reg 510, freeze 2/2, scripting 34/34 (803s),
  integration 130, robustness 63, fuzz 24, perf 3, diff 24 (+1),
  ascii_guard clean.
- **R-2c + R-2 partial LANDED** (`1814ac36`): unwrap-chain `Option[Vec[Int]].unwrap()
  .len()` receiver resolves both the annotated shape (was a clang error:
  `xiom_str_len` fed `%struct.Vec`) and the unannotated shape (was
  `strlen` of the boxed handle -> 6); `hint_is_vec_or_slice` accepts
  bare/qualified/bracketed Vec/Slice payload hints and
  `local_opt_payload_xiom` container payloads are consulted. Inline
  aggregate match payloads bind by address and Option/Result LOCAL
  scrutinees match in place, so `Some(c) => c.inc()` persists (was 6/6,
  now 6/7); temporaries keep the snapshot. Locks
  `m159_r2c_unwrap_vec_len`, `m160_r2_match_alias` + 2 checker_locks
  (21/21). Gates: e2e 2389/2389 (+4 ignored, 1224s), checker 195,
  stdlib-exec 85 (+2), modules 40/40, feature-reg 510, freeze 2/2,
  scripting 34/34 (1097s), integration 130, robustness 63, fuzz 24,
  perf 3, diff 24 (+1), ascii_guard clean.
- **Stage 6 tier 2 LANDED** (`75623b4f`): W008 literal integer div/rem by zero
  (parens/`-0` unwrapped; floats and float-adopting int literals stay
  silent), W006 shift amount out of range (type-aware; `Int`=64-bit;
  literal amounts only), W007 self-comparison always true/false
  (non-float reflexive types; floats/calls/structs excluded). `W005` is
  the m142 codegen known-gap advisory (stderr-only, not a checker lint),
  so div/rem ships as `W008`. Locks `m156_w008_*`, `m157_w006_*`,
  `m158_w007_*` + 6 checker_locks (19/19). Gates: e2e 2389/2389
  (+4 ignored, `--test-threads 12`), checker 195, stdlib-exec 85 (+2),
  modules 40/40, feature-reg 510, freeze 2/2, scripting 34/34 (1202s
  under concurrent stdlib smoke load), integration 130, robustness 63,
  fuzz 24, perf 3, diff 24 (+1), ascii_guard clean. Next: backlog
  (R-2 partial + R-2c first).
- **Compiler v0.62.1 released** (tag re-cut once for the CR-guard fix,
  `f93f4ee6`; publish glob fix `99096abd`): R-5 (PR #4), C001, C9/C8b,
  CI hygiene, `m150_dl_num_parse` e2e lock. Gates: e2e 2389/2389
  (+4 ignored), checker 195, stdlib-exec 85, modules 40/40, feature-reg
  510, freeze 2/2, quick suites, CLI/MCP 44/pkg 75, notes 6.
- **Stage 6 W004 landed** (`43623bc4`): unreachable match arm; locks
  `m154_w004_unreachable`/`_guard` + 2 checker_locks (12/12). Probe-caught
  AST fix: bare enum variants parse as DOTTED `Pattern::Ident`;
  `pattern_is_catch_all` now excludes them (hardens W002/W003 too).
- **R-2d landed** (`830a68d0`), **pushed** with W004 + the handoff
  (origin/main = `f7baa932`): `--check` no longer implicit-main-wraps
  library files; verified on the benchmark's t3 solution + 3 templates;
  lock `m155_r2d_check_library` (checker_locks 13/13). The e2e suite never
  uses `--check`, so the changed branch is structurally outside it.
- **Versioning FAQ**: stdlib v0.62.0 = the stdlib repo's release
  (`80e767b`, tag `stdlib-v0.62.0`; the tag was force-updated to `0e63101`
  on 2026-09-29 for the registry re-cut); compiler v0.62.1 = our patch
  release
  PINNED to that stdlib; extension 0.12.0 = independent VS Code extension
  version (last real change 9/22; no update is delivered until a version
  bump). The three are expected to differ.
- **Backlog (in order)**: ~~benchmark R-2 partial + R-2c~~ (landed
  2026-09-29); ~~packages `byte_at >= 128` direct compare~~ (landed
  2026-09-29); loop-CSE retry -- **retry run clean on `f4af5f64`**
  (`docs/repro/loop-carry-cse/probe_loop_cse.xi` -> `bad=0`, V1/V2/V3
  all correct, same as v0.62.1); the pre-fix amqp decoder is NOT in the
  packages git history (only the fragment in the repro README), so a
  faithful probe needs the porting session's pre-fix file -- relayed to
  the packages lane; module-level const/table materialization;
  same-name fn shadowing (no redefinition diagnostic); `Vec` ~2^24 cap;
  transient `program_exit=-1` capture batch; playground polish
  (Range-only `unknown type 'Iterator'` warning; W005 stub behind
  `(2 + 2.5).to_str()`); CI Heavy Suites triage.
- **Git**: origin/main = local main (tip = this handoff commit; prior
  tips `bc047d9c`, `72e42ac2`, `19c611c0`); all pushed, tree clean.
  Landed commits: tier-2 `75623b4f`, R-2 `1814ac36`, byte_at `f4af5f64`,
  session docs `e064fddf`, vscode 0.12.1 refresh `19c611c0`, docs
  `72e42ac2`/`bc047d9c`. Repo-local nested `stdlib` checkout at `80e767b`
  (tag `stdlib-v0.62.0` now points at `0e63101`; refresh at the next pin
  step).

# BATCH HANDOFF (2026-09-28, R-5 relay fix -- branch `bench/r5-extern-gate`)

Branch `bench/r5-extern-gate` (worktree `E:\xiom-lang\xiom-bench`, off
release main `7ca323a4`, rebased over the v0.62.0 pin) carries the
benchmark-relay R-5 fix: the T002 extern gate no longer fires on
`xiom.math.primitives`' own `pub fn abs` when `xiom.ffi.c`'s private libc
`abs` is also in the import closure (`use xiom.num;` + `use xiom.ffi.dl;`);
catalog-body findings now print the module tag BEFORE the span and carry
`catalog:<module>` in the JSON diagnostics envelope (merged with the Stage 6
warning stream). Locks: `tests/regression/m150_dl_num_parse` (check-clean
positive), `m150_dl_num_catalog_abs` (e2e + CI line), `m150_extern_gate`
(negative extern gate). Evidence: docs/COMPILER_BUGS.md "2026-09-28 -- R-5
FIXED". Gates: checker_locks 10/10, xiom-check 195/195, targeted e2e 1/1,
full e2e 2387/2387 (+4 ignored) on pin `stdlib-v0.62.0` (80e767b). The same
bugs-log entry files the open `io.parse_int` -> bare `@is_empty` C001 as the
next batch. Off-tree sibling note: the benchmark side's fairness hardening
(D1-D7a) landed in the benchmark repo; unrelated to this branch.

---

# CONTINUATION HANDOFF (2026-09-27 (3), compiler lane -- item 3 + W002/W003 landed; release may cut early)

Supersedes the 2026-09-27 (2) header below (kept as history). Evidence:
docs/COMPILER_BUGS.md "2026-09-27 -- item 3 landed: exact arity ON" and
"2026-09-27 -- Stage 6 lint wave: W002 + W003".

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM SWARM compiler lane. Read the top section of SESSION.md
> and docs/STAGE6_LINT_WAVE.md first. State: **v0.62.0 release prepared
> locally (pin `stdlib-v0.62.0`, version 0.62.0, strict clauses ON,
> brackets held); pushes pending the owner's go**; stdlib checkout detached
> at `stdlib-v0.62.0`; local main carries the campaign UNPUSHED (65 commits
> after the last docs commit); tree clean; latest full e2e 2386/2386
> (+4 ignored, script cache redirected -- this box's Defender blocks
> `%TEMP%\xiom_run` exes with os error 225). Run long suites with reduced
> threads (`-- --test-threads 12` for e2e, 8 for stdlib_tests):
> default-thread runs storm under concurrent lane load and can produce the
> cross-lane transient `program_exit=-1` silent compile failures (packages
> row 28; my e2e m35 storms) -- never record a silent failure as a pass
> without a re-run.
> Item 3 (exact arity), Stage 6 W002+W003, and R53 (`&mut` write-through)
> are DONE -- do not redo. If the release pushes are still pending, follow
> the pending push sequence in the snapshot. W004 is next in the wave,
> then tier 2 (W005-W007).
> Backlog order after the wave: (1) transient `program_exit=-1` capture
> batch (now also covers the Defender-blocked script-exe class), (2)
> `byte_at >= 128` direct compare (REPRODUCED), (3) loop-carried CSE retry
> with the amqp:1266 fragment, then module-const/table materialization,
> same-name fn shadowing, Vec 2^24 cap, bracket-flip + stdlib canonicalize
> (3 sites). Method: repro-first under tmp/sprintc/, locks
> (tests/regression fixture + checker_locks/e2e entry + CI line where
> e2e-able), full e2e ONCE per batch, python tools/ascii_guard.py check
> before every commit, commit atomically with SESSION.md +
> COMPILER_BUGS.md evidence. Identity Lefteris Notas
> <lefterisnotas@gmail.com>; pushes only when the owner asks; never
> rebuild target/debug while e2e runs.

## Status snapshot (2026-09-27 (3))

- **Stage 6 W002 + W003 LANDED** (warning-only, user-program scope, one
  code each). W003: unreachable statement after an unconditional diverger
  (return/break/continue, all-diverging if, catch-all all-diverging match,
  `while true` without a reachable break) -- warns once per block.
  W002: unconditional recursive cycles over the user unit's non-generic
  free fns; the graph is built from CERTAIN-call edges only (every path
  evaluates the call before any exit), so guard-first recursion stays
  silent. Plumbing: `CheckError.code`; `warn_coded_at`; driver prints
  `warning[WNNN]`; JSON envelope carries the warning stream with
  kind/code; `docs/JSON_DIAGNOSTICS_V1.md` updated (kind `warning`,
  W000-W003); coded warnings survive the error path with their label.
  Locks: `m151_w002_cycle` / `m151_w003_unreachable` (positive, compile
  exit 0) + `_guard` / `_guarded` (negative), 4 checker_locks tests.
  Gates: e2e 2385/2385 (+4 ignored, `--test-threads 12`), checker 195,
  strict corpus 1, stdlib-exec 85 (+2), modules 40/40, feature-reg 510,
  integration 130, robustness 63, fuzz 24, perf 3, diff 24 (+1),
  checker_locks 8/8.
- **Flake corroboration**: packages row 28 (`program_exit=-1`, empty
  output, green on re-run: memcached/git2/db2) matches the e2e m35 storm
  signature (31-32 spurious silent compiles per loaded run; all fixtures
  green solo). stdlib_tests reproduced it too (0/40 then 32/40 under lane
  load; 40/40 at `--test-threads 8`). Capture batch queued (exit code +
  dump on a loaded re-run).
- **Item 3** (exact arity, pin `0c50ac6`) landed earlier today
  (commit `0f3f5083`): four hunks + three resolver fixes + m150 locks.
- **Release**: owner leaning to cut v0.62.0 before the stdlib's 100%
  ("stable compiler first"; the next release continues Stage 6 +
  findings). Await the stdlib relay before executing the 2026-09-25
  order; `XIOM_STRICT_BRACKETS` default flip and the release-notes
  re-conversion are in that order.
- **R53 LANDED: `&mut` out-param write-through.** Plain-local calls to
  `&mut`/pointer params (`set_one(x)`) now pass the local's slot address
  (the implicit form of the explicit-borrow branch) instead of BUG-31's
  discarded temp copy; the guard set covers inferred pointer locals,
  address-carrying ref params/locals, arrays, closures, fn-typed locals and
  handle locals. Packages `probe_out_params.xi` `bad=0`; struct-bag variant
  green. Locks: `m152_mut_write_through` + e2e + CI. Gates: e2e
  2386/2386 (+4 ignored), stdlib-exec 85, feature-reg 510, modules 40/40,
  checker 195, strict 1, quick suites, locks 8/8.
- **C001 LANDED (benchmark unblocker)**: `io.parse_int` -> bare
  `@is_empty` fixed -- a let-bound Str from a receiver-sugar free fn
  (`let trimmed = s.trim();`) never recorded its XIOM type, so
  `trimmed.is_empty()` mis-resolved the receiver as a MODULE and emitted
  an undefined `@is_empty`. The resolver now derives the primitive from
  the receiver local's LLVM type (`i8*`->Str, ...) when the matching
  `Type.method` key exists. Lock: `m153_parse_int_trim_isempty` + e2e +
  CI. Gates: e2e 2387/2387 (+4 ignored), checker 195, stdlib-exec 85,
  modules 40/40, feature-reg 510, quick suites, locks 8/8. Awaiting
  0.62.1 with the R-5 PR.
- **Stage 6 W004 LANDED**: unreachable match arm (unguarded catch-all
  shadow; duplicate literal/variant, structurally keyed with payload
  arity). Probe-caught AST subtlety fixed: bare enum variants parse as
  DOTTED `Pattern::Ident`; `pattern_is_catch_all` now excludes those
  (hardens W002/W003 too). Locks `m154_w004_unreachable`/`_guard` +
  2 checker_locks tests (12/12). Gates: e2e 2389/2389 (+4 ignored),
  checker 195, stdlib-exec 85, modules 40/40, feature-reg 510, freeze 2/2,
  quick suites. Next: tier 2 (W005-W007).
- **v0.62.1 patch RELEASED (pushed; release run green)**: R-5
  (PR #4 merged, all checks green -- first fully green CI), C001, C9/C8b
  (playground), CI hygiene, and the promoted `m150_dl_num_parse` e2e lock.
  Gates: full e2e 2389/2389 (+4 ignored), checker 195, stdlib-exec 85,
  modules 40/40, feature-reg 510, freeze 2/2, integration 130, robustness
  63, fuzz 24, perf 3, diff 24 (+1), CLI/MCP 44/pkg 75, notes 6, workspace
  `--lib`; notes verify green. Release tag re-cut once onto the CR-guard
  fix (`f93f4ee6`); the publish-job glob fix (`*.js`/`*.d.ts`) landed after
  (`99096abd`); the v0.62.1 wasm glue was fix-forwarded from the CI artifact.
- **Playground fixes landed**: C9 (`opt -verify` -> `-passes=verify` with
  legacy fallback + silent skip; the LLVM-18 warning no longer prints on
  every run) and C8b (wasm-bindgen glue fix-forward on v0.62.0:
  `xiom-wasm.js`/`.d.ts`/`_bg.wasm` uploaded, generated from the released
  wasm with CLI 0.2.126; release.yml now builds + publishes the glue).
  Next: R-5 PR merge (CI hygiene), then 0.62.1.
- **Release v0.62.0 RELEASED (pushes done 2026-09-28)**: `STDLIB_VERSION`
  -> `stdlib-v0.62.0` (tag on the stdlib release cut `80e767b`, pushed),
  workspace version 0.61.3 -> 0.62.0 (`Cargo.toml` + `cargo update -w`
  lock refresh), release notes re-converted with the stdlib fragment
  (6 highlights, verify green). Strict-CLAUSE default ON
  (`XIOM_STRICT_CLAUSES=0` opts out) with the literal-0 null-cast
  exemption; strict-BRACKETS default flip HELD (3 stdlib mixed sites
  relayed: `io/fs.xi` 36+244, `math/algebra_extended.xi` 311; lands next
  release). Gates: full e2e 2386/2386 (+4 ignored; script cache
  redirected -- Defender blocks `%TEMP%\xiom_run` exes, os error 225),
  checker 195, stdlib-exec 85, modules 40/40, feature-reg 510, parser
  106, notes 6, integration 130, robustness 63, fuzz 24, perf 3, diff 24,
  CLI suites, MCP 44, pkg 75.
  Pending push sequence: DONE -- stdlib tag `stdlib-v0.62.0`, compiler
  `main` = `7e4d0909`, annotated tag `v0.62.0`; XIOM Release 36438304704
  all green (guard, 5 packages incl. windows-x64, publish, docs dispatch)
  -> https://github.com/xiom-lang/xiom/releases/tag/v0.62.0. CodeQL on
  main green. Extra CI dispatch 36439203600 found 3 PRE-EXISTING CI-only
  failures (parser stack overflow on both OSes; 2 Windows-path lib tests
  run on Linux) -- follow-up CI-hygiene batch queued; registry canary
  remains the external lane's step.
- **Git**: local `main` = origin/main + 65 commits after this docs commit
  (all UNPUSHED; pushes only on the owner's ask). stdlib checkout
  detached at `stdlib-v0.62.0`; compiler tree clean.

# CONTINUATION HANDOFF (2026-09-27 (2), compiler lane -- item 3 landed; Stage 6 lint wave next)

Supersedes the 2026-09-27 header below (kept as history). Evidence for the
item-3 landing lives in docs/COMPILER_BUGS.md "2026-09-27 -- item 3 landed:
exact arity ON".

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM SWARM compiler lane. Read the top section of SESSION.md
> and docs/STAGE6_LINT_WAVE.md first. State: v0.61.3 released; stdlib
> checkout detached at stdlib main `0c50ac6` (pushed); local main carries
> the campaign UNPUSHED (60 commits after this docs commit); tree clean;
> latest full e2e 2385/2385 (+4 ignored) -- run it with
> `-- --test-threads 16` on this box: default-thread runs storm the ~300
> contiguous m35 compiles and produce silent spurious failures (all clean
> individually and in isolation; infrastructure, not code). No red gates.
> Item 3 (exact arity) is DONE and ON: pin `0c50ac6` + the four hunks + three
> resolver fixes (wildcard bound-ownership deferral, receiver-leaf matching,
> interface member arity model), locks `m150_exact_arity` + CI. Items 1, 2
> and R-1, R-2, R-3 are DONE -- do not redo. Item 4 (pin bump + release)
> follows the 2026-09-25 order and is blocked on the stdlib release tag
> (100%). Meanwhile run the Stage 6 lint wave: W002 + W003 first per
> docs/STAGE6_LINT_WAVE.md (warning-only, user-program scope, one code per
> lint, two locks per lint: positive warns + exit 0, negative silent;
> corpus/e2e stay zero-warning). Plumbing notes below. Method: repro-first
> under tmp/sprintc/, locks (tests/regression fixture + checker_locks entry
> + CI line where e2e-able), full e2e ONCE per batch (~30 min, no rebuilds
> during), python tools/ascii_guard.py check before every commit, commit
> atomically with SESSION.md + COMPILER_BUGS.md evidence. Identity Lefteris
> Notas <lefterisnotas@gmail.com>; pushes only when the owner asks; never
> rebuild target/debug while e2e runs.

## Status snapshot (2026-09-27 (2))

- **Item 3 LANDED: exact arity ON.** `STDLIB_VERSION` -> stdlib main tip
  `0c50ac6` (pushed; carries the 90e9185 call-site fixes + c193bc4 m146
  prep). Four hunks flipped (COMPILER_BUGS m147 recipe) with
  call-shape-correct expected counts (explicit-receiver instance calls,
  implicit-this static calls). The flip surfaced and this batch fixed three
  latent resolution defects: wildcard singleton capture of a derived
  `compare` for generic `T: Ord` receivers (bind-bound receivers now defer
  to interface dispatch); receiver-sugar shape `fn Box.get[T](b: &Box[T])`
  + `b.get()` (normalized receiver-leaf matching); and the interface member
  arity model (`&Self` operands vs implicit receivers, m37_bug45). Locks:
  `tests/regression/m150_exact_arity/{main,reject_extra,reject_missing}.xi`
  + `e2e_m150_exact_arity`/`_rejects` + CI line. Pin absorption step 6
  verified: `tcp_connect("127.0.0.1", 1)` takes Err with a negative code.
- **Gates (all green)**: full e2e 2385/2385 (+4 ignored; --test-threads 16),
  checker 195/195, strict-clause catalog corpus 1/1, stdlib-exec 85/85
  (+2 ignored), stdlib modules 40/40, feature-reg 510/510, integration 130,
  robustness 63, fuzz 24, perf 3/3, diff 24 (+1 ignored). Harness fragility
  filed: default-thread e2e storms the m35 block into silent spurious
  compile failures; the harness does not retry `None` compiles and the
  driver prints nothing when clang/link fails (silent-failure gap); 20,400
  stale `e2e_*.exe` (6.3 GB) were cleaned from the repo root.
- **Stage 6 lint wave READY to start** (spec: docs/STAGE6_LINT_WAVE.md;
  order: W002 + W003, then W004, then W005-W007). Sized plumbing:
  - Checker warnings are `CheckError { message, span, cause, guaranteed }`
    (crates/xiom-check/src/types.rs:610), pushed via `warn`/`warn_at`
    (lib.rs:1350/1356), catalog-scoped by the "catalog body" prefix.
    Only 8 `CheckError {` literal sites -> adding `pub code: Option<String>`
    is feasible; alternatively carry "W002:"/"W003:" prefixes.
  - Driver printing: crates/xiom/src/lib.rs:868-879 partitions
    `catalog_warns` (W000, capped 5) vs `own_warns` (W000) -- own_warns
    must print the per-lint code. JSON envelope currently carries errors
    only; spec item 2 wants the new `kind`/`code` in the v1 diagnostics
    (docs/JSON_DIAGNOSTICS_V1.md); `Diagnostic{kind,code,...}` already
    exists for W001 (lib.rs:582).
  - Lint scope = the USER program only; `checking_catalog` guards the
    stdlib so `catalog_corpus_is_clean` + the zero-warning e2e tests stay
    green structurally.
  - Two locks per lint: positive fixture (binary test asserts
    `warning[WNNN]` in stderr AND exit 0) + negative fixture (silent),
    pattern in crates/xiom/tests/checker_locks.rs; CI line for e2e-able
    cases. W002 = call graph over direct calls by name, warn only when
    every function in an SCC unconditionally reaches a cycle call before
    any exit (guard-first recursion must stay silent). W003 = unreachable
    statement after a diverger in the same block (per-block, not across
    labels).
- **Release side**: item 4 order (from the 2026-09-25 section): stdlib ref
  with release-notes/v0.62.0.md -> pin + STDLIB_VERSION -> stdlib gates ->
  `XIOM_STRICT_BRACKETS` default flip (+ clause default once confirmed) ->
  re-convert notes -> version 0.61.3 -> 0.62.0 -> push (FIRST CI since R66)
  -> tag -> registry canary -> website notes. Version still 0.61.3.
- **Git**: local `main` = origin/main + 60 commits after this docs commit
  (all UNPUSHED; pushes only on the owner's ask). stdlib checkout detached
  at `0c50ac6`; compiler tree clean.
- **Backlog (new)**:
  - Benchmark lane resume: R-5 fix + fixtures saved at
    `%TEMP%\kilo\bench_r5_withdraw.patch` / `bench_r5_fixtures\` /
    `bench_r5_NOTES.md` (owner relayed); resume in a dedicated worktree
    (branch `bench/r5-extern-gate`), then the `io.parse_int` -> bare
    `@is_empty` C001 follow-on batch.
  - Packages findings (`E:\xiom-packages\packages\docs\COMPILER-FINDINGS.md`):
    the arity row is FIXED by item 3; next compiler-lane candidates:
    `&mut Int` write-through miscompile (upnp), loop-carried CSE miscompile
    (amqp:1266), mixed-bracket typos accepted silently (parser diagnostic),
    bit tests with sign bit set, `byte_at` vs UInt8 >= 128.
  - Playground findings: `12 + 2.to_string()` silently concatenates
    ("122"); `(2 + 2.5).to_str()` hits the W005 stub (prints 0) while
    `float_to_string(...)` prints 4.5 and annotated Float64 locals work;
    `for x in range(...)`/Range values emit `unknown type 'Iterator' --
    defaulting to i64` warnings (semantics correct; Vec/array loops quiet).
  - Harness hardening: retry/telemetry for silent (`None`) compile
    failures; driver should print the clang/link failure (no silent
    failures); artifact retention in the repo root.

# CONTINUATION HANDOFF (2026-09-27, compiler lane -- waiting on stdlib; Stage 6 lint wave next)

Supersedes the 2026-09-26 header below (kept as history). All evidence for
2026-09-26/27 lives in docs/COMPILER_BUGS.md (m142-m148 + R-1/R-2/R-3).

## Next-session kickoff prompt (copy/paste)

> Continue the XIOM SWARM compiler lane. Read the top section of SESSION.md
> and docs/STAGE6_LINT_WAVE.md first. State: v0.61.3 released; stdlib
> detached at stdlib-v0.61.3; local main carries the campaign UNPUSHED
> (59 commits after this docs commit); tree clean; latest full e2e
> 2383/2383 (+4 ignored); no red gates. Items 1, 2 and R-1, R-2, R-3 are
> DONE -- do not redo. Item 3 (exact arity) is implemented and VERIFIED but
> OFF: it lands only with a pin carrying stdlib 90e9185 (unpushed as of
> 2026-09-27; origin/main 49b4731) -- on their push: bump STDLIB_VERSION to
> that ref, apply the four hunks (COMPILER_BUGS m147 section), full gates.
> Item 4 (pin bump + release) follows the 2026-09-25 order. Meanwhile run
> the Stage 6 lint wave: W002 + W003 first per docs/STAGE6_LINT_WAVE.md
> (warning-only, user-program scope, one code per lint, two locks per lint:
> positive warns + exit 0, negative silent; corpus/e2e stay zero-warning).
> Plumbing notes below. Method: repro-first under tmp/sprintc/, locks
> (tests/regression fixture + checker_locks entry + CI line where
> e2e-able), full e2e ONCE per batch (~30 min, no rebuilds during),
> python tools/ascii_guard.py check before every commit, commit atomically
> with SESSION.md + COMPILER_BUGS.md evidence. Identity Lefteris Notas
> <lefterisnotas@gmail.com>; pushes only when the owner asks; never rebuild
> target/debug while e2e runs.

## Status snapshot (2026-09-27)

- **R-3 CLOSED as already-fixed**: block comments `/* ... */` are an
  intentional lexer feature (crates/xiom-lexer/src/lib.rs:102) and compile
  in block/type/expression positions; an unterminated one reports
  `error[L001]: <line>:<col>: unterminated block comment` (probes
  tmp/sprintc/m149_block_comment_probe{,2,3}.xi). The benchmark relay's
  R-3 is stale; no compiler change. Benchmark trio R-1/R-2/R-3 all closed.
- **Stage 6 lint wave READY to start** (spec: docs/STAGE6_LINT_WAVE.md;
  order: W002 + W003, then W004, then W005-W007). Sized plumbing:
  - Checker warnings are `CheckError { message, span, cause, guaranteed }`
    (crates/xiom-check/src/types.rs:610), pushed via `warn`/`warn_at`
    (lib.rs:1350/1356), catalog-scoped by the "catalog body" prefix.
    Only 8 `CheckError {` literal sites -> adding `pub code: Option<String>`
    is feasible; alternatively carry "W002:"/"W003:" prefixes.
  - Driver printing: crates/xiom/src/lib.rs:868-879 partitions
    `catalog_warns` (W000, capped 5) vs `own_warns` (W000) -- own_warns
    must print the per-lint code. JSON envelope currently carries errors
    only; spec item 2 wants the new `kind`/`code` in the v1 diagnostics
    (docs/JSON_DIAGNOSTICS_V1.md); `Diagnostic{kind,code,...}` already
    exists for W001 (lib.rs:582).
  - Lint scope = the USER program only; `checking_catalog` guards the
    stdlib so `catalog_corpus_is_clean` + the zero-warning e2e tests stay
    green structurally.
  - Two locks per lint: positive fixture (binary test asserts
    `warning[WNNN]` in stderr AND exit 0) + negative fixture (silent),
    pattern in crates/xiom/tests/checker_locks.rs; CI line for e2e-able
    cases. W002 = call graph over direct calls by name, warn only when
    every function in an SCC unconditionally reaches a cycle call before
    any exit (guard-first recursion must stay silent). W003 = unreachable
    statement after a diverger in the same block (per-block, not across
    labels).
- **Release side**: item 3 flip verified locally against a patched pin
  mirroring stdlib 90e9185 (corpus + full e2e green); blocked on their
  push. Item 4 order (from the 2026-09-25 section): stdlib ref with
  release-notes/v0.62.0.md -> pin + STDLIB_VERSION -> stdlib gates ->
  `XIOM_STRICT_BRACKETS` default flip (+ clause default once confirmed) ->
  re-convert notes (currently `xiom-release-notes verify --tag v0.62.0`
  green, 4 highlights; tests 6/6) -> version 0.61.3 -> 0.62.0 -> push
  (FIRST CI since R66; Windows leg unverified) -> tag -> registry canary ->
  website notes. Version still 0.61.3.
- **Git**: local `main` = origin/main + 59 commits after this docs commit
  (all UNPUSHED; pushes only on the owner's ask). stdlib checkout detached
  at `stdlib-v0.61.3`; origin/main 49b4731.
- **Backlog**: R-5/R-6 (need docs/FAIRNESS-RELAY-2026-09-26.md, not in
  this tree); C8 wasm asset (release-lane hygiene, not compiler code);
  tier-2/3 lints (W005-W007 then the noise-budget decision); post-pin
  check that the W005 hash/Error.chain tolerances are dead in the stdlib
  ref (keep the tolerance mechanism as safety).

# CONTINUATION HANDOFF (2026-09-26, compiler lane -- items 1+2 + sibling-bind)

Supersedes the 2026-09-25 header below (kept as history). Evidence for these
batches: docs/COMPILER_BUGS.md "2026-09-26 -- m142: ptr.is_null() silent-stub
kill + UFCS", "2026-09-26 -- m143: by-value receiver container mutation" and
"2026-09-26 -- m144: sibling-method receiver binding (HashMap crash class)".

## Status snapshot

- **R-2 DONE (m148, benchmark relay): match payload bindings ALIAS the box.**
  `match o { Some(v) => { v.n = 6; } }` on a persistent `Option[Cell]` used
  to mutate a stack copy (re-read saw the old value); `Some(v) => v.push(x)`
  on a Vec payload kept len 0. The arm binding now registers the payload
  ADDRESS with the pointer-backed struct-local convention for aggregate and
  Vec payloads (field writes, method receivers and pushes hit the box).
  Also fixed in the same batch (pre-existing on the 2026-09-22 release
  binary): temporary scrutinees (`match Some(Cell{ n: 9 }) { Some(t) => t.n }`)
  bound the raw i64 handle and read 0 -- `scrutinee_payload_xiom` now
  infers `Some/Ok/Err` literal inner types and
  `infer_expr_xiom_type_deep` handles `Expr::Struct`. Lock
  `tests/regression/m148_match_payload_alias` + e2e + CI line.
  Harness fix: stdlib_tests compiles now pass `--timeout 900` (the CLI's
  300s default tripped deterministically under the 39-way parallel load;
  two incidents, solo runtime ~120s).
- **Playground relay answer (tcp_connect)**: the stdlib's `Int32` extern
  declaration change is committed on their side but NOT yet pushed
  (origin/main still 49b4731). It ships in the first release whose
  `STDLIB_VERSION` carries that ref AND whose compiler carries m146; the
  wasm-asset release does not fix it. Verify tcp_connect at the first such
  pin (absorption step 6).
- **Item 3 PREP DONE (m147): G-10 receiver-registry fix + flip verified.**
  The stdlib reports item 3 fixed on their side (90e9185: printf x3,
  _scrypt_blockmix x2, path.replace) but that ref is UNPUSHED
  (origin/main still 49b4731) and `STDLIB_VERSION` is stdlib-v0.61.3, so
  the exact-arity flip cannot be committed green yet. What landed:
  `check_implicit_self_method` now treats ANY hit in the receiver's method
  registry as the receiver method (the old first-param heuristic missed
  this-based GENERIC methods -- `HashMap.get[K,V](key)` omits the receiver
  from params -- so `HashMap.contains`'s bare `get(key)` fell through to
  xiom.array's free `get(arr, idx)` and tripped exact arity);
  `receiver_in_params` decides the shape/offset, preserving the Vec4f
  historical behavior. The four arity flips (impl `!=`, module `!=`,
  method-path expected_args, bare-path with implicit-this allowance) were
  implemented, temporarily enabled, and VERIFIED: corpus + full e2e green
  against a locally patched pin mirroring 90e9185. They are OFF in the
  commit (flip recipe in COMPILER_BUGS). On the stdlib push: bump
  STDLIB_VERSION to the ref, apply the four hunks, full gates; at release
  re-pin to their tag (item 4).
- **R-1 DONE (m145, benchmark relay)**: C-family/Rust bitwise precedence.
  `1 << 8 | 2` parsed as `1 << (8|2)` (silent wrong values: 1024);
  `& ^ |` shared the `*`/`/` level (`3 | 4 << 1` == 14, `a & b * c` ==
  `(a&b)*c`). The parser now nests `parse_cmp -> bit_or -> bit_xor ->
  bit_and -> shift -> add -> mul`: shifts bind tighter than `&` > `^` > `|`
  > comparisons, additive/mul stay tighter than shifts, and the stdlib's
  `(n >> hi) & 1 == 1` shape is unchanged. Locks: 2 parser AST-shape unit
  tests + `tests/regression/m145_shift_precedence` + e2e + CI line.
- **Playground relay DONE (m146, net.tcp_connect)**: reproduced on Windows --
  `tcp_connect("127.0.0.1", 1)` returned Ok on a refused connection. Two
  stacked causes: (1) the binding path recorded a call-return XIOM type
  WITHOUT updating `signed_locals`, so an `Int32` result widened `zext i32
  -1 to i64` == 4294967295 and `result < 0` was false; fixed (all four
  inference arms in the let/var paths now track signedness). (2) The pinned
  `xiom.net` declares the extern `-> Int` (i64) while C returns `int` (i32):
  `mov eax,-1` zero-extends across the ABI, so even a correct comparison
  cannot see the sign -- stdlib action: declare int-returning externs
  `Int32` (needs this fix in the pin to behave), or add a runtime shim
  returning a 64-bit sentinel. Runtime inspection: `xiom_socket_connect`
  correctly returns `connect(2)`'s result and uses a blocking SOCK_STREAM
  (no completion wait needed); errno/WSAGetLastError is not propagated in
  the `code` field (enhancement). Not fixed by the wasm asset upload --
  needs the stdlib change + pin carrying both. Lock:
  `tests/regression/m146_signed_extern_result` + e2e + CI line.
- **Interface-shape answer for the stdlib's Error.chain wave**: empirically
  (probes m146_*): interface dispatch works when the concrete type is
  statically known at the call site (including default methods calling
  siblings); generic bounds `[T: Error]` monomorphise; interface-typed
  params taking aggregates are C001; interface VALUES in `Option[Error]`
  payloads/unknown receivers are the W005 erased gap (no dynamic dispatch).
  Expected shape: make error data a CONCRETE closed type (`ErrorInfo`
  struct/enum with `cause` as an index/handle) and keep max one interface
  as a generic bound; never use the interface itself as a value type in
  signatures (`Option[Error]`, `e: Error`).
- **Sibling-method receiver binding DONE (m144)**, found during the item-3
  arity survey: `HashMap.insert/get/contains` crashed with an access
  violation on the pinned stdlib (pre-existing: the 2026-09-22 release binary
  reproduces). Four root causes, all fixed:
  1. monomorphised generic bodies never set `current_receiver`, so G-10
     implicit-self resolution was DEAD in every generic method body;
  2. bare sibling calls to generic methods mapped the explicit args
     positionally to (self, ...) -- the KEY was inttoptr'd as the receiver
     pointer and dropped (2-arg call vs 3-param def);
  3. `body_uses_receiver_state` did not count bare sibling calls, so such
     methods were registered WITHOUT a `%param_self` slot while the call site
     still passed one (arg shift);
  4. the G-10 receiver check compared `%struct.X*` vs `%struct.X` with
     `ends_with` (false -- the string ends with `*`), dropping the receiver
     for non-generic this-based sibling calls; and primitive by-value
     receivers passed the alloca address where the value was expected.
  Also reroutes generic bare sibling calls through `self.<name>(args)` and
  loads primitive receiver values. Lock
  `tests/regression/m144_sibling_method_calls/main.xi` + e2e + CI line
  (HashMap insert/resize re-insert/get/contains/count roundtrip + the
  non-generic Holder chain; pre-batch release binary exits 2).
- **Item 2 DONE (m143)**: an explicit by-value `self` method that mutates a
  CONTAINER FIELD (`self.v.push(x)`, `self.m.insert(...)`,
  `self.s.insert(...)`) now uses the POINTER receiver ABI. Root cause: the
  registration + definition `is_mut` detection only recognized FIELD
  ASSIGNMENTS (BUG 31/38b), so `fn S.add(self, x)` was registered/defined
  by-value; the callee's `self.v` push updated a COPY of the Vec header
  (len/cap) and the caller's container silently kept the old length.
  New detector `block_mutates_receiver_container` (decl.rs registration +
  definition) recognizes container-mutator calls on `self`/`this`/bare
  receiver fields; pure accessors keep the by-value ABI. Bare-field
  (this-based) and read-only forms are locked too. Lock
  `tests/regression/m143_receiver_container_mutation/main.xi` +
  `e2e_m143_receiver_container_mutation` + CI line.
- **Item 1 DONE (m142)**: the `ptr.is_null()` silent-stub class is killed.
  - `emitter.rs::emit_undefined_symbol_stubs` now FAILS the compile (C001)
    for any called-but-undefined symbol, naming symbol/return type/IR
    line/caller; two known gaps stay stubbed but LOUDLY (W005): unresolved
    calls inside CONTRACT CLAUSES and to INTERFACE method leaves with no
    concrete impl (`Error.description`/`source`). A symbol called from both
    a tolerated and a non-tolerated site is a hard error.
  - Checker: a value-rooted receiver shadows a same-named imported module
    alias (`ptr.is_null()` in `Ref.release` bound the FIELD, not `xiom.ptr`);
    R8 method-position free-fn resolution now matches ref-ish/generic first
    params (`*Int` vs `*const T`).
  - Codegen: UFCS resolution + receiver-as-arg-0 ABI for generic and
    registered free fns; receiver type-arg inference from field/pointer
    slots; pointer-like receiver detection.
  - Real bugs surfaced by the loud C001 and fixed: `panic(msg)` had NO
    lowering (every panic silently no-oped); `T()` in erased generic bodies
    is the concrete zero; `type_id::<T>()`/`field_offset::<T>(x)` runtime
    folds; **parallel codegen (`--parallel-codegen`) never seeded the
    call-resolution state** (use-imports, bare aliases, preassigned symbols,
    generic decls) into per-function emitters -- bare imported/generic calls
    silently stubbed in parallel mode.
  - The stub-pass IR scan now ignores text inside string constants (the
    selfhost compiler embeds generated IR in `c"..."` literals -- a false
    C001 for `@sq`/`@add`).
- **Lock**: `tests/regression/m142_ptr_isnull_ufcs/main.xi` + `e2e_m142_ptr_isnull_ufcs`
  + CI line + codegen unit test `m142_undefined_symbols_fail_loudly`
  (includes the quoted-IR negative case). In-process IR harnesses
  (feature-reg/integration/robustness/fuzz) opt into
  `set_legacy_stub_unresolved(true)` because they compile without checker
  and stdlib.
- **Gates (m142 batch)**: full e2e **2378/2378 (+4 ignored)** (two runs:
  2376 pass/2 fail -> fixed -> 2378/2378; the failures were the parallel
  state bug and the missing `type_id` fold); checker 195/195; feature-reg
  510/510; integration 130; robustness 63; fuzz 24; stdlib-exec 85/85
  (+2 ignored); stdlib modules 40/40; api-freeze 2/2; perf 3/3; diff 24;
  xiom lib 54; checker_locks/doctor/borrow green; tool tests green;
  ascii_guard green.
- **Gates (m143 batch)**: full e2e **2379/2379 (+4 ignored)** in one run;
  stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40; feature-reg
  510/510; integration 130; robustness 63; fuzz 24; probes green
  (`m143_receiver_mutation_probe`, `pkg_e001_accessor`); ascii_guard green.
- **Gates (m144 batch)**: full e2e **2380/2380 (+4 ignored)** in one run;
  checker 195/195; stdlib-exec 85/85 (+2 ignored; two smokes failed on the
  first run -- os/folder + net/http2 primitive-receiver clang error -- fixed
  in-batch); stdlib modules 40/40 (one flaky 6-min timeout on the first
  run, green on rerun); feature-reg 510/510; integration 130; robustness
  63; fuzz 24; perf 3/3; diff 24; ascii_guard green.
- **Gates (m145 batch)**: full e2e **2381/2381 (+4 ignored)** in one run;
  parser 106/106 (2 new precedence locks); checker 195/195; stdlib-exec
  85/85 (+2 ignored); stdlib modules 40/40; feature-reg 510/510;
  integration 130; robustness 63; fuzz 24; perf 3/3; diff 24;
  ascii_guard green.
- **Gates (m146 batch)**: full e2e **2382/2382 (+4 ignored)** in one run;
  checker 195/195; stdlib-exec 85/85 (+2 ignored); feature-reg 510/510;
  integration 130; robustness 63; fuzz 24; perf 3/3; diff 24; probes
  (`m146_tcp_int32_probe` exit 0 = Err correct; `m146_i32_widen_probe`
  exit 0); ascii_guard green.
- **Gates (m147 batch, checks OFF + official pin)**: full e2e
  **2382/2382 (+4 ignored)**; checker 195/195; stdlib-exec 85/85
  (+2 ignored); feature-reg 510/510; integration 130; robustness 63;
  fuzz 24; perf 3/3; diff 24; ascii_guard green. (Flip-enabled
  verification runs: corpus green + full e2e green against the patched
  pin, 2 runs.)
- **Gates (m148 batch)**: full e2e **2383/2383 (+4 ignored)**; checker
  195/195; stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40 (timeout
  fix); feature-reg 510/510; integration 130; robustness 63; fuzz 24;
  perf 3/3; diff 24; probes green (`m148_match_bind_probe` after=6,
  `m148_match_vec_payload_probe` len=1); ascii_guard green.
- **New benchmark blocker list**: R-1/R-2/R-3/R-5/R-6 per the relay, but
  `docs/FAIRNESS-RELAY-2026-09-26.md` is NOT in this tree -- R-1 fixed
  (m145), R-2 fixed (m148), R-3 (block-comment diagnostic) summarized,
  R-5/R-6 contents unknown; need the doc or a summarized relay to act.
- **Git**: local `main` = origin/main + 58 commits (all UNPUSHED; pushes
  only on the owner's ask). stdlib checkout still detached at
  `stdlib-v0.61.3`.

## Item-3 arity survey (2026-09-26, temporary local flip -- reverted)

With all four exact-arity checks temporarily enabled (`!=` at the impl and
module-prefix sites + the documented method-path/bare-path formulas), the
corpus reports EXACTLY the seven known stdlib call sites and nothing else:
`io.printf` x3 (io.xi:73, io/console.xi:92,162 -- C printf 2 args vs the
1-param wrapper), `_scrypt_blockmix` x2 (crypto/kdf.xi:353,365 -- 2-param sig,
1 arg), `collections.get` (collections.xi:1040, `get(key)` inside
`HashMap.contains`), `path.replace` (os/path.xi:160 -- 2-param sig, 3 args).
A sweep of 147 fixtures/probes (tests/regression + examples/e2e +
tmp/sprintc) found ZERO new violations.

`collections.get` is NOT a stdlib call-site bug: it is the G-10
bare-sibling/implicit-this compiler gap FIXED by m144 (the checker's bare
path chose the 2-param method sig because `owned_here` skips G-10 when the
module owns same-leaf methods). When item 3's flip lands, the checker's bare
path must account for implicit-this: when the selected sig is a receiver
method of the current receiver and `args.len() + 1 == sig.params.len()`,
accept without error (or relax the `owned_here` gate for receiver-method
hits). Remaining true stdlib fixes for item 3: printf x3,
_scrypt_blockmix x2, path.replace.

## Cross-lane updates (what stdlib/packages must now do)

- **stdlib**: the `ptr.is_null()` workaround can be reverted -- `ptr.is_null()`
  (method form) now compiles AND returns the real result; `Ref.release` /
  `RefMut.release` in `xiom.cell` may switch back from `is_null(ptr)`.
  W005 lists the exact spots their wave should still fix: (a)
  `xiom.hash` `Int.hash` ensures `a == b => a.hash() == b.hash()` references
  UNDECLARED `a`/`b` (invalid clause; currently stubbed); (b) the
  interface-default dispatch gap (`Error.chain`'s `self.description()` /
  `self.source()`) stays W005-stubbed until dynamic interface dispatch
  lands. The arity call-site list is unchanged (item 3 gate).
- **packages/website**: no action; W005 warnings appear only for the two
  gaps above.
- **Backlog (added, not started)**: C8 release-lane: v0.61.3 GitHub release
  lists `xiom-wasm-0.61.3.wasm` in SHA256SUMS but ships no such asset (mirror
  404s; other five verify OK; ops' dl-deploy warns until fixed -- upload the
  wasm or regenerate SHA256SUMS). R-1..R-3 from
  `docs/FAIRNESS-RELAY-2026-09-26.md` (benchmark relay): `a << b | c` parses
  as `a << (b | c)` (silent wrong values; fix precedence or lint), `match` on
  a persistent `Option[T]` binds a copy, `/* */` deserves a targeted
  "block comments unsupported" diagnostic. Test-hygiene backlog: ~22
  feature-regression sources use legacy `;`-separated enum variants /
  `= struct {}` spellings and only compile via parse-error recovery.

## Remaining work order (compiler lane)

1. ~~`ptr.is_null()` silent-stub kill~~ DONE (m142).
2. ~~By-value receiver Vec mutation~~ DONE (m143): pointer receiver ABI for
   explicit-`self` methods that mutate container fields.
3. **Re-land exact arity** only after the stdlib wave fixes its call sites
   (list unchanged: io `printf` x3, `_scrypt_blockmix` x2,
   `collections.get`, `path.replace`); flip the three `>` to `!=`, corpus +
   e2e.
4. **Pin bump + release** (blocked on the stdlib's 100%): unchanged from the
   2026-09-25 section (do NOT bump the version or tag before their ref).

# CONTINUATION HANDOFF (2026-09-25, compiler lane -- post-relay)

Supersedes the 2026-09-24 header (kept below as history). Full evidence for
this stretch lives in docs/COMPILER_BUGS.md under the 2026-09-24/25 sections
(Sprint A-D, relay batches, packages relay #1-#3, stdlib relay).

## Status snapshot

- **Released:** compiler **v0.61.3**; stdlib pinned at `stdlib-v0.61.3` (the
  local `stdlib/` checkout is detached there). `origin/main` = R66 only;
  local `main` carries the whole campaign **UNPUSHED** (50 commits: R67-R72,
  m125/m126/m128, the front-end audit, the release-notes publisher, Sprints
  A-D, the relay batches and fixes). Tree clean.
- **Release notes DRAFTED:** `release-notes/v0.62.0.md` + `.json` (4
  compiler highlights; the stdlib fragment budget is 2 -> schema max 6);
  `xiom-release-notes verify --tag v0.62.0` is green fragment-free.
  Re-convert after the pin bump so their fragment merges.
- **Gates (latest evidence):** full e2e **2377/2377 (+4 ignored)**; checker
  195/195; parser unit tests green incl. the new bracket lax/strict test;
  integration 130; feature-reg 510; robustness 63; fuzz 24; api-freeze 2/2;
  stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40; mcp 44; pkg 75;
  release-notes 6; perf 3/3; catalog corpus clean; workspace all-targets
  clean. (Earlier full runs: relay 2379, Sprint D 2377, Sprint C 2377,
  Sprint B 2374, Sprint A 2373.)

## Landed this campaign (do NOT redo)

- **Sprint A-D**: shared toolchain probe + doctor v2 (`--json`, exit codes,
  `--deep`, identity block, OS remediation); FE-8 deprecations (`xiom
  install|publish` -> `xiom pkg` aliases, `xiom update` retired, pkg help,
  plan banner); FE-10..15/FE-17 (installers ship 9 tools + `lib/` layout,
  banners, uninstall PATH, grouped `--help`, `--version` install root,
  `module x;`); `xiom toolchain check [--json]`; MCP `get_contracts` /
  `search_symbols`; FE-16 pin embed.
- **Sprint C**: fn-value/generic-mono ABI unification (parser `type_to_expr_ident`
  / `type_name_str`; checker `from_marker`, method_target guard; codegen
  marker/locals/for-loop/array-buffer/paren-target fixes; `fn_arg_generic_binding`
  + LLVM->XIOM mapping) and E001 conservatism (`check_stmt` per-statement
  release, `write_borrow` sentinel fix, `LoanSet` mark/release_since).
- **Post-relay fixes**: private same-leaf triage (types + enums); clause
  Bool-mix rejection + `@pre` typing (light validator, `XIOM_STRICT_CLAUSES=1`
  strict mode; stdlib verified green under it); `@pre`-on-method-call runtime
  snapshot (`as_source...`/pre-vars collector); unsigned CONSTANT + FIELD
  sign-extension (`as_source_is_signed`); **Result payload `&r.value` ->
  boxed pointee** (`f2a14f07`, lock `e2e_m140_result_payload_ref`);
  **`XIOM_STRICT_BRACKETS=1`** transition switch (`bbb6bf0b`, lock m141);
  benchmark-chaos tests relocated + e2e harness deletes outputs; Stage 6
  fmt-peek perf budget + `docs/STAGE6_LINT_WAVE.md` lint-wave spec.

## Remaining work order (compiler lane)

1. **`ptr.is_null()` silent-stub kill** (live: `xiom.cell` `Ref.release` /
   `RefMut.release` at 155/181; METHOD form emits NO call -> false; direct
   `is_null(ptr)` is correct). Plan in COMPILER_BUGS "Stdlib relay" -> NEXT
   BATCH: (a) `emitter.rs::emit_undefined_symbol_stubs` collects unresolved
   called symbols and the compiler fails loudly (C001 naming the symbol)
   instead of synthesizing `ret 0`; (b) checker aligns FIELD receivers with
   the LOCAL "cannot call 'X' on this expression" error and adds the UFCS
   lookup (receiver as arg 0, ref-ish incl.); (c) codegen same UFCS lookup;
   (d) lock: null pointer field, direct+method both true, non-null control.
   Stdlib workaround until then: direct `is_null(ptr)`.
2. **By-value receiver Vec mutation**: `fn S.add(self)` with `self.v.push(x)`
   loses the mutation (`tmp/sprintc/pkg_e001_accessor.xi`); the receiver is
   already a pointer in IR, so audit the container-FIELD access path for
   by-value-self methods, fix, lock, e2e.
3. **Re-land exact arity** only after the stdlib wave fixes its call sites:
   checks are documented in-line at three sites (bare ~6810, method offset
   ~6246, module-prefix ~4065); stdlib list: io `printf` x3,
   `_scrypt_blockmix` x2, `collections.get`, `path.replace`; `xiom.cell` is
   OURS (item 1). Flip the three `>` to `!=`, corpus + e2e.
4. **Pin bump + release** (blocked on the stdlib's 100%): stdlib release
   carrying `release-notes/v0.62.0.md` -> set `stdlib/` to that ref + bump
   `STDLIB_VERSION` -> stdlib-dependent gates on the pin -> flip the
   `XIOM_STRICT_BRACKETS` default (and the clause default once they confirm)
   -> re-convert notes -> version bump 0.61.3 -> 0.62.0 -> push (FIRST CI
   since R66; Windows leg unverified) -> tag -> registry canary -> website
   notes. Then Stage 6 (lint wave first, fmt-peek restructure next), Stage 7.

## Cross-lane status (2026-09-25)

- **stdlib:** wave 31 complete (factorial contracts; modules 509/509, corpus
  951/951, probes 183/183, barename 0/509); wave 32/33 queued plus fix-first
  probes; their main canonicalized the 13 remaining mixed-bracket sites and
  re-checks clean; they need `XIOM_STRICT_BRACKETS=1` self-verification and
  the pin; arity call-site list outstanding; strict clause mode verified
  green on their main. Trap-4 re-runs belong to packages (no aiff port there).
- **packages:** trap 4 re-run happened on the RELEASED 0.61.3 (fix staged for
  the next build; their README/SESSION pin it); 0 mixed-bracket sites; trap 14
  extended (parameter/local bracket laxness); traps 16/17 informational.
- **website/playground:** relays delivered (commits `6424002` / `9fc587f`):
  semicolon rule docs/lessons, crash-exit reporting, `--explain` cwd gap
  (Stage 6 item), diagnostic-code corrections actioned on our side
  (`b5b816b0`).

## Environment / method

- Repo-local identity `Lefteris Notas <lefterisnotas@gmail.com>`; the website
  repo requires `git commit -s` (DCO).
- CI does NOT run on main pushes (PR + dispatch only) -> the release push is
  the first CI run since R66; the Windows leg is the main risk.
- Never rebuild `target/debug/xiom.exe` while a cargo e2e runs; full e2e is
  ~30 min; run it ONCE per batch; commit atomically; run `python
  tools/ascii_guard.py check` on STAGED files before every commit (chain with
  `if ($?)` so a guard failure blocks the commit).
- Stale `%TEMP%\kilo\*` stdlib trees cause W001 module-collision noise --
  delete them.

## Continuation prompt (paste into the next session)

> Continue the XIOM SWARM. Read the top section of SESSION.md (the 2026-09-25
> compiler-lane handoff above), docs/FRONTEND_AUDIT.md, docs/COMPILER_BUGS.md
> (the 2026-09-24/25 sections) and docs/STAGE6_LINT_WAVE.md before touching
> code.
>
> State: v0.61.3 released; `stdlib/` detached at `stdlib-v0.61.3`; local
> `main` carries the whole campaign UNPUSHED (50 commits); tree clean; full
> e2e 2377/2377 (+4 ignored); no red gates. The combined v0.62.0 release
> waits on the stdlib's 100% + tag handover (order it per "Remaining work
> order" item 4; do NOT bump the version or tag before their ref lands).
> Sprints A-D are DONE -- do not redo them.
>
> Start with item 1: the `ptr.is_null()` silent-stub kill (emitter stub pass
> -> C001, checker field-receiver alignment + UFCS, codegen UFCS, lock), then
> item 2 (by-value receiver Vec mutation). Item 3 (exact arity) re-lands only
> after the stdlib fixes its call sites; `xiom.cell`'s `is_null` is ours.
>
> Method (non-negotiable): repro-first with a minimal probe under
> `tmp/sprintc/`; fix with locks (e2e fixture `tests/regression/mNNN_*` +
> test + CI line for e2e-able behavior; `crates/xiom/tests/checker_locks.rs`
> for compile-fail and env-switch behavior); run the full e2e ONCE per batch
> (~30 min, no rebuilds during); `python tools/ascii_guard.py check` green
> before every commit; commit atomically with evidence in SESSION.md and
> docs/COMPILER_BUGS.md; update the cross-lane lists whenever a fix changes
> what stdlib/packages must do.
>
> Environment: identity `Lefteris Notas <lefterisnotas@gmail.com>`; pushes
> only when the owner asks (the release push is deliberate); never rebuild
> target/debug while e2e runs; delete stale `%TEMP%\kilo\*` stdlib trees.

# CONTINUATION HANDOFF (2026-09-24, compiler lane)

Supersedes the 2026-09-23 header. The 2026-09-16 handoff and the dated
timeline below are the detailed log -- read the newest entries for evidence.

## Status snapshot

- **Released:** compiler **v0.61.3** (release run 35889350771, all platforms +
  VSIX 0.12.0 live on both marketplaces). Compiler matches stdlib v0.61.3;
  `STDLIB_VERSION` pins `stdlib-v0.61.3`.
- **Git:** `origin/main` = `1f7a9fbc` (R66 + its handoff note are PUSHED).
  Local `main` carries the whole campaign ahead of origin: R67+R68, R69, R70,
  R71, R72/m127, m125 (freeze + CI gate), m126 (deterministic publish bytes +
  `publish --tarball`), m128 (publish sends the registry `compiler` field),
  the front-end audit (`docs/FRONTEND_AUDIT.md`), the website release-notes
  publisher (`crates/xiom-release-notes`), the docs/handoff commits, and the
  Sprint A front-end batch (shared toolchain probe + doctor v2 + FE-8/FE-9).
  Tree clean.
- **stdlib checkout:** `stdlib/` is now **detached at the pin**
  `stdlib-v0.61.3` (`c7b4027`). It had been 6 commits stale (`385e1e4`);
  Phase 0 refreshed it and re-ran the stdlib-dependent suites on the pin.
- **No red gates.**

## Gates (latest evidence)

- Full e2e **2376/2376 (+4 ignored)** -- re-run ON THE PIN for the
  packages-relay batch (2026-09-25, 1980.4 s; adds e2e_m138). The
  non-blocker batch ran 2375/2375 (+4 ignored, 1809.8 s); the relay batch
  2379/2379 (1871.5 s); Sprint D 2377/2377 (1449.5 s); Sprint C 2377/2377
  (1512.3 s); Sprint B 2374/2374 (1691.6 s); Sprint A 2373/2373 (1851.5 s).
- checkers: checker 195/195, parser 102/102, integration 130/130,
  feature-reg 510/510, robustness 63/63, fuzz 24/24.
- stdlib-dependent (re-run on the pin): api-freeze **2/2** (m125 regen),
  stdlib-exec **85/85 (+2 ignored)**.
- tooling: pkg **75/75** (m126 + m128 + FE-9 alias), release-notes **6/6**
  (new), mcp 39/39, ascii_guard OK; new front-end locks: toolchain 7/7 +
  doctor 10/10 unit tests and `doctor_cli` 2/2 (CI step "Front-end CLI
  lock").
- pre-flight for the combined release (2026-09-24, on the current pin):
  workspace lib scan clean, integration 130/130, feature-reg 510/510,
  robustness 63/63, fuzz 24/24, api-freeze 2/2, stdlib-exec 85/85 (+2
  ignored), stdlib modules 40/40, mcp 44/44, pkg 75/75, release-notes 6/6,
  perf 3/3; release-notes/v0.62.0.md+json drafted and `verify`-green
  (fragment-free; re-convert after the pin bump).
- CI: ubuntu-latest green on the pushed R66 state. The Windows CI leg has NOT
  been re-run since R66 (pushes to main do not trigger CI) -- the next push/PR
  validates it; R66 removed the previous Windows-only failure.

## Owner decision: ONE combined release (no intermediate tags)

Git free-plan build minutes are charged, so everything ships in a single tag.
Recommended version: **v0.62.0** (user-visible behavior + UX changes).
Ordering:

1. Compiler lane completes **Sprint A (front-end P0)**, **Sprint B (P1)** and
   **Sprint C (fn-value / generic-mono ABI + E001)**, each batch with its lock
   + one full e2e, committed/pushed as we go (pushes are free).
2. **stdlib lane** completes its 100% readiness plan (coverage waves, dedup
   translation units, tzdata phase 2, untested-surface classes, stub
   re-triage), adds `release-notes/<tag>.md` for the new tag, and cuts ITS
   release.
3. Compiler bumps `STDLIB_VERSION` to the new stdlib tag, runs the full gate
   list on the pin, bumps the workspace version to `0.62.0`, authors
   `release-notes/v0.62.0.md` (the release workflow's `verify` gate REQUIRES
   the committed JSON before the tag), pushes, and tags.
4. **Registry** re-canary after the tag. **Website lane** publishes the notes
   JSON to `dl.xiom-lang.org/releases/<tag>/release.json` and sets
   `"notes": true` in `releases/index.json` (the dispatch now carries
   `client_payload[notes_path]`).

TLS/schannel FFI hardening (stdlib B-list, gates any HTTPS/TLS claim) must
ship in this same window or be explicitly excluded from the release notes.

## Queue, in order

**Sprint A -- front-end P0: LANDED (2026-09-24)** -- see the "Sprint A
landed" note above. FE-1..FE-5, FE-7, FE-8, FE-9 complete (+FE-16 via the
embedded pin); FE-6 (`doctor --deep`) deliberately deferred to Sprint B.

**Sprint B -- front-end P1: LANDED (2026-09-24)** -- see the "Sprint B
landed" note above (FE-6, FE-10..FE-15, FE-17 complete).

**Sprint C -- fn-value / generic-mono ABI unification** (highest correctness
item; one batch or split by root cause, each with locks):
- m127 residuals: `Vec[fn].new()+push` elements, element-to-local calls,
  fn-element calls through struct fields (repros `tmp/probe_p1/`).
- packages lane's 3 confirmed probes (`tmp/fp_probe/`, originals under
  `%TEMP%\kilo\fp-probe\`): fn-ptr struct field (clang ptr/i64 mismatch),
  generic `[T,U]` fn-ptr with U=Str (corrupt Str), generic `Vec[U]` map
  (mis-written elements; AV for U=Str).
- stdlib lane's cross-type callback matrix (their `tools/known_failures/`):
  cross-type returns wrong (Int->Str by-ref/by-value, Int->Float64,
  `sort_by_key[Int,Str]`); same-type and concrete correct; Str->Int correct.
  Root cause: the call/return ABI must follow the MONOMORPHISED
  instantiation, not the erased generic signature.
- E001 conservatism: no warning for consumed temporary borrows; never weaken
  the genuine-overlap warning (their
  `tools/probes/evidence/p_e001_borrow_conservatism.xi`).
Specs: `docs/COMPILER_BUGS.md` (m127, R66-R72) + the 2026-09-24 timeline
entries.

**Sprint D**: LANDED (2026-09-24) -- `xiom toolchain check [--json]` and the
MCP `get_contracts`/`search_symbols` tools shipped (see "Sprint D landed").
The `update`/`rollback` half still needs in-process provenance-attestation
verification (dependency procurement first); it refuses with exit 3 and the
status is recorded in `docs/POST_RELEASE_PLAN.md` section 1.

**Sprint E**: Stage 6 (performance program) and Stage 7 (selfhost gate).
**Stage 5 is CLEAR** (compiler lane). Starting points, from the Stage 6/7
notes: Stage 6 has its gate (`crates/xiom-codegen/tests/perf_budget_tests.rs`:
IR byte budgets + 180s ceiling + byte-identical determinism canary, CI-wired)
and one landed step (deterministic `pick_deterministic` selection); its
continuation items are the real incremental engine (only reachable functions
re-emitted, per-body snapshot cache) and parallel codegen.
**Stage 6 also has a queued work order now: the control-flow lint wave --
`docs/STAGE6_LINT_WAVE.md` (W002 unconditional recursive cycle from the
2026-09-25 owner playground probe, W003 unreachable-after-divergence, W004
unreachable match arm, tier-2 W005-W007; warning-only, user-program scope).**
Stage 7 is the
selfhost ladder (v092..v11 milestone emitters, not yet a self-compiling
chain). Both are multi-session programs; scope the first batch from the
Stage 6 notes before touching code.

**Small cleanups** (from older handoffs): DONE (2026-09-24) -- the
`xiom-benchmark-chaos` e2e tests t2-t5 no longer point at the removed
monorepo checkout (relocated to the internal `tests/ecosystem/t*.xi` copies,
ignored with a reason; compile coverage stays in `e2e_i2_parallel_codegen`),
and the e2e harness deletes its per-invocation outputs (binary + .wasm).
Legacy-package migration note/codemod remains the packages lane's policy.

## Cross-lane status

- **stdlib:** m125 freeze regen CLOSED independently on their side (2/2 on
  compiler main). They re-probed R67/R68/R69/R70/R72 and confirmed the
  still-open `Vec[fn]` literal / `.new()+push` AVs; their green lock
  `p_r70_pending_shapes.xi` is parked for the pin-bump commit. They still owe:
  the full readiness plan, a `release-notes/<tag>.md` fragment for the next
  release, and (optional) minimal repros for checker E001 + the generic
  fn-pointer limitation (now delivered for the latter).
- **registry:** policy settled at their `5c1cdcb`: deterministic assets make
  re-runs byte-stable; re-canary after any asset regeneration or packing
  change. m126 delivered the client-side half of exact-bytes promotion
  (deterministic writer + `publish --tarball`, printed SHA256) and m128 now
  sends the per-version `compiler` field (manifest `compiler:` or
  `--compiler <tag>`; warns when absent) so the website can correlate
  packages with toolchain releases. Nothing pending from ops; production
  publish waits on the owner's environment approval. Open: FE-9 name
  alignment (`xiom.std` vs `xiom-std`); package repos should add `compiler:`
  to their manifests.
- **packages:** 3 repros confirmed on our build; they will commit the probes
  under `docs/repro/`. Their migration-note request (old dialect) is policy,
  not a parser bug; the `xiom.ffi` triage abort is harness-side.
  - Relay 2026-09-25 (traps 12-13): the widened-UNSIGNED-CONSTANT
    sign-extension (`239u8 as Int` = -17) was NOT previously tracked; FIXED
    this batch (`as_source_is_signed`, lock `e2e_m138_u8_const_widen`). The
    `&mut` call-site silent copy (`push_one(v)` mutating a temporary) was NOT
    tracked either; reproduced, OPEN as a design decision (require explicit
    `&mut` vs auto-borrow the place) -- COMPILER_BUGS 2026-09-25 entry. The
    NUL-terminated Str note is a known by-design limitation, now listed in
    `AI_CONTEXT.md`.
  - Relay 2026-09-25 (trap 14 + section 10): arity laxness, `Vec<UInt8]` bracket
    laxness, E001 advisory after an immutable accessor, and `&struct.field`
    trapping only for `&Vec` params -- triage in progress (see the next
    COMPILER_BUGS entry).
  - Relay #2 triage DONE (COMPILER_BUGS 2026-09-25 "Packages relay #2"):
    arity is unvalidated on the primary call path (plain + method; missing
    args silently 0, extra args dropped) -- enforcement implemented, GATED OFF
    until the stdlib fixes 7 call sites (printf, _scrypt_blockmix, get,
    replace, is_null); `Vec<UInt8]` accepted because the parser never checks
    closer-matches-opener -- strict form implemented, GATED OFF until the
    stdlib fixes 18 mixed-bracket sites in 7 files (io/fs x10, io/console,
    io/pipe, core/contracts, math/approximation, test/harness x2, test/test
    x2); the accessor case reproduced as Vec MUTATION LOSS (same copy class
    as the open `&mut` finding), not as an E001 warning; `&struct.field` to
    `&Vec` NARROWED to the Result-payload shape and REPRODUCED
    (`tmp/sprintc/pkg_result_value_ref.xi`; their probe_result_value.xi,
    packages commit 6310dba) -- queued as the next codegen batch alongside
    the by-value Vec-handle copy class; the FIELD sign-extend shape
    (`h.b as Int` = -17) is FIXED; `io.println` Str-only is by design and is
    now stated in `AI_CONTEXT.md`.
  - Strict-bracket switch LANDED (`XIOM_STRICT_BRACKETS=1`, parser + m141
    lock + unit tests; lax default keeps the pin green; full e2e 2377/2377
    (+4 ignored), corpus clean). Stdlib wave 31 canonicalized their 13
    remaining mixed sites; they can self-verify with the switch. Default
    flips at the pin bump. Packages tested trap 4 on the RELEASED 0.61.3
    (fix is in local main; closes on the next build).
  - Relay #2 trap 4 (Result-payload `&field` to `&Vec`) FIXED (`f2a14f07`):
    boxed payloads were addressed at the handle SLOT instead of the pointee;
    `take(&r.value)` now reads the real Vec and `&mut r.value` mutation
    reaches the box. Lock `e2e_m140_result_payload_ref`; full e2e
    2377/2377 (+4 ignored). Remaining in the copy class: by-value RECEIVER
    methods on Vec fields (`S.add(self)` losing `self.v.push`) -- next batch.
- **website:** schema-v1 contract implemented on our side (publisher + gate +
  CI tests); requirement 5 (dl + `releases/index.json` `notes: true`) is
  their lane; the dispatch now carries `notes_path`.
  - Playground UX observation (2026-09-25, owner probe): a program that
    crashes (e.g. infinite mutual recursion -> stack overflow, exit
    `0xC000001D`) is reported as "Program ran with no output". The runner
    should surface the non-zero/crash exit instead of only the empty stdout.
    Not a compiler bug (repro in the relay below); website-lane fix.
  - Semicolon rule relay (2026-09-25, owner decision: KEEP option (a), the
    Rust-like tail form): website syntax docs + playground starter should
    state that `;` ends statements and ONLY a block's final value expression
    may omit it, and starter/multi-statement examples should use `;` on every
    statement. A `P001` note pointing at the missing separator is queued in
    Stage 6 (`docs/STAGE6_LINT_WAVE.md`, companion polish); `AI_CONTEXT.md`
    and the MCP language guide were updated on our side (commit in this
    batch).
  - Diagnostic-code drift (website request 2026-09-22) ACTIONED on our side:
    `AI_CONTEXT.md` no longer claims an emitted `X` family (L/P/T/E/C/W, T
    umbrella; runtime contract violations print without a code; X reserved),
    the MCP `explain_error_code` example is `T001`, its fallback maps `W` and
    labels `X` reserved, and the `--explain` unknown-code hint no longer
    suggests `X0010`; `docs/AI_PIPELINE.md` carries a historical note.
  - NEW compile-side gap found while fixing that: `xiom --explain` and the
    MCP `explain_error_code` resolve `docs/error_codes/<code>.md` from the
    CURRENT DIRECTORY, and the compiler repo has no such directory -- the
    reference only works inside a checkout that contains the pages (e.g. the
    website repo), not from a user project or an install. Stage 6 item:
    stage the pages into `lib/docs/error_codes/` in the release archives
    (release.yml fetch) or embed a minimal index; coordinate with
    website/ops. Recorded in the relay to website.
- **benchmark:** `COMPILER_VERSION` still pins the absent v0.61.0 -- they
  should bump after our next release exists.

## Environment / method notes

- **Identity**: gh active account must be **Lefteris-Notas**
  (`gh auth switch --user Lefteris-Notas`). Repo-local git identity is
  `Lefteris Notas <lefterisnotas@gmail.com>` (global differs; leave it).
- **CI does NOT run on pushes to main** (PR + `workflow_dispatch` only).
- **Never rebuild `target/debug/xiom.exe` while a cargo e2e runs.** One e2e
  is ~23-28 min; batch fixes per run.
- **`--emit-ir` prints INTERMEDIATE output.** For the IR clang compiles,
  force a link failure (`--link missing_xyz`) and read `<output>.ll`.
- **Large fixed arrays** (>= 16 KiB) are memset-zeroed and accessed by
  address (`LARGE_ARRAY_MIN_BYTES`); clang 22 ISel crashes above 32 KiB.
- Repro harnesses: `tmp/repro_lessons.ps1`, `tmp/output_check.ps1`;
  playground clone `tmp/playground`.
- Stdlib override for a NEWER stdlib: set BOTH `XIOM_STDLIB` and
  `XIOM_RUNTIME_DIR` (runtime C resolves separately).
- **Test paths must use `/`**; the e2e stdlib guard accepts both layouts.
- WSL/Linux CI proxy: `XIOM_REQUIRE_STDLIB=1` + the curated `ci.yml` list.
- Bench harness on Windows: set a clean `TMP`/`TEMP` first.
- **ascii_guard**: staged files must be pure ASCII
  (`python tools/ascii_guard.py check`).
- Delete extracted release archives after verification (stale stdlib trees
  trip `e2e_m17_zero_warnings`).
- ISel/LLVM minimizer: `tmp/extract_closure.py` + `tmp/preprobe/`.
- Local repro dirs: `tmp/probe_p1/` (R66-R72 + m127), `tmp/fp_probe/`
  (packages fp probes), `tmp/relnotes_smoke/` (release-notes CLI smoke).

## Continuation prompt (copy/paste into the next session)

> Continue the XIOM compiler-lane campaign in `E:\xiom-lang\xiom` (branch
> `main`; push only when asked). Read the top section of SESSION.md
> ("CONTINUATION HANDOFF (2026-09-24, compiler lane)") plus
> `docs/FRONTEND_AUDIT.md` (the owner-approved P0/P1 backlog) and
> `docs/COMPILER_BUGS.md` (R66-R72, m127, Sprints A-D) before touching code.
>
> State: compiler v0.61.3 released; `STDLIB_VERSION` pins `stdlib-v0.61.3`
> (the local `stdlib/` checkout is detached at that tag); `origin/main` has
> R66 only, with the whole campaign unpushed on local `main` (R67-R72, m125,
> m126, m128, the audit, the release-notes publisher, Sprints A-D, docs).
> Full e2e 2376/2376 (+4 ignored) on the pin (packages-relay batch);
> api-freeze 2/2 and stdlib-exec 85/85 on the pin; no red gates. The owner
> batched everything into ONE release (no intermediate tags; recommend
> v0.62.0), sequenced as: compiler Sprints A+B+C -> stdlib completes its plan
> + cuts its release + ships its release-notes fragment -> bump
> STDLIB_VERSION -> full gates on the pin -> author release-notes/v0.62.0.md
> (the release workflow's `verify` gate requires the committed JSON BEFORE
> the tag) -> version bump + push + tag -> registry re-canary, website
> publishes the notes to dl.
>
> Sprints A-D are LANDED -- see "Sprint D landed" above; do not redo them.
> OWNER SEQUENCE (2026-09-24): ship the combined release BEFORE Stage 6.
> The compiler side is release-ready -- see "Release pre-flight" above; it
> waits only on the stdlib lane (their plan -> their release with the
> release-notes/v0.62.0.md fragment -> the ref). If the pin has landed, run
> the release sequence (bump STDLIB_VERSION -> stdlib-dependent gates on the
> new pin -> re-convert notes -> version bump -> push -> tag -> canary);
> otherwise start Stage 6's non-blocking prep. Stage 5 is CLEAR. The Stage 6
> gate exists at `crates/xiom-codegen/tests/perf_budget_tests.rs`; the real
> incremental engine + parallel codegen and the Stage 7 selfhost ladder are
> the open work.
>
> Method (non-negotiable): work repro-first; rebuild `cargo build -p xiom`
> after checker/codegen changes; add an e2e lock (`e2e_mNNN_*` fixture + the
> CI lock line in `.github/workflows/ci.yml`) for every compiler behavior
> change; run the full e2e ONCE per batch; never rebuild while an e2e runs;
> keep `python tools/ascii_guard.py check` green; commit atomically with
> evidence in SESSION.md and docs/COMPILER_BUGS.md.
>
> After Sprint E: the release sequence in the State paragraph (stdlib
> completes -> pin bump -> full gates -> release-notes/v0.62.0.md -> version
> bump + push + tag -> registry re-canary, website notes to dl).

## Sprint A landed (2026-09-24, compiler lane)

Front-end P0/P1 (FE-1..FE-5, FE-7, FE-8, FE-9; FE-16 via the pin embed) is
implemented and gated. Details + evidence: docs/COMPILER_BUGS.md
("2026-09-24 -- Front-end audit Sprint A").

- `crates/xiom/src/toolchain.rs`: ONE clang/opt/nasm probe for doctor AND the
  driver (PATH first, resolved to absolute paths; then per-OS known locations
  incl. the winget globs; the personal NASM path and `find_tool`/`find_nasm`
  are gone; `opt` follows clang's directory).
- `crates/xiom/src/doctor.rs`: doctor v2 -- identity block (exe/install root/
  XIOM_HOME/stdlib root via `paths::stdlib_root()` + version/tool path+version/
  runtime), warnings (compiler<->stdlib version, stdlib<->embedded pin,
  runtime missing, duplicate installs), OS-specific remediation with no
  `xiom install llvm`, `--json`, exit codes 0/1/2.
- FE-8: `xiom publish` is a deprecation alias for `xiom pkg publish` (legacy
  git-tag flow deleted); `xiom update` points at the reinstall one-liner /
  future `xiom toolchain update`; pkg help example fixed; the old improvement
  plan is marked historical.
- FE-9: `xiom.std` <-> `xiom-std` client aliases; live-verified (`info` both
  spellings -> v0.61.3 sha256 1ad1b33a5caa; `install xiom.hello` end-to-end
  into a temp XIOM_HOME).
- Locks: 7 toolchain + 10 doctor unit tests; `doctor_cli` 2 binary tests
  (CI step "Front-end CLI lock"). Workspace all-targets check clean; ascii
  guard clean; **full e2e 2373/2373 on the pin**.
- Deferred: clang version floor (no agreed floor), `doctor --deep`/`--fix`
  (Sprint B FE-6), installers FE-10..FE-15, grouped help FE-12.

Next: Sprint C (fn-value/generic-mono), then Sprint D, E.
(Landed -- see "Sprint C landed" below.)

## Sprint B landed (2026-09-24, compiler lane)

Front-end P1 (FE-6, FE-10..FE-15, FE-17) is implemented and gated; details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Front-end audit Sprint B").

- FE-17: one optional `;` after a `module` header (brace-less and block form)
  is tolerated; lock `e2e_m129_module_trailing_semicolon` (+CI line) and a
  parser unit test.
- FE-6: `xiom doctor --deep` compiles AND RUNS a trivial program end-to-end.
- FE-12: grouped `xiom --help` (no sprint tags; every flag kept; tool help
  line for the dispatcher).
- FE-13: `xiom --version` prints the install root; `xiom.bat` prefers its own
  directory over a stale LOCALAPPDATA install.
- FE-10/FE-14: installers ship all 9 tools + the archive `lib/` layout and
  finish with `xiom doctor` (`--json` in CI); modern banners. Found and fixed
  a pre-existing `-Unattended` crash (`$registerExt` vs the `$RegisterExt`
  switch parameter).
- FE-15: the generated uninstaller cleans PATH (user + best-effort machine).
- FE-11: unpinned LLVM direct downloads removed from install_deps.ps1/.sh.
- Gates: parser 102/102; full e2e **2374/2374** on the pin; workspace
  all-targets check clean; ascii guard clean.

## Sprint C landed (2026-09-24, compiler lane)

fn-value / generic-mono ABI unification + E001 conservatism; full details and
root causes in docs/COMPILER_BUGS.md ("2026-09-24 -- Sprint C").

- Parser/checker: `Vec[fn() -> Int]` type args keep the fn marker (no more
  `Ident("_")`); `method_target` distinguishes `Type.method[Arg]()` /
  `module.fn[Arg]()` from `value.field[i]()`; array literals of fn refs type
  as `Vec[fn(...) -> R]`.
- Codegen: fn markers in `type_string_full`/`type_annotation_name`, i64-erased
  fn locals, fn-ref markers recorded for array bindings, for-loop fn elements
  called env-first, raw array buffers wrap fn refs, `(op.f)(x)` unwraps to the
  env-first field path.
- Generic mono: fn-typed params infer T/U from the argument's registered
  signature (`fn_arg_generic_binding` + LLVM->XIOM name mapping) -- cross-type
  U=Str / U=Float64 callbacks and `Vec[U]` returns are correct.
- E001: per-statement temporary-borrow release + the `read_borrow_count`
  sentinel fix; `smoke_collect_sparse` 7 -> 0 warnings, genuine overlap still
  warns.
- Locks: e2e m130 (fn-value ABI), m131 (generic callback ABI), m132 (E001
  consumed); `crates/xiom/tests/borrow_e001.rs` (m132/m133 warning contract,
  CI step extended). Probes: 15 probe_p1 + 7 fp + stdlib matrix all green.
- Gates: full e2e **2377/2377** on the pin (1512.3 s); checker 195/195;
  parser 102/102; workspace all-targets clean.
- Open follow-up recorded: the checker accepts a by-ref callback where a
  fn-typed param declares `fn(T) -> U` by value (runtime AV); needs a
  fn-signature compatibility check.

## Sprint D landed (2026-09-24, compiler lane)

`xiom toolchain check` + the MCP structured-contract tools; details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Sprint D") and the status note in
docs/POST_RELEASE_PLAN.md section 1.

- `crates/xiom/src/toolchain_cmd.rs`: GitHub-Releases-only `check [--json]`
  (`current/latest/platform/up_to_date/notes` + `install_kind/asset/exe`),
  exit codes 0/1/2, package-manager + dev install detection; verified live
  (v0.61.3 = latest, exit 0). `xiom doctor` points at the command.
- `update`/`rollback` refuse with exit 3 pending in-process attestation
  verification (dependency procurement); the refusal names `check` + the
  release installer. POST_RELEASE_PLAN records the block precisely.
- MCP: `get_contracts {symbol, verify?, file?}` (structured signature +
  requires/ensures/invariants with source lines; Z3 fold when `verify:true`)
  and `search_symbols {query, file?}` (ranked stdlib + project hits), built on
  the same live stdlib catalog scan as `xiom_stdlib_reference`. 18 tools.
- Gates: full e2e **2377/2377** on the pin (1449.5 s); MCP 44/44 (5 new
  contract tests); workspace all-targets clean; live check green.

Next: Sprint E (Stage 6/7).

## Relay batch landed (2026-09-24, compiler lane)

Two stdlib-relayed compiler findings fixed (the third, the cross-type
callback matrix + E001 repro, landed in Sprint C); full details in
docs/COMPILER_BUGS.md ("2026-09-24 -- Relay").

- Private same-leaf type collision (R44/Timer case): the collision triage now
  includes private type decls, so `xiom.async.Timer` (private) and
  `xiom.async.timer.Timer` (pub) emit distinct qualified layouts instead of
  silently sharing one. Lock `e2e_m134_private_same_leaf`.
- Clause-position `Bool == Int`: clauses were never type-checked; a light
  clause validator now rejects Bool-vs-concrete comparisons (plus an
  `@pre` typing arm). Locks `checker_locks.rs` (m135 reject, m136 green) +
  `e2e_m136_well_typed_clauses`.
- Deferred with precise evidence (needs stdlib-lane clause fixes before the
  compiler can require every clause to be Bool): the 8 stdlib clause sites
  listed in COMPILER_BUGS. Also recorded: `@pre` on a method call has a wrong
  RUNTIME snapshot (typing-only lock).
- Gates: full e2e **2379/2379** on the pin (1871.5 s); checker 195/195;
  catalog corpus clean; workspace all-targets clean.

## Stage 6 increment (2026-09-24, compiler lane)

Sprint E started with the smallest safe bite: the fmt-peek closure shape now
has its own perf budget. `tests/perf/fmt_peek.xi` (Int/Float64 `.to_str()`,
the shape that pulls `xiom.fmt` through `collect_external_decls`'s peek) is
covered by `perf_budget_tests.rs::perf_budget_fmt_peek_shape`: byte budget
95,000 (baseline 80,300) plus a 20 s debug-profile ceiling, so the documented
sweep p50 3.9 -> 7.9 s regression cannot grow unnoticed while the real fix
waits. Perf 3/3.

Stage 6 open work order: (1) the control-flow lint wave
(`docs/STAGE6_LINT_WAVE.md`) -- W002 unconditional recursive cycle (the
2026-09-25 owner playground probe: `a() { b(); print; } b() { a(); print; }`
never prints and dies via the runtime fault trap, exit `0xC000001D`; the
playground reporting "ran with no output" is a website-lane fix), W003
unreachable statement after a diverging statement, W004 unreachable match
arm, then tier 2 (literal `/ 0` -- today it traps at runtime; `1 << 64` --
today runs with a garbage result; non-float self-comparison); warning-only,
user-program scope, two locks per lint; plus the companion P001
missing-semicolon suggestion + beginner-docs pass, and an explicit decision
against newline-as-separator -- see the spec's "Companion diagnostics
polish"). (2) the reachable-function-only peek
restructure (fix shape in COMPILER_BUGS: peek the checker-resolved module
shallow, run the reachability filter, then pull the deps named by the
SELECTED decls to a fixpoint). (3) parallel monomorphization profiles,
linker strategy, more budget metrics. (4) `--explain` reference resolution:
ship `docs/error_codes/` under `lib/docs/` in the release archives (or embed a
minimal index) so the command works outside a checkout that contains the
pages; today it is cwd-relative. Stage 7: the selfhost ladder
(multi-phase).

## Release pre-flight (2026-09-24, compiler lane)

The compiler side is release-ready on the current pin; the only external
dependency left is the stdlib lane's own release.

- Gate battery re-run on the pin after all of Sprints A-D + the relay batch:
  full e2e **2375/2375 (+4 ignored)** on the final tree, checker 195/195,
  parser 102/102, integration
  130/130, feature-reg 510/510, robustness 63/63, fuzz 24/24, api-freeze
  2/2, stdlib-exec 85/85 (+2 ignored), stdlib modules 40/40, mcp 44/44,
  pkg 75/75, release-notes 6/6, perf 3/3; workspace all-targets clean.
- Compiler-side release notes DRAFTED and committed:
  `release-notes/v0.62.0.md` + `.json` (4 highlights, 3 breaking, 2 docs);
  `xiom-release-notes verify --tag v0.62.0` is green fragment-free. The
  stdlib fragment in the JSON is re-converted after the pin bump.
- Remaining sequence (blocked on the stdlib lane): stdlib completes its plan
  -> cuts its release carrying `release-notes/v0.62.0.md` -> hand over the
  ref -> bump `STDLIB_VERSION` -> re-run the stdlib-dependent gates on the
  new pin (api-freeze, stdlib-exec, stdlib modules, full e2e) -> re-convert
  the notes -> version bump 0.61.3 -> 0.62.0 -> push origin/main (FIRST CI
  run since R66 -- the Windows leg is the main unknown) -> tag v0.62.0
  (guard: tag == workspace version + ancestor of main) -> registry
  re-canary -> website publishes the notes to dl with notes: true.
- Not release-blocking: the attested updater (D2 tail, procurement) and the
  legacy-package migration note/codemod (packages lane policy). The earlier
  follow-ups (strict clause default, `@pre`-on-call runtime, private enums,
  e2e/harness cleanups) are landed or switched -- see "Non-blocker batch
  landed".
- Cross-lane update (2026-09-24 evening, stdlib relay): their clause cleanup
  landed (`Rc`/`Arc.new` -> `result.strong_count() == 1`, `ptr.replace` ->
  `dest != null`, `math.pow`/`pow_pure` `as Float64`, `array.is_sorted_by`
  implemented), and STRICT MODE is verified GREEN on their main with our
  built compiler (`XIOM_STRICT_CLAUSES=1 ... catalog_corpus_is_clean` -> 1
  passed). Their notes fragment was trimmed to 2 highlights (our 4 + their 2
  = the schema max of 6). Their release cut is gated on reaching 100%
  (coverage waves, control_theory + lp_simplex, collect/lfu dedup, geom API
  unit, tzdata phase 2, untested-surface generator), per their
  docs/RELEASE_CHECKLIST.md.
- PIN-BUMP ORDERING NOTE: the strict-default flip must land WITH the pin
  bump -- the current `stdlib-v0.61.3` checkout still carries the 8 unfixed
  clause sites and `catalog_corpus_is_clean` indexes the repo `stdlib/`
  checkout directly (it does NOT honor XIOM_STDLIB), so flipping now would
  red the corpus gate. Sequence at the bump: set `stdlib/` to their release
  ref -> bump `STDLIB_VERSION` -> `XIOM_STRICT_CLAUSES=1 cargo test -p
  xiom-check catalog_corpus_is_clean` (expect green) -> remove the env gate
  in `check_clause_bool_mix` (default becomes strict) -> re-run the
  stdlib-dependent gates + full e2e -> re-convert the notes -> version bump
  -> push -> tag.

## Non-blocker batch landed (2026-09-24, compiler lane)

The known non-blocking items are cleared or moved behind a switch; details in
docs/COMPILER_BUGS.md (relay follow-ups, status 2026-09-24 evening).

- `@pre` on a METHOD CALL -- FIXED at runtime: the pre-state collector
  recorded the callee name instead of the receiver, so `len()@pre` read the
  live length (ensures `len() == len()@pre + 1` fired although the length grew
  by exactly 1). Bare callee idents are no longer collected as variables, and
  a method with any `@pre` always snapshots `self`. m136 now runs the real
  clause at runtime.
- Full predicate-Bool clause checking -- TRANSITION SWITCH:
  `XIOM_STRICT_CLAUSES=1` enables the strict rule while the default stays
  light. All 8 stdlib clause sites were triaged as GENUINE stdlib issues (no
  checker gaps): ptr 91 undefined `old_value`, math 201/561 Float64-vs-Int,
  sync 378 / rc 29 bare `strong_count` without `()`, array 219 nonexistent
  `is_sorted_by`. The stdlib lane verifies with
  `XIOM_STRICT_CLAUSES=1 cargo test -p xiom-check catalog_corpus_is_clean`;
  when that is green, flip the default (one condition in
  `check_clause_bool_mix`). Fixture m137 + checker lock cover both modes.
- Private ENUM same-leaf triage -- landed: pub AND private enums now
  participate like types; identical layouts keep the shared key.
- Cleanups -- benchmark-chaos tests relocated to the internal copies and
  ignored (compile coverage stays in `e2e_i2_parallel_codegen`); the e2e
  harness now deletes its per-invocation outputs.
- Gates: full e2e **2375/2375 (+4 ignored)**, checker 195/195, catalog corpus
  clean, workspace all-targets clean.

# XIOM Handoff -- 2026-09-16 (compiler lane; rounds 61-83 in docs/SESSION.md)

2026-09-17 update (post-split, `main`): the registry-client findings
R32-R38 are FIXED -- terminal integrity gates on install, canonical
registry URLs + trust-store normalization, surfaced registry error bodies,
yanked-package handling, XIOM_HOME-aware package cache. Details and the
verification record are in docs/COMPILER_BUGS.md ("Registry-client
integration findings"). The registry repo's e2e harness gained the matching
locks (cache path + tamper/error-code/all-yanked assertions) -- test-only,
owned by the registry lane. Remaining supply-chain tail: transitive
dependency closure from registry metadata.

Also 2026-09-17: R39 FIXED -- same-leaf TYPE collision across project
modules (catalog injection now module-qualifies colliding non-generic type
leaves and rewrites all references; `crates/xiom-check/src/type_qualify.rs`,
lock `e2e_m84_type_same_leaf_modules`, CI lock line updated). The bench graph
no longer emits a bare `%struct.Metrics`/`%struct.Record`; its IR is
deterministic at 5,764,620 bytes.

Also 2026-09-17: R40 FIXED -- `derive[Clone]` on a pointer receiver
(`m: &M` -> `m.clone()`) passed the pointer where the by-value `%self` was
expected; LLVM accepted the mismatch silently and the clone was garbage. The
non-generic instance-method receiver coercion now loads the struct when the
callee's first param is a by-value struct (call.rs), lock
`e2e_m85_clone_ref_receiver` + CI lock line.

Also 2026-09-17: R41 FIXED -- generic-method calls passed struct VALUES where
a pointer self was expected (bench `Pair.read_first_Int_Int`); the generic
receiver ABI block now mirrors the non-generic materialization. R42 FIXED --
bare enum variants resolve scope-first (checker parity) so
`var c: Container[Int] = Empty` binds Container, not the first-declared
`Message`. See docs/COMPILER_BUGS.md R41/R42.

Also 2026-09-17 (stdlib-session sweep relay): R43 FIXED --
`&v` on a reference-typed local hard-errored C001, breaking the stdlib's
`var v = b; ... &v` pattern (x25519_keypair -> _bigint_to_le); `&*v == v`
now lowers to the stored pointer. Lock `e2e_m86_ref_of_reference`.
R44 SLICE LANDED (2026-09-18) -- `http_parse_response` invalid GEP was a
stdlib same-leaf collision; the stdlib renamed `net.net.HttpResponse` ->
`NetHttpResponse` (ce0c7fa) and the compiler now includes catalog modules
in the collision qualification under the shape-conflict standard
(all-catalog groups only), so the collision cannot resurface: pin probe +
negative control, lock `e2e_m90_stdlib_same_leaf_http`, corpus 949/949.
R45 FIXED -- the single-param sweep's tuple mismatch
(tuple element names came from LLVM widths: Bool -> "Int", `as UInt16` ->
"Int16") plus the early-splice requirement (LLVM rejects alloca of a
forward-referenced named type). Lock `e2e_m87_tuple_element_types`; sweep
repro now compiles/links/runs. See docs/COMPILER_BUGS.md R43-R45.

Also 2026-09-17: STAGE 5 SUPPLY-CHAIN TAIL CLOSED -- transitive dependency
closure for registry installs (version-range matcher; cycle-safe closure
whose dependency source is the verified tarball's manifest; every artifact
verified) and transitive lockfile pinning (`Lockfile::from_resolved`).
Registry e2e 20/20 with a real dependency fixture. Driver hygiene landed:
temp names randomized per invocation/session; the stage-5 plan entries for
sandbox false-green (AUDIT #11), library `process::exit` (AUDIT #12), dbg MI
quoting (AUDIT #20) and fmt `defer` (AUDIT #10) were already closed and are
now recorded as DONE. Stage 5 remaining (feature-scale): clap-based arg
parsing, LSP incremental reparsing + cross-file index, fmt body-inline
comment trivia, cargo-vet audits.

2026-09-18 update (post-split, `main`): R46 LANDED -- the bench graph is
**CLANG-CLEAN** (`xiom --emit-ir examples/benchmark/main.xi | clang -c -x ir
-` exits 0; deterministic at 5,808,645 bytes; locks m87-extended/m88/m89).
The stdlib lane verified R43/R44/R45 probes and check_modules 509/509 on
483f283e, and reported the full unmodified sweep as "1 fast clang failure +
3 hangs >5-15 min". Re-measured here: with the RELEASE driver the sweep's
codegen+clang finish in **481.6s** and fail only at link
(`undefined symbol: unsetenv`, the stdlib Windows gap); the debug driver
exceeds its 300s watchdog, which is what looked like a hang. No compiler
hang. Remaining OPEN compiler finding: none from the R46 family -- R46b
(2026-09-18) closed the cross-module qualified-receiver and receiver-only
generic method instantiation residuals; m88 now locks the direct call forms
(wrappers removed). **R44 qualification slice LANDED (2026-09-18)**: catalog
(`xiom.`) modules now participate in the R39/R46 collision triage under the
stdlib audit standard -- all-catalog groups qualify only when declared
shapes conflict (facade duplicates keep the legacy key); project-owned
groups keep the R39/R46 rule, so user emission is untouched (bench IR
byte-identical at 5,808,645). Verified: `p_http_resp_codegen` green on the
pin (negative control reproduces the clang GEP error), check_modules
509/509, smoke corpus 949/949 on stdlib main, stdlib_exec 85/85 on the pin,
lock `e2e_m90_stdlib_same_leaf_http`. Remaining stdlib-side work: their
16-group dedup worklist (hygiene now, not a correctness prerequisite).
Stdlib status
for the release lane: their pin is v0.60.0 and release tag v0.60.1 predates
R43/R45/R46, so `COMPILER_VERSION` moves only when a release contains them
(their nightly already tests main).

Branch `main` (post-split). The round-83 slice (pre-split housekeeping +
handoff) and earlier rounds live in the pre-split history; the R32-R46
hardening slices are the latest compiler-lane commits.
Working tree should be clean; `.xiom_ai.json` is generated tooling state and
is now untracked/ignored (it has been committed before -- `git rm --cached`
in this round). In the monorepo phase the parallel stdlib lane committed to
the same branch and sometimes swept the whole tree (re-check `git log --stat`
if a change seems missing); after the split the stdlib is a separate repo and
its checkout at `stdlib/` is gitignored.

## Current state (2026-09-19, post-R48) -- READ FIRST

Everything after this section is the pre-R31/r31-r83 history. Live state:

- **Stage 5 is COMPLETE**: clap owns the flag surface (steps 1+2), cargo-vet
  gate (175 exempted), fmt body-inline comments, LSP cross-file index +
  parse cache, driver temp hygiene, help parity (full `--help`, `--strict-mode`
  alias), CRB-1..CRB-5 done with the ops batch, AI-mode audit (installer
  config fixed, `--ai-local` enforced, HTTPS/plaintext key guard, structured
  `FIX:/WHY:/Confidence:` hint fields).
- **Compiler bugs cleared through R48** (docs/COMPILER_BUGS.md is the
  authority). Fixed in R48: interface dispatch through `&T`/`&mut T` generic
  args (9 playground L6 lessons), pointer/double match-result zero-init,
  script-cache build identity, native `xiom fmt|lsp|mcp|pkg|dbg|verify|ffigen`
  dispatch, WASM release asset. Lock `e2e_m92_interface_dispatch_zero_init`.
- **R49 batch LANDED (2026-09-19)**: playground R48-residue fixes --
  L6-28 (struct-literal generic inference + annotation type args + concrete
  Option registration + boxed struct-element load/unbox + clone type),
  L6-31 (duplicate definition dedupe), L8-15/L8-18 (computed Vec receivers,
  consistent box/unbox, inline set memcpy), L6-05 (mono param Vec element
  types), L2-19 (qualified enum variants), L0-11/array-of-Str (i8* element
  type), L5-32/L5-34 (match slots keep Str + Some/Ok payload unbox),
  L5-42 (bare conversion-call return type), and the stdlib-relayed
  `@pre` call-capture bug (all-ident collection + pointer-slot rebind for
  ref params + Vec-buffer deep snapshot). Locks `e2e_m93..m98` + CI lock
  line; fixtures `tests/regression/m9[3-8]_*`.
- **Verified on the R49 batch**: full e2e **2346/2346** (locks m93-m98
  included), checker 195/195, feature-reg 510/510, and the
  `stdlib_api_freeze_tests` gate GREEN after the R49-1 resolver fix +
  snapshot regen. The earlier `56b6e0e3` record (2340/2340, feature-reg
  510/510, checker 195/195, robustness 63/63, fuzz 24/24, perf/determinism
  2/2, fmt 86/86, lsp 45/45, stdlib-exec 85/85 (+2 ignored), xiom lib
  32/32, pkg/dbg/mcp 63/34/39, `cargo deny` + `cargo vet` clean, ASCII
  guard, bench IR 5,808,645 bytes + `clang -c` exit 0) still holds for the
  unchanged lanes; workspace version 0.61.0.
- **Release lane**: `release.yml` ships all nine CLI tools + pinned z3
  (tools/z3-pins.json, glibc floor 2.35) + `xiom-wasm-<ver>.wasm`; guard
  (tag == workspace version, ancestor of main) and the `compiler-release`
  docs dispatch (`XIOM_RELEASE_TOKEN` must cover xiom-lang/website) are
  wired. Pending: stdlib lane green -> bump `STDLIB_VERSION` in the release
  PR -> tag `v0.61.0`; optional rewrite of the protected tags
  `v0.60.0`/`v0.60.1` (old objects remain on the remote).
- **R54 / R49-4 CLOSED (2026-09-21)**: the clang 22.1.8 ISel crash is fixed
  in codegen -- large fixed-array locals (>= 16 KiB) are zero-initialized
  with `llvm.memset` instead of an aggregate `store zeroinitializer`, and
  indexed access uses their address (no whole-array value materialization).
  `p_sweep_single_param.xi` compiles AND links in ~56 s (previously clang
  ISel 0xC0000005); lock `e2e_m109_large_fixed_array`. Windows-pin note:
  the pinned stdlib's `os/env.xi` still calls `unsetenv` directly, so the
  probe links on Windows only after the pin moves to the
  `xiom_env_set/unset` shim revision (bump `STDLIB_VERSION`).
- **CRB-6 / release dry run (2026-09-21)**: `release.yml`'s three z3 bundle
  steps now resolve the vendor license by NAME under the extraction tree
  (`find`/`Get-ChildItem -Recurse`) instead of assuming `LICENSE.txt` sits at
  the archive root -- both platform jobs had failed with `cp: cannot stat
  '/tmp/xiom-z3/LICENSE.txt'`. Committed `30c5a483`, pushed. Dispatch
  35647022547 is fully green: guard, linux-x64, windows-x64, VSIX
  (`xiom-vscode-0.12.0.vsix` + `SHA256SUMS-vscode`); marketplace publishes
  and the GitHub Release are skipped (tag-gated). No tag yet.
- **IDE distribution + release batch (2026-09-21, registry relay via owner)**:
  (1) the VS Code VSIX is now UNIVERSAL -- `program: ./xiom-dbg.exe` is gone
  from `editors/vscode/package.json`; `xiom-lsp`/`xiom-dbg` resolve at runtime
  (`xiom.debugAdapterPath` / `xiom.lsp.path` -> PATH -> workspace `target/`;
  `xiom.dbg.path` is a deprecated alias) and the bundled-binary fallback is
  removed. Verified locally with `vsce package` (9 files, 70.55 KB, no .exe).
  (2) First-run UX: activation verifies `xiom`/`xiom-lsp`/`xiom-dbg` and shows
  a notification linking to xiom-lang.org/install plus `XIOM: Recheck
  toolchain`; the toolchain is NEVER installed silently.
  (3) release.yml gained the `vscode` job: universal VSIX + SHA256SUMS ride on
  the GitHub Release; `vsce publish`/`ovsx publish` run only on tag builds and
  only when `VSCE_PAT`/`OVSX_TOKEN` are present; the release job waits on it.
  (4) `xiom --version`/`-V` always reports the workspace version; the stale
  hardcoded stats/tag defaults are gone (XIOM_RELEASE_TAG/STATS are optional
  build-time stamps) and `_build_linux.sh` no longer stamps ancient values.
  All nine tools self-report v0.61.0; the release guard + staged asserts still
  match `v<ver>`.
  (5) editors/README + vscode/README document the distribution policy;
  Visual Studio is deferred in ROADMAP M13.11; other editors stay config-only.
  (6) Marketplace metadata: `homepage` (xiom-lang.org), `bugs` (GitHub issues),
  8 keywords, dark gallery banner, and a valid 256x256 icon; release.yml now
  gates each publish step on that marketplace NOT already having the extension
  version -- bump `editors/vscode/package.json` for every extension change,
  toolchain-only releases skip publishing cleanly (vsce refuses to republish
  an existing version).
- **RELEASE v0.61.3 PUBLISHED (2026-09-23)**: compiler version now MATCHES
  stdlib-v0.61.3 (the owner asked for version parity). Run 35889350771 fully
  green: guard, windows-x64, linux-x64, macos-arm64, macos-x64, VSIX, GitHub
  Release. Assets: `SHA256SUMS`, `xiom-0.61.3-{linux-x64,macos-arm64,macos-x64,
  windows-x64}` and `xiom-vscode-0.12.0.vsix`. The extension publish steps were
  SKIPPED by the version-absent gate (0.12.0 already live on both
  marketplaces) -- the gate works as designed. The release job's docs dispatch
  SUCCEEDED this time (the owner's PAT scope fix), so the website received
  `compiler-release` automatically. Note: `gh` was active as
  `Lefteris-Ngonart` (no write access to the org) after the token change; the
  compiler lane switched back to `Lefteris-Notas` and pushed with its token.
- **CI health batch (2026-09-22, from PR #3 triage)**: the stdlib pin PR #3
  surfaced PRE-EXISTING CI breakage (CI never runs on main pushes, so the
  suite hadn't been compiled in CI since R51-era changes). Fixed on main:
  1. `crates/xiom/tests/scripting_tests.rs`: `script_cache_get` takes the opt
     level since R51 -- `--all-targets` had never compiled it.
  2. Encoding: `python tools/ascii_guard.py repair --apply` (13 tracked files:
     BOMs + typographic chars) and pinned `actions/checkout` in `dco.yml`
     (repo policy requires full SHAs; the DCO workflow was unpinned, so every
     DCO run failed with "action is not allowed").
  3. `paths.rs`: the r31 skip-guard test is now CI-aware (`XIOM_REQUIRE_STDLIB=1`
     panics by design) and the crb3c home-candidate assertion is
     host-separator-agnostic.
  4. **JIT on Linux (real bug)**: the shared-lib JIT links the runtime C inline;
     without `-fPIC` ld rejects the runtime TLS relocation
     (`R_X86_64_TPOFF32 ... recompile with -fPIC` ->
     `failed to set dynamic section sizes`). The driver now passes `-fPIC` for
     `-shared` on non-Windows. Linux jit tests 4/4.
  5. e2e stdlib guard accepts BOTH layouts (`xiom/io.xi` flat or
     `xiom/io/io.xi` module dirs); the flat-only probe made every guarded test
     panic under `XIOM_REQUIRE_STDLIB=1` in CI even though the checkout existed.
  6. Test fixture paths normalized `\` -> `/` across five test files (~2400
     literals) for Linux CI portability; intentional escape sequences
     untouched.
  PR #3 outcome: ubuntu-latest fully GREEN on cd42f5d7; windows-latest failed
  only `e2e_p1_contract_methods` (RUN-time access violation `-1073741819`),
  deterministic on the runner (2/2) but NOT reproducible locally with either
  stdlib pin, debug or release compiler, NASM present (RC=0 against the
  v0.61.3 tree). PR closed as superseded and the pin landed directly:
  **`9f78c853` STDLIB_VERSION -> `stdlib-v0.61.3`**.
  Follow-ups: (a) Windows-runner-only AV on `e2e_p1_contract_methods` (CI-side
  toolchain triage); (b) the `xiom-benchmark-chaos\reference\systems\*` e2e
  paths are monorepo remains -- the benchmark moved to its own repo under the
  xiom-project org and now uses the DOWNLOADED compiler; consider dropping or
  relocating those tests; (c) main CI still does not run on pushes (PR +
  dispatch only), so main health is only checked when a PR opens.
- **Registry staging canary VERIFIED + stdlib 0.61.3 (2026-09-22)**: the
  registry lane independently verified `xiom-std@0.61.3` on staging
  (provenance: xiom-lang/stdlib / publish-registry.yml / refs/heads/main /
  runUrl; sha256 + signature re-checked against the served tarball). Stdlib
  release 35726136818: validate + both gates + package + canary-dispatch all
  green; the **pin-PR job failed** on `GraphQL: Resource not accessible by
  personal access token (createPullRequest)` -- the branch was pushed, so the
  compiler lane opened **PR #3** (pins `STDLIB_VERSION` to `stdlib-v0.61.3`).
  Keygen hint corrected (`898ab391`): publisher keys are ephemeral per run;
  consumers pin the REGISTRY key via `xiom pkg trust`. **PAT scope gap**
  (owner): `XIOM_RELEASE_TOKEN` needs Contents: read/write on
  `xiom-lang/website` (docs dispatch 403) and Pull requests: read/write on
  `xiom-lang/xiom` (pin-PR 403). Production debut: approve run 35726136811
  after ops deploys `trusted-publishers.json` (refs/tags/stdlib-v*) and
  rebuilds production; packages canary still needs the curated batch +
  packages staging entries in ops' file.
- **R67 FIXED (2026-09-23, benchmark/option-porter relay)**: `Ok(x)`/`Err(x)`
  (and `Some`/`None`) inside a function returning a USER struct whose name
  contains "Result"/"Option" (`TestResult`, `Options`) built the WRONG struct
  -- the ctor sites chose the container with `ctor_ret.contains("Result")`, so
  `%struct.TestResult` became the Result type (clang: "invalid getelementptr
  indices"; porter workaround was helper ctors). Fixed via the strict
  container-leaf test `is_llvm_container_struct` (bare `Result`/`Option`,
  module-qualified, or concrete `Result__A__B`/`Option__T`) at all four ctor
  sites; concrete `Option[Point]`/`Result[Point, Str]` returns still build
  their concrete containers. Lock `e2e_m120_ctor_user_struct_return` + CI line.
  Full e2e **2368/2368**.
- **R68 FIXED (2026-09-23, packages relay -- legacy nested `extern`)**:
  `extern "C" { ... }` inside a function body (the audio_beep/legacy-package
  idiom) fell into the expression parser and produced the misleading
  `P001: 'extern' is a reserved keyword and cannot be used as an identifier`.
  The parser now parses nested extern blocks in `parse_block` and hoists them
  to the module level immediately before the enclosing declaration (all three
  top-level loops), so the FFI symbols are declared before their call sites;
  duplicate per-function blocks merge. Parser unit tests + probes (main,
  second fn, if-body, block-form module, duplicates) pass; lock
  `e2e_m121_nested_extern` + CI line. The remaining rule-drift inventory
  (declarations without terminators, extern/unsafe contract requirements) is
  a dialect-migration policy item, not a parser bug; the `xiom.ffi` triage
  abort lives in the packages harness (message absent from this repo).
  Full e2e **2369/2369** (R67+R68 batch).
- **R69 FIXED (2026-09-23, the queued generic `T.to_str()` denormal)**:
  root cause was NOT the design question -- `local_xiom_types` is global and
  never cleared, and the monomorphisation param loop recorded only
  Vec/array/fn params. A generic `x: T` kept a STALE `x: Float64` from
  xiom.fmt, so `fn show[T](x: T) -> Str { return x.to_str(); }` lowered
  through the Float64 conversion: show(99) printed the i64 bits as a double
  (4.891e-322), show("hi") printed 0. The mono path now mirrors `compile_fn`
  and records the substituted param type (T=Int -> "Int"; unresolved T stays
  "T" and falls through to the correct erased-LLVM inference). Verified with
  `tmp/cleanbench/to_str_edges.xi` (now 42/1.5/2.5/true/hello/7/65/99/123)
  and the T = Int/Str/Bool/Float64/UInt + two-param lock
  `e2e_m122_generic_param_type` + CI line. Full e2e **2370/2370**.
- **R70 FIXED (2026-09-23, packages relay -- `for x in <collection>`)**: the
  For lowering treated EVERY iterable as `Range{start,end}`: a `%struct.Vec`
  used its DATA POINTER as the index and stored `data+1` back into field 0
  (silent data-pointer corruption), so Vec loops ran ZERO times (heap address
  > len) and any Vec touched by a loop produced BUG-17-family garbage (the
  geometer's "str_len garbage on Vec[Str] elements"); `&Vec` iterables emitted
  invalid IR (clang "invalid getelementptr indices"). The checker bound the
  loop var to Int ("simplified"), so `for s in vec_of_str` mistyped.
  Collections now lower to real element loops (Vec value/`&Vec` header/array
  literal/fixed array; typed element loads; break/continue/nesting intact);
  `range_inclusive`/`0..=b` lower with end+1; real `Range` values keep
  {start,end}; anything else fails LOUDLY. Checker: element-typed loop vars +
  `Vec[elem]` registered for array-literal bindings. Lock
  `e2e_m123_for_in_collections` + CI line. checker 195/195, parser 101/101,
  feature-reg 510/510, integration 129/129, robustness 63/63, fuzz 24/24,
  full e2e **2371/2371**.
  Remaining relayed (package repros needed; local probes pass): indexed
  `Vec[fn]` calls, untyped `Vec[Int]` reads lowered as Str compares, and
  `byte_at(...) == UInt8` for bytes >= 128.
- **R71 FIXED (2026-09-24, queued `all`/`none` stubs)**: the method form
  returned TRUE for every collection in programs without the core module
  (legacy `xiom_all`/`xiom_none` stubs called with len=0) and silently FALSE
  in programs WITH it (`v.all(pred)` resolved to the generic `core.all`,
  never monomorphised -> returning-zero auto-stub). Both now lower INLINE
  over the Vec header with the closure ABI (env[0] = code pointer, i64
  element args, fail-fast, empty = vacuously true); function-NAME predicates
  fail loudly with a pointer to `xiom.core.all/none`. Method-form calls on
  collection receivers now prefer the inline semantics even when a helper is
  registered; the user-function guard still protects the direct form and
  non-collection receivers (Set/Map/iterator methods unchanged -- m39/m48).
  Lock `e2e_m124_contract_all_none` + CI line; checker 195/195; feature-reg
  510/510; robustness 63/63; fuzz 24/24; stdlib-exec 85/85 (+2 ignored);
  full e2e **2372/2372**.
- **m125 FIXED (2026-09-24, stdlib API freeze -- the last red gate)**: the 9
  missing frozen signatures were RENAME-ONLY drift vs the pinned
  stdlib-v0.61.3 tree: `async.Executor.*` -> `AsyncExecutor.*` (same 7
  methods/signatures) and `net.http_get/http_post` returning
  `Result[NetHttpResponse, NetError]` (R44 rename). No API removals, so the
  FROZEN snapshot was regenerated with the new names (header documents the
  decision) and both freeze tests are GREEN (2/2). The gate was absent from
  the   CI path; a `Stdlib API freeze` CI step now runs it. Test-only + CI
  change (no compiler code). Checker 195/195 unaffected.
- **m126 FIXED (2026-09-24, deterministic publish bytes)**: `create_tarball`
  shelled out to `tar`/PowerShell, so published bytes varied run to run and
  never matched a release asset. New in-repo deterministic writer
  (`xiom-pkg/src/tarball.rs`): sorted entries, ustar headers with
  `SOURCE_DATE_EPOCH` mtime (default 0), uid/gid 0, fixed modes 0644/0755,
  gzip MTIME 0/OS 255 with STORED deflate blocks -- no new dependencies
  (supply-chain gate stays audited-only), byte-identical archives for the
  same tree. Symlinks/special files refused loudly. New
  `xiom pkg publish --tarball <PATH>` promotes exactly the given bytes
  (prints their SHA256; temp-only cleanup). 5 new unit tests + Python
  `tarfile` round-trip validation; pkg 72/72 (was 67). Operational note
  unchanged: re-canary after a release re-run.
- **Packages relay triage + Phase 0 (2026-09-24)**: the packages lane sent
  three minimal repros (probes in
  `C:\Users\lefte\AppData\Local\Temp\kilo\fp-probe\`), REPRODUCED on the
  current build (R66-R72 + m127) and confirmed as the fn-value / generic-mono
  ABI family:
  * `fp4_structfield.xi` (`type Op = { f: fn(&Int) -> Int; }`; `(op.f)(&3)`)
    -> compile fails: clang `%tmp16 defined with type ptr but expected i64`
    in `inttoptr i64 %tmp16 to i64 (i64)*` (fn-typed struct-field load not
    coerced for the call path). Concrete, small, highest-priority of the three.
  * `fp5_two_params_scalar_str.xi` (generic `conv[T, U]` with `U = Str`)
    -> compiles, returns the wrong Str (rc=1 vs the green U=Int control).
    The mono'd closure/return ABI uses the erased/generic type for U.
  * `fp4_maptou.xi` (generic `Vec[U] map`, `U = Str`) -> runtime AV
    `-1073741819`; `fp6_diag_vec.xi` (U = Int) -> rc=100 (`w[0]==0`: generic
    `Vec[U]` push/element stride writes the wrong value). Concrete
    `fp4_vecmap.xi`/`fp5_two_params_scalar.xi` controls pass.
  These fold into **Sprint C (fn-value convention unification)** and extend
  it with the generic-mono return/element ABI; probes accepted -- packages
  will commit them under `docs/repro/` (copies kept in `tmp/fp_probe/`).
  Hygiene: `module x;` (trailing semicolon on a brace-less module header)
  -> P001 at the `;`; the diagnostic is clear but the spelling is natural --
  add a small parser tolerance (queue item FE-17).
  Phase 0 executed: local `stdlib/` was 6 commits BEHIND the pin
  (`385e1e4` vs `stdlib-v0.61.3` = `c7b4027`); refreshed to the tag and
  re-gated -- api-freeze 2/2, stdlib-exec 85/85 (+2 ignored) ON THE PIN. The
  full e2e on the pin runs with the next batch (Sprint A), so the recorded
  2373 is pre-pin-honesty only for the suites re-run here.
- **Release strategy REVISED (owner, 2026-09-24 -- Git free-plan build
  minutes)**: **ONE combined release**, no intermediate tags. Ordering:
  1. Compiler lane completes Sprints A (front-end P0), B (P1), C (fn-value +
     the three packages repros), plus the small FE-17; commits/pushes are
     free, so all work lands on `main` first.
  2. Stdlib lane completes its A-list (coverage waves, dedup TUs, tzdata
     phase 2, untested-surface classes, stub re-triage) and cuts ITS next
     release (stdlib repo budget).
  3. Compiler bumps `STDLIB_VERSION` to the new stdlib tag, runs the full
     gate list on the pin, and cuts the single compiler release (recommend
     **v0.62.0**: user-visible behavior changes, doctor v2, publish alias).
  4. Registry re-canary after that release (assets deterministic; same bytes
     on re-run, but new tag = new asset).
  TLS/schannel FFI hardening (stdlib B-list, gates any HTTPS/TLS claim) must
  either ship in this same window or be explicitly excluded in the release
  notes.
- **Website "What's new" compliance (2026-09-24)**: new tool
  `crates/xiom-release-notes` (std-only, no deps) implements the website
  contract (schema v1, `xiom-lang/website` `docs/release-notes-schema.md`):
  `convert` parses `release-notes/<tag>.md`, merges the stdlib fragment at
  `<stdlib>/release-notes/<tag>.md` (highlights default to `kind: stdlib`),
  validates HARD (ASCII/plain-text/no internal ids or hashes, summary 1-240
  without a version, 1-6 highlights, title<=60, text<=320, kind enum, present
  `breaking` where `- None.` becomes `[]`, docs https), and writes
  `release-notes/<tag>.json` deterministically; `verify` regenerates from the
  markdown and fails unless the committed JSON is byte-identical. Wired:
  `release-notes/TEMPLATE.md` + `README.md`, a CI test step
  (`cargo test -p xiom-release-notes`, 6 unit tests), and the release workflow
  gate BEFORE `gh release create` (stdlib checkout at the pin + toolchain +
  `verify`), plus `client_payload[notes_path]` in the `compiler-release`
  dispatch so the website lane can publish the same file to
  `dl.xiom-lang.org/releases/<tag>/release.json` and set `"notes": true` in
  `releases/index.json` (requirement 5 lives in the website/ops lane).
  Smoke-tested end-to-end via the CLI (convert + verify + JSON parse with
  Python; fragment kinds merged correctly).
- **stdlib relay #2 triage (2026-09-24)**: recorded, fixes scheduled:
  * E001 conservatism: deterministic repro at their
    `tools/probes/evidence/p_e001_borrow_conservatism.xi` (pattern: a
    `&local` call, then a later `&mut local` call; 7 warnings in
    `smoke_collect_sparse`, compile/run green). New checker item: E001 must
    not fire for non-overlapping/consumed temporary borrows, WITHOUT weakening
    the genuine-overlap warning. Scheduled with Sprint C.
  * Generic fn-pointer cross-type matrix: 4 repros filed in their
    `tools/known_failures/` (fnptr Int->Str by-ref; map Int->Str and
    Int->Float64 by value; Option[Int].map[U]; sort_by_key[Int,Str] silent
    mis-sort). Matrix: concrete and same-type generic callbacks are CORRECT;
    CROSS-TYPE callback returns are wrong (Str->Int is correct). Stdlib
    surfaces to avoid until fixed: `sort_by_key[Int,Str]` and
    array/Range/MapIter/Option/Result `.map` with cross-type U; they locked
    the working surface with `smoke_sort_by_key`. Same root family as the
    packages' fp probes: the call/return ABI must follow the MONOMORPHISED
    instantiation, not the erased generic signature.
  * m125 closed independently on compiler main (freeze 2/2 in 46s).
  * Shape re-probes confirmed R67/R68/R69/R70/R72 and the still-open
    `Vec[fn]` literal / `.new()+push` AVs (matches our m127 residuals);
    their green lock `p_r70_pending_shapes.xi` is parked for promotion in the
    pin-bump commit.
- **Front-end audit + backlog (2026-09-24, NOT implemented)**: after a Win11
  first-run report (`xiom doctor` said LLVM missing although LLVM was
  installed; it pointed at the dead `xiom install llvm`), a read-only audit of
  everything a new user sees is in **docs/FRONTEND_AUDIT.md** with a
  prioritized backlog (FE-1..FE-16) and two decision briefs. Headlines:
  doctor's LLVM/NASM probes are PATH-only while the driver falls back to the
  standard locations (FE-1/FE-3, root cause of the report); doctor's
  remediation string is a dead command (FE-2); `xiom publish` still runs the
  legacy git-tag flow and `xiom update`'s text points at the wrong thing
  (FE-8); the registry hosts the stdlib as `xiom-std` while manifests depend
  on `xiom.std` and the pkg help examples say `xiom.stdlib` (FE-9, registry
  lane); installers ship 6 of 9 tools and a non-archive runtime layout
  (FE-10); `install_deps.ps1`'s LLVM fallback is an unsigned 19.1.0 download
  (FE-11). D-1 recommendation: keep the stdlib bundled (platform dep) and
  align names; D-2: yes, build the agreed `xiom toolchain check|update`
  (POST_RELEASE_PLAN 1), starting with `check --json`.
- **Cross-lane relays (2026-09-24, for awareness -- no action taken)**:
  * stdlib lane: R66-R70 fixes are on compiler `main` but NOT in the v0.61.3
    pin (e2e 2371 there); next pin bump carries them and unlocks the
    `for x in <Vec>` probe shapes. Their
    `stdlib_api_freeze_no_removals` snapshot item is now RESOLVED here
    (m125, rename-only regeneration + CI step) -- the duplication-gate twin
    removal was waiting on it. Their waves remaining (recon ready): math
    number_theory/factorial/combinatorics, text, regex, test, collections
    (~35 safe clauses), error/io tails; dedup translation units
    (hash vs linkedhash, cache vs lru) need the freeze-snapshot regen (done);
    tzdata phase 2 + untested-surface tail (44 struct-param fns without a
    usable ctor, non-scalar fn params, 83 generic fns) needs new
    gen_call_probes.ps1 generator classes.
  * registry lane: recorded in their SESSION.md at 5c1cdcb. Policy now reads
    that deterministic assets make re-runs byte-stable, but re-canary after
    (a) any asset regeneration or (b) any change to release packing (the
    canary validates the auth/mapping/provenance path and the publish-time
    signature, not the bytes). `xiom pkg publish` still re-packs, so
    byte-identical staging->production promotion needs the client to pack
    deterministically or to publish an existing tarball -- m126 delivered
    BOTH (deterministic writer + `publish --tarball`), so the registry lane
    now has the client-side half; they noted it as optional/unscheduled.
    Nothing pending from ops or registry config.
  * benchmark lane: `COMPILER_VERSION` still pins the absent v0.61.0;
    recommend bumping to v0.61.3 and refreshing the STATUS fields after
    re-running the four suites.
- **m128 FIXED (2026-09-24, registry relay -- package/toolchain correlation)**:
  `xiom pkg publish` never sent the registry's per-version `compiler` field
  (the server reads `req.body.compiler`, capped at 64 chars), so the website
  could not correlate packages with toolchain releases. Publish now resolves
  the tag from `--compiler <tag>` (wins) or the manifest's optional
  `compiler: "v0.61.3"` field, validates it as `vX.Y.Z` (suffix allowed,
  1-64 ASCII), prints it, and adds it to the multipart fields; publishing
  without one warns loudly (not fatal). The field starts flowing with the
  next release; package repos should add `compiler:` to their manifests.
  pkg tests 74/74 (2 new).
- **m127 FIXED (2026-09-24, packages relay -- indexed `Vec[fn]` calls)**: an
  array literal of fn REFERENCES stored the RAW code address as the element
  while every call path uses the closure ENV convention, so `fns[i]()` loaded
  field 0 from machine code -> deterministic AV (the relayed
  `xiom.test.run_all`; workaround `run_test_at(index)`). `compile_array_as_vec`
  now wraps fn-reference elements into envs via `wrap_fn_ref_env` (B-007
  thunk), with the return type derived from the `fn(...) -> R` spelling. Lock
  `e2e_m127_fn_vec_indexed_calls` + CI line; checker 195/195; feature-reg
  510/510; integration 130/130; robustness 63/63; fuzz 24/24; full e2e
  **2373/2373**.
  OPEN (documented in COMPILER_BUGS, repros in tmp/probe_p1/):
  `Vec[fn].new()+push` elements, element-to-local calls, and fn-element calls
  through struct fields still break -- they need a unified fn-value
  convention across generic instantiation, element tracking and the raw-code
  call path (dedicated refactor).
- **Item-2 VERIFIED GREEN (2026-09-23)**: `stdlib_tests::
  stdlib_all_modules_compile_to_ir` passes on the current pin (with and
  without `XIOM_REQUIRE_STDLIB=1`); the R62-era ascii85/`float_to_string`
  findings were resolved by the stdlib pin refresh. Remaining red:
  `stdlib_api_freeze_no_removals` (9 stale snapshot signatures
  `async.Executor.*`, `net.http_get/http_post`; red at baseline) -- cross-lane
  snapshot regeneration, not a compiler bug.
- **Cross-lane relay (benchmark repo, 2026-09-23)**: releases v0.61.1/v0.61.3
  exist but the benchmark's `COMPILER_VERSION` still pins v0.61.0 (absent).
  Recommend bumping to v0.61.3, then re-run the four benchmark suites and
  refresh the STATUS compiler fields (they currently record the pin while the
  suites ran on the installed v0.61.3). Publishing stays blocked on repo
  protection + registry scope additions (owner). Not actionable in this repo.
- **R66 FIXED (2026-09-23, P1-4 contract methods -- the Windows-CI AV)**:
  `e2e_p1_contract_methods` AV'd (`-1073741819`) only on `windows-latest`;
  locally it returned a WRONG answer instead (sorted `[1..5]` reported false).
  The contract lowering passed a pointer to the receiver VALUE to
  `xiom_is_sorted(i8*)`/`xiom_contains(i8*, i64)`; the runtime reads `data[0]`
  as the element COUNT, and for a `%struct.Vec` that word is the DATA POINTER
  -> unbounded scan (runner: unmapped memory; locally: garbage words decided
  the answer). `is_sorted`/`contains` now lower INLINE over the real Vec
  header (len/data/elem-size; sign-correct int loads, `fcmp` for floats,
  `strcmp` for Str; array-literal/fixed-array receivers bridge to a heap Vec;
  unsupported element kinds and non-collection receivers fail LOUDLY).
  Lock `e2e_m119_contract_method_values` + CI line; the p1 fixture now asserts
  values; integration tests updated (inline scan + loud-rejection negative).
  `all`/`none` keep the legacy `len=0` stub (not the reported AV).
  Full e2e **2367/2367**; feature-reg 510/510, stdlib-exec 85/85 (+2 ignored),
  diff 24/24, robustness 63/63, fuzz 24/24. NOTE: `stdlib_api_freeze_no_removals`
  is RED at 9 missing signatures (`async.Executor.*`, `net.http_get/http_post`)
  and fails IDENTICALLY at the pre-R66 baseline -- snapshot vs pinned-tree
  drift, cross-lane (stdlib lane), NOT this change.
- **R65 FIXED (2026-09-22, stdlib p_platform_env)**: `xiom.env.OS/ARCH/FAMILY`
  were hardcoded literals in the stdlib ("windows"/"x86_64"), so a Linux build
  reported windows while runtime detection said linux. Codegen now computes
  them from the TARGET at the module-qualified reference site
  (`target_platform_constants`: wasm -> unknown/wasm32/wasm; aarch64/riscv64
  -> linux/<arch>/unix; Native -> the compiler's own OS/ARCH/FAMILY), and
  seeds the leaf key for bare references. Verified on Windows and Linux (WSL)
  with the stdlib's exact probe: `env.OS=[linux] env.FAMILY=[unix]
  env.ARCH=[x86_64]`. Lock `e2e_m118_env_platform_constants`; full e2e
  2366/2366. Cleanup note: remove `tmp/` archive extracts after verification
  (duplicate stdlib trees trip `e2e_m17_zero_warnings`).
- **R64 (2026-09-22, AI context + post-release spec)**: `AI_CONTEXT.md` at the
  repo root is now the single source of truth for toolchain facts: compiled
  into `xiom-mcp` via `include_str!` and served as
  `xiom_workflow_guide {topic:"context"}` (verified over stdio), shipped in
  every archive as `lib/AI_CONTEXT.md` (release staging + `package.ps1`/sh),
  and renderable by the website docs. Refreshed `crates/xiom-mcp/src/guides.rs`
  where stale: registry URL is now `registry.xiom-lang.org` (was `.com`),
  `xiom pkg` command set (search/info/install/publish/keygen/sign/lock/list),
  OIDC publishing note, `xiom run --opt-level/-O` + script-cache temp
  fallback, `--timeout`, the bundled `z3`/`xiom-wasm.wasm` in the binary
  table, and the long-stale "716 tests" note. Added
  `docs/POST_RELEASE_PLAN.md` spec'ing the verified toolchain updater and the
  MCP `get_contracts`/`search_symbols` tools (owner-approved, post-0.61.1).
- **CRB-8 / RELEASE v0.61.1 PUBLISHED (2026-09-22)**: first public release.
  Two live issues found and fixed during the tag run:
  (a) the VSIX publish to the VS Code Marketplace was rejected by the
  name-similarity policy (`Similar extension display names: ... Language
  Support`); `displayName` is now **"XIOM Toolchain"** (package id stays
  `xiom-lang.xiom`, version 0.12.0).
  (b) The `v0.61.0` tag could not be moved to the fix commit -- the repo's
  release-tags ruleset blocks tag deletion/updates (verified: remote rejected)
  -- so the release was cut as **v0.61.1** (workspace version + README bumped;
  all nine tools report v0.61.1). The `v0.61.0` tag remains as a dead,
  release-less tag; nothing was ever published under it.
  (c) The GitHub Release job failed only at the LAST step, the
  `compiler-release` dispatch to xiom-lang/website: `Resource not accessible
  by personal access token (403)` -- `XIOM_RELEASE_TOKEN` lacks Contents:
  read/write on the website repo. The step is now `if ! gh api ...` +
  warn/exit 0 so a docs-dispatch failure can never fail a release; the PAT
  still needs extending for automatic docs publishing.
  Verified live: GitHub Release `v0.61.1` carries `SHA256SUMS`,
  `xiom-0.61.1-{linux-x64,macos-arm64,macos-x64,windows-x64}` and
  `xiom-vscode-0.12.0.vsix`; the linux archive contains `bin/xiom-pkg`,
  `bin/xiom-dbg`, `bin/z3` and the `xiom-std` manifest; VS Marketplace
  `xiom-lang.xiom` 0.12.0 and its item page return 200; Open VSX 0.12.0
  returns 200. Staging canary prerequisites are satisfied.
- **CRB-7 (macOS release enablement, 2026-09-22)**: enabling
  `RELEASE_BUILD_MACOS` exposed two real portability bugs, both fixed:
  (a) `.cargo/config.toml` carried target-wide `-Wl,-stack_size` rustflags for
  macOS, which Apple's `ld` rejects for non-executables -- every proc-macro
  dylib failed (`ld: -stack_size option can only be used when linking a main
  executable`); the macOS entries are removed (macOS main-thread stack is
  8 MB; use `cargo:rustc-link-arg-bins` if a bin ever needs more).
  (b) `std::arch::is_x86_feature_detected!("avx512f")` sat under a
  `cfg!(target_arch = "x86_64")` RUNTIME check, so it still COMPILED (and
  failed) on arm64; the block is now `#[cfg(target_arch = "x86_64")]`.
  (c) `macos-x64` moved from the retiring `macos-13` (queued >1 h) to
  `macos-15-intel`; the release job now `needs: [build, vscode, build-macos]`
  with `!failure() && !cancelled()` so the optional macOS dependency neither
  skips nor stalls a release. Verified: dispatch 35679231885 is green for
  guard + linux-x64 + windows-x64 + macos-arm64 + macos-x64 + VSIX; the
  downloaded `xiom-0.61.0-macos-arm64.tar.gz` contains all nine tools
  (incl. `xiom-dbg`), `z3` + `libz3.dylib`, `LICENSE-Z3`, and the correct
  `xiom-std` manifest.
- **R63 FIXED (2026-09-22, playground C3/C6 pack)**: script-run `--opt-level`
  dispatch behind leading global flags + `-O<n>` spelling; JIT/script cache
  falls back to `$TMPDIR/xiom_jit` when HOME is set but unwritable;
  `STDLIB_VERSION` bumped from `stdlib-v0.60.0` to stdlib main
  `385e1e44fac37a9403cd04cdb5e13d4c122a8710` (correct `xiom-std` manifest,
  env shim, waves 15-20), local checkout refreshed. Gates on the new pin:
  e2e 2365/2365, stdlib-exec 85/85, feature-reg 510/510, diff 24/24.
- **R62 FIXED (2026-09-22, playground request #2)**: fmt reachable-only peek
  -- primitives peek nothing (codegen builtins), floats peek the small
  `xiom.convert`, generics keep the R47 fmt fallback; `peeked_leaves` keeps
  `float_to_string` through the injection reachability filter and the codegen
  float builtin resolves its symbol via `fn_symbol_map`. Bench:
  to_str/hello emit-ir ratio 1.81x -> 1.12x, loop_200 -31%; output `42`/`1.5`.
- **R61 FIXED (2026-09-21)**: R7 residual + interface-ABI ruling. (a) local
  explicit-generic calls (`var h = make_holder[JsonValue]()`) never recorded
  the substituted return type, so `h.values[0]` lost the element type and the
  index read loaded the first 8 bytes of an inline aggregate as a pointer
  (0xC0000005 in `p_generic_push`/`p_gp_b`/`p_gp_c`); `infer_call_return_xiom`
  now substitutes `Expr::GenericCall` explicit type args. (b) `impl
  Trait[Args]` self PARAMs are retyped from `Self` to the impl type (T001).
  (c) zero-generic entries in `generic_fn_decls` no longer route into the
  generic path (silent 0/no call). (d) RULING: the interface value-receiver
  ABI is unimplemented; codegen now rejects aggregate args for interface-typed
  params LOUDLY (`p_hash_probe` no longer silently returns 5381). Locks
  `e2e_m116_generic_ctor_field_index`,
  `e2e_m117_interface_value_abi_rejected`; full e2e 2365/2365.
- **R59/R60 FIXED (2026-09-21, stdlib relay findings)**: (R59)
  `p_result_tuple_vec_loop` -- the enclosing match arm's result slot leaked
  into nested loop bodies (`oid.push(v)` as a while body's last statement
  stored `%struct.Vec` into the arm's `%struct.Result` slot); While/For/Spawn
  bodies now save+clear `match_result_ptr`. (R60) `p_ref_tuple_mangle` --
  reference-typed tuple elements kept their `&` ("&Vec" -> mangled
  `Tuple__&Vec__Vec`) and container-ctor elements were named by their erased
  LLVM type (`Vec[UInt8].new()` -> "Int"); the expr-side namer strips
  reference markers and BOTH namers resolve container-ctor elements to the
  container base (non-ctor calls keep their qualified inferred name).
  Locks `e2e_m114_result_tuple_vec_loop` / `e2e_m115_ref_tuple_mangle`; the
  three locks an over-broad shared-namer version broke (m37/m44/m48) are
  green. Full e2e 2363/2363.
- **R58 / L8-14 FIXED (2026-09-21)**: Map[Str,Str] morse trap. Payload-type
  resolution for `unwrap`/`unwrap_or` only handled Ident receivers, and a
  `&Map[Str, Str]` param's tracked type was truncated to `"&Map"` (args
  dropped), so `morse.get(ch).unwrap_or("?")` returned the Str payload as a
  raw i64 handle and `result + code` printed pointers. Fixed
  `ref_preserving_name` (bracket-preserving via `type_annotation_name`),
  `generic_container_last_arg` (&/&mut stripping), and both unwrap builtins
  (call receivers resolve via `scrutinee_payload_xiom`; aggregates unbox via
  `try_unbox_payload`). L8-14 prints all four lines; lock
  `e2e_m113_map_str_str_morse`; full e2e 2361/2361.
- **R57 / L3-50 FIXED (2026-09-21)**: `?` on a Result with a tuple payload.
  `let (a, b) = two()?` bound both names to the raw boxed-tuple handle
  (`a + b` printed pointer arithmetic). The `?` handler now unboxes AGGREGATE
  payloads out of the erased i64 slot via `try_unbox_payload`: tuples
  (`(Int, Int)` -> `Tuple__Int__Int`), nested containers, and registered
  structs/enums; primitives/Str/floats stay raw i64; Err propagation is
  unchanged. L3-50 prints 8 deterministically. Lock
  `e2e_m112_try_result_tuple`; full e2e 2360/2360.
- **R56 / L5-40 FIXED (2026-09-21)**: `Map[Int, Vec[Str]]` container payload.
  Decision: container elements are INLINE (32-byte `%struct.Vec` header in the
  slot, matching `Vec[Vec[T]]`); concrete Option/Result layouts keep inline
  payloads. Coordinated five-site change: (1) both explicit type-arg fallbacks
  render nested args via `type_arg_to_name` + mono substitution
  (`Map.new` was `_Int_Int`); (2) `infer_generic_ident_type` prefers the
  tracked local type with args over the erased LLVM slot (`Map.insert` was
  `_Int_Vec`); (3) `record_field_vec_elem` accepts container element types
  (index read now memcpys the inline header); (4) the mono signature builder
  lowers substituted container names (`V` -> `Vec[Str]`) to `%struct.Vec`
  instead of i64; (5) match payload binding resolves fields through the
  CONCRETE `%struct.Option__*`/`Result__*` registration when present. L5-40
  prints 2; lock `e2e_m111_map_vec_container_payload`; the two locks earlier
  attempts broke (`e2e_fnptr_vec_index_call`, `e2e_m71_concat_index_elem`)
  are green -- fixed-array brackets are excluded from container detection.
  Full e2e 2359/2359.
- **R55 / L6-40 FIXED (2026-09-21)**: module-scoped generic factory whose T
  is fixed only by a LATER call (`var runner = plugin_runner.create();
  plugin_runner.add_plugin(&mut runner, EchoPlugin{});`) mono'd `run_all` with
  T=Int -> C001. Added a bounded function-body evidence pre-pass
  (`prepass_generic_type_evidence`, decl.rs) that runs before each body is
  emitted, records the factory call site's concrete types (callee byte-span
  key, consumed before the `0` fallback) and seeds the binding's container
  type; `infer_call_return_xiom` resolves module-qualified generic returns
  from the recorded instantiation or the pre-pass evidence (the `var` arm
  records the binding before compiling its initializer). Lock
  `e2e_m110_generic_factory_evidence` (fixture
  `tests/regression/m110_generic_factory_evidence`); full e2e 2358/2358.
- **R53 registry metadata consumption (2026-09-21, registry relay)**:
  `xiom-pkg` now consumes the server-extracted package metadata
  (`license`, `categories`, `keywords`, `repository`) in the index and
  `/packages/:name`; `search` matches keywords/categories, prints them, and
  takes `--category <c>` (local filter) plus `--json`; new `info
  <pkg>[@version]` renders description/categories/keywords/license/repo,
  the pinned version's digest/signature and the sorted version list (human
  and JSON); publish prints 201-body `warnings`. Manifest parsing tolerates
  inline and multi-line metadata arrays (locked by test). MCP gained
  `search_packages({query?, category?})` and `package_info(name)` (16 tools
  total) invoking `xiom-pkg --json`, returning the exact index field names.
  Tests: pkg 72/72, mcp 39/39. Live smoke: search/info JSON + `xiom pkg`
  dispatch verified against the staging registry.
- **R53 staging verification (2026-09-21, registry relay #2)**: re-verified
  live against `https://staging.registry.xiom-lang.org` via the `XIOM_REGISTRY`
  override. MCP `search_packages({category:"core"})` and
  `search_packages({query:"matrix"})` both return `xiom.math` with
  categories/keywords populated; the `matrix` hit is keyword-only (the name is
  `xiom.math` and the description has no `matrix`), proving keyword indexing
  is consumed. MCP `package_info("xiom.math")` returns description, categories,
  keywords, license, repository, latest, and per-version
  sha256/signature/publicKey/dependencies/yanked. CLI parity holds for
  `xiom-pkg` and the `xiom pkg` forwarder (`search --category core --json`,
  `search matrix --json`, `info xiom.math --json`). Wire-shape gaps reported
  to the registry: index top-level `registry`/`version` are declared but
  unused and `updated_at` is ignored (no schema-version negotiation);
  version-level `size`, `published`, `yankedAt`, `yankReason` and the
  duplicated per-version metadata block are dropped; the server `/search`
  endpoint emits `results` while the client `--json` envelope uses `packages`
  (the client filters `/index.json` locally, so a server-side search switch
  would need a mapping); `/categories` has no client surface.
- **R52 payload/binding batch (2026-09-19, playground audit S19)**: closed
  the remaining 11 nondeterministic/pointer-print lessons (L3-02,
  L5-09/20/24/26/29/31/35/36/43) plus L5-21 via a family of type-erasure
  fixes: concrete generic-struct field/method typing, match-result and
  match-expression Str prediction, Map[Str,Str]/unannotated-Some payload
  binding, redundant explicit-self argument handling, and single evaluation
  of side-effecting match scrutinees. Locks `e2e_m106..m108`; e2e
  2356/2356. Still open: L3-50 (Result tuple payload path) and L8-14
  (Map[Str,Str] morse trap).
- **R52 verifications (2026-09-19, packages + website relays)**:
  `xiom-verify --check` DOES run the bundled/auto-detected Z3 over the
  generated SMT-LIB (Z3Runner: Z3_PATH -> `<exe_dir>/[../bin/]z3[.exe]` ->
  PATH; verdicts Proven/Violated/Inconclusive/Error), while `xiom --verify`
  (and `xiom-verify` without `--check`) is EXPORT-ONLY (writes SMT-LIB; only
  prints "Z3 found -- use 'z3 file.smt2'"). Website `compiler.md` line ~81
  ("`--verify` runs Z3 over that") is therefore inaccurate -- the corrected
  wording (exported vs checked vs proved) is relayed for the website lane.
  `cargo build -p xiom --features nasm` assembles the runtime .asm objects
  (`stdlib/runtime/*.obj` refreshed by nasm 3.02) and links/runs the asm
  memop path (m96 fixture exit 0), so both configurations now work. Release
  artifacts rebuilt from source: `target/release/xiom.exe pkg --help` execs
  the sibling xiom-pkg (was the stale pre-R48 binary) and `pkg keygen
  --help` prints the per-command usage; release compile smoke exit 0.
- **R52 packages relay (2026-09-19)**: (a) unqualified `assert` after
  `use xiom.test;` returned a corrupt TestResult (F/0) because the keep-first
  bare alias bound the transitively-imported private `core.assert`; bare-call
  resolution now only honours a keep-first alias whose target is PUB and
  otherwise ranks imported-module exports (private helpers inside their own
  module, e.g. `normalize_duration`, stay reachable). Lock
  `e2e_m105_unqualified_assert`. (b) `xiom.std` (and legacy `xiom-std`) is a
  platform dep: a version spec no longer fails the registry closure. (c)
  `xiom pkg <cmd> --help` prints that command's usage (`keygen --help` no
  longer writes a key). (d) The local `target/release/xiom.exe` is stale vs
  source (pre-R48 dispatch); a fresh release build is needed for publishing
  -- the source is correct.
- **R51 stdlib residual (2026-09-19, p_pre_capture_callee)**: implication-
  wrapped contract clauses (`result is Some => ...@pre...`) never emitted
  entry snapshots -- the `@pre` walkers had no `Imply`/`Is` arms, so the
  ensures compared the live pointer with itself (2 == 2 - 1 in list/queue/
  rbtree/fenwick). Both walkers now descend; `p_pre_capture_callee.xi` and
  `p_wave8_shapes.xi` exit 0; lock `e2e_m104_pre_capture_callee`. The stdlib
  lane can restore the strong `@pre` size clauses in the blocked modules.
- **R51 L4 cluster (2026-09-19, playground audit S19)**: closed the four
  remaining `L4-*` clang failures -- `@pre` receivers in `.len()`
  (`items@pre.len()`, `self@pre.items.len()`) now resolve through the
  Vec/field type helpers, and `for i in range(0,n)` resolves the range
  builtin structurally so a user `fn range(...)` cannot hijack the loop.
  The playground's C17 residue is now L6-40 (needs the fixpoint inference
  pre-pass) and L5-40 (container-ABI design, see the L5-40 entry). Locks
  `e2e_m101..m103`.
- **R51 playground requests (2026-09-19, audit S19)**: `--opt-level N` is now
  honored on the script-run path (`xiom run [--opt-level N] file.xi`) and is
  part of the script-cache key (`xiom-cache-v3|...|opt=N`), so an -O0 run is
  never served an -O2 binary; the flag is also stripped from the positional
  args (it used to be read as the file name). An empty/unset HOME or
  USERPROFILE now falls back to `%TEMP%/xiom_jit` (the cache kept working;
  earlier an empty HOME produced a cwd-relative path). Verified on the lesson
  corpus shape: opt=0 and opt=2 builds populate separate entries, cached
  reruns 0.03-0.08s, HOME-less cached rerun works. Tests: xiom lib 32/32,
  cli_args 1/1; jit cache-key test covers level separation.
- **R50 registry follow-ups (2026-09-19, relayed by the registry session)**:
  `xiom install` now delegates to the verified `xiom pkg install` client and
  `xiom update` is retired with guidance -- the legacy git-clone-from-
  `/packages.json` handlers (`handle_install`, `fetch_registry_index`,
  `parse_deps_from_manifest`, `RegistryPackage`) were deleted so no install
  path can skip checksum/signature/yank. Package names are dotted: the
  resolver accepts canonical `xiom.std` and the legacy `xiom-std` alias
  (the pinned stdlib checkout still declares the hyphen form; coordinate the
  full switch with the stdlib session). Tests: pkg 64/64, xiom lib 32/32,
  mcp 39/39, graph 31/31, cli_args 1/1. Queued: `xiom pkg update|outdated`
  reading /index.json and a SHA256-verified self-update from
  dl./latest.json; OIDC publish needs no client change.
- **OPEN compiler bugs after R49** (exact repros/evidence in
  docs/COMPILER_BUGS.md; reproduce from the playground lesson sources):
  1. L6-40: module-scoped `Runner[T: Plugin]` `create()` -- T is only fixed
     by a LATER call; needs a fixpoint pre-pass (single-pass inference
     emits the `0` fallback). Trace evidence captured.
  2. L5-40: builds now, but `group_by_age` prints 0 -- Map container-payload
     ABI (insert boxes the Vec then memcpys the unboxed header into an
     8-byte handle slot; get derefs the handle).
  3. C18/C19 residue: remaining per-lesson triage (L3-02,
     L5-09/20/24/26/29/31/35/36/43, L3-50 exit 200, L8-14 `0xC000001D`,
     L5-21 Float64 bits inside `Vec[T]`).
  4. R49-1 module-path alias: FIXED (declared-identity parsing + import
     rewrite + freeze-resolver header index). `stdlib_api_freeze_tests` is
     GREEN (214/214 frozen entries; 52 stale/renamed snapshot lines
     regenerated in the same commit). Lock `e2e_m99_module_path_alias`.
  5. R49-3 Result payload contract: FIXED (payload rebound `.value`
     container dispatch; lock `e2e_m100_result_payload_contract`).
     R49-4 (clang ISel crash) still filed.
  6. Perf: `.to_str()` auto-injected `xiom.fmt` closure +2.3-2.5 s ->
     reachable-function-only peek (Stage 6 gate).
  7. C3 (script-mode flags) / C6 (stdlib `package.xi`): need an exact
     playground repro before changing behavior.
- **Final-IR tip**: `--emit-ir` prints the INTERMEDIATE emitter output; to
  get the IR clang actually compiles, force the link to fail
  (`--link missing_xyz`) and read `<output>.ll` (kept on failure, deleted
  on success). A later pass rewrites e.g. `Task.to_str` stubs to
  `Str.to_str`.
- **Playground repro recipe**: `git clone --depth 1
  https://github.com/xiom-lang/playground tmp/playground`; each lesson
  `lessons/**/Lx-yy.json` carries `.solution`; extract to `tmp/lessons/` and
  run `target\debug\xiom.exe -o tmp\lessons\Lx-yy.exe tmp\lessons\Lx-yy.xi`
  (rebuild `cargo build -p xiom` first; stdlib resolves from `stdlib/`).
  Do NOT rebuild while an e2e run is active: the harness spawns
  `target/debug/xiom.exe` and a mid-run replacement produces flaky
  crashes/false failures.

## State at handoff

Stage 3 Item A CLOSED (`strict_catalog_findings = true`), the R-bug queue
through R24 is CLEARED, and the supply chain is signed. Highlights:

- **R21 (flip closure, round 61)**: scope-first user aliases (pre-load
  worklist + single-segment `use X as Y;`), container-receiver wildcard
  guard, generic-`!` defer, `examples/test_mod` fixture collision. Locks
  `m74`; corpus gate live (checker 189/189).
- **R20 (round 62)**: catalog same-leaf alias delegation binds the
  checker-recorded owner-qualified target; deterministic suffix scans.
  Lock `m75_alias_delegation`.
- **R18 (round 63)**: contract payload reads on the bare `is` rebind; Err
  rebinds load Result field 2. Lock `m76`.
- **R16 (round 64)**: extern return types feed pointer inference
  (`buf + len` is pointer arithmetic). Lock `m77`.
- **R15b (round 65)**: same-leaf USER-program modules (module-qualified
  registration, collision-qualified symbols, full-path-first aliases).
  Lock `m78` (multi-source).
- **Catalog collisions (round 66, R21d)**: deterministic winner
  (source-dir index -> structural path match -> smallest canonical path),
  canonical identity, `warning[W001]` for loaded modules, `release/` skipped.
- **LSP (round 67)**: `Backend::parse_cached` (content-hash) + cross-file
  definition; lsp 44/44.
- **R22 (round 68)**: plain module imports bind for codegen receivers via
  `module_receiver_paths` (catalog-isolated); percent dedup unblocked.
- **Fuzz + sanitizer CI (round 69)**: standalone `fuzz/` cargo-fuzz
  workspace (lexer/parser/ctfe/pipeline + seeds); CI `fuzz-smoke` (ASAN,
  30s/target) + `sanitizer-smoke`; CI runs bin-only crate tests
  (pkg/dbg/lsp/mcp), robustness/fuzz suites, m74-m78 locks.
- **dbg async MI reader (round 70)**: reader thread + bounded-wait queues;
  non-stopping continues report "running" instead of hanging the DAP.
- **`.xi` DWARF (round 71)**: `DISubroutineType` (the old `type: !{}`
  dropped ALL DWARF) + per-statement `!DILocation`s under `-g`;
  llvm-symbolizer maps body lines. Lock `m79`.
- **R23 (round 73)**: fn-typed values are env-first in every shape
  (call-through-value, `Vec[fn()].pop()` payloads, struct fields). All async
  probes green; lock `m80`.
- **R24 (round 74)**: catalog-body externs win under isolation (program
  bodyless decls no longer shadow stdlib extern blocks); `extern_fns` is
  part of the per-body context. The Stage 7 selfhost compile gate
  (`diff_tests::test_selfhost_v092_compiles`) is GREEN.
- **Supply chain (round 75)**: ed25519 `signing.rs` (keygen/sign/verify +
  `~/.xiom/trusted_keys.json` trust store), fail-closed install for trusted
  registries, signed publish over ureq-only multipart (curl gone),
  `XIOM_REGISTRY_TOKEN` auth header, git deps must pin a full 40/64-hex
  commit. CLI: `xiom pkg keygen|trust|trusted|sign|verify`.
- **R25 CLOSED (round 77)**: the 30-module bench graph emits byte-identical
  IR (5,686,880 bytes, 5/5 runs; round-76 drifted ~130-230 bytes). Six
  HashMap-order classes fixed: `resolve_bare_fn_ref_key` (scope-first +
  deterministic suffix pick) wired into the fn-as-value ptrtoint,
  `wrap_fn_ref_env` and all five fn-typed-arg param lookups; fn-value
  ptrtoints map through `fn_symbol_map`; @pre snapshot worklist, mono
  worklist (total order), concrete-Option builtins and variant scans made
  deterministic. Lock `e2e_m81_fn_ref_same_leaf`; the determinism canary now
  covers BOTH selfhost v092 and the bench graph.
- **R27 CLOSED (round 78)**: installed-binary stdlib discovery. Pure
  `stdlib_candidates`/`is_stdlib_root` wired into the M12 bootstrap,
  `find_stdlib_dirs()` and `find_runtime_c()`; exe-relative `lib/` outranks a
  stale global `XIOM_HOME` (which is a fallback, not an override); catalog
  search dirs use the FIRST valid root only. Fake-install + release-binary
  `use xiom.io` sims exit 0; 5 new unit tests. Release R0 compiler-side
  blockers are done.
- **R31 CLOSED (round 82)**: cross-repo test isolation for the split. One
  helper (`xiom-graph::paths`): R27 candidates + `stdlib_root()`,
  `stdlib_smoke_dir()` (XIOM_STDLIB_SMOKES -> `<stdlib>/tests/smoke/` ->
  legacy), `stdlib_or_skip()`/`skip_if_missing()` (loud SKIP locally, hard
  FAIL under `XIOM_REQUIRE_STDLIB=1`). Driver delegates; JIT resolves runtime
  via the scan (`xiom build-runtime` verified); codegen/LSP/MCP tests
  parameterized; R9-01 guard prints SKIP; `scripts/fetch-stdlib.ps1|.sh` +
  `STDLIB_VERSION` (`main` for now -- release lane swaps in the split tag) +
  `.gitignore stdlib/`; packaging bundles from the checkout and drops the
  `xiom-playground` WASM copy for a release-artifact lookup.
- **Stage 6 start (round 76)**: `perf_budget_tests.rs` (IR byte budgets +
  180s ceiling + byte-identical determinism canary), CI-wired;
  deterministic variant->parent-enum and `type_meta` selection
  (`pick_deterministic`: current module -> shortest key -> lexicographic).

Last full gates (round 82): workspace `--lib` 521/521, checker 189/189,
feature-reg 510/510, stdlib-exec 85/85 (+2 ign), perf 2/2, pkg 52/52, dbg
34/34, lsp 44/44, mcp 39/39, jit 5/5, `xiom build-runtime` exit 0, e2e
**2331/2331** (R30 binary; the R31 run is the split gate), `cargo build
--release -p xiom` clean. Known red, stdlib-lane owned, pre-existing:
`stdlib_api_freeze_no_removals` (52 drifted signatures) and
`stdlib_tests::stdlib_all_modules_compile_to_ir` (`encoding.ascii85` T001) --
both documented in docs/COMPILER_BUGS.md R31 FIXED.

## R25 -- CLOSED (round 77)

Deterministic fn-REFERENCE resolution landed; the bench-graph drift is dead.
Full evidence and the fix breakdown are in docs/SESSION.md round 77. In
short: scope-first + `pick_deterministic` bare-fn resolver (lib.rs) used by
the fn-as-value ptrtoint / thunk symbol / fn-typed-arg param paths; the
ptrtoint now materializes the pre-assigned `fn_symbol_map` symbol; @pre
snapshot order, mono worklist total order, concrete-Option builtin order and
the variant-parent scans are sorted/deterministic.

Verify with:

```
target\debug\xiom.exe --emit-ir examples\benchmark\main.xi > a.txt   (x3, compare bytes)
cargo test -p xiom-codegen --test perf_budget_tests
```

Residual findings: the bare-variant disagreement was fixed by R42
(scope-first parent pick); the `%struct.Metrics` GEP by R39; the generic-arg
inference leak by R46. **The bench graph is now CLANG-CLEAN** (R46):
`xiom --emit-ir examples/benchmark/main.xi | clang -c -x ir -` exits 0, and
the determinism canary stays green at 5,808,645 bytes. Stage 6 measures
emitted IR bytes only, so this never gated; it is now also a full-build
candidate.

Formal R25 entry in docs/COMPILER_BUGS.md is deferred (that file had
uncommitted stdlib-lane WIP at close; append once clean).

## Open compiler findings (pre-selfhost, not R0-blocking)

1. **Generic method instantiation misses for qualified receivers** (R46
   residual) -- **FIXED 2026-09-18 (R46b); m88 wrappers REMOVED**. (a) A
   `module.Type` receiver resolved its same-leaf type by `type_meta` HashMap
   order, so `g.Box.new` bound the sibling module on ~25% of runs and fell
   to an erased zeroinitializer stub. The receiver path is now expanded
   through the checker's module bindings (`qualified_type_key_for_path`,
   deterministic). (b) A generic method whose type parameter comes only from
   the receiver (`value_of()`/`is_sealed()` on `Box[Int]`, and computed
   receivers like `g.Box.new[Str]("x").value_of()`) now infers the type arg
   from the receiver call's instantiation (`receiver_generic_arg_at`)
   instead of the literal-0 stub. Lock `e2e_m88_generic_same_leaf_boxes`
   covers the direct cross-module call forms. See docs/COMPILER_BUGS.md
   R46b.

1b. **R44 same-leaf class -- resolved for HttpResponse, class remains**:
   the stdlib renamed `net.net.HttpResponse` -> `NetHttpResponse` (their
   `ce0c7fa`), so the http probe/fuzz parser are green. The other same-leaf
   groups (40 total; facade duplicates and genuinely conflicting
   collect/math/etc. types) still rely on first-wins; a dedicated
   compiler+stdlib slice (stdlib dedup first, then stdlib-wide R39
   qualification + their smoke battery) is the remaining work. Experiment
   evidence in docs/COMPILER_BUGS.md R44.

FIXED (R46): **bench graph clang-clean** -- generic same-leaf collisions are
shape-triaged, bare literals that are both struct and enum variant bind by
field names, mono tuple params use the concrete substitution, mono names are
identifier-sanitized, and method-receiver variants resolve leaf-scope.
Locks m87 (extended), m88, m89; bench IR deterministic at 5,808,645 bytes.

FIXED (R45): tuple element naming (Bool/`as` targets) + tuple defs spliced
into the type-decl block; lock `e2e_m87_tuple_element_types`; the stdlib
single-param sweep repro compiles/links/runs. Sweep-side exclusions for the
compiler run (stdlib/harness, not codegen): `xiom.os.env_unset` links
`unsetenv` (absent on Windows MSVC); `async_read_line(0)` passes a NULL
FILE* to `fread`; the run stops on an expected `requires` trip with dummy
args. See docs/COMPILER_BUGS.md R45.

FIXED (R43): `&v` on a reference-typed local now lowers to the stored
pointer (`&*v == v`) instead of C001 -- unblocks x25519_keypair /
_bigint_to_le; lock `e2e_m86_ref_of_reference`.

FIXED (R41/R42): generic pointer-self receiver ABI + scope-first bare-variant
resolution -- see docs/COMPILER_BUGS.md.

FIXED (R40): `derive[Clone]` on a pointer receiver -- the non-generic
instance-method receiver coercion now mirrors the by-value `%self` ABI
(loads the struct); lock `e2e_m85_clone_ref_receiver`.

FIXED (R39): the same-leaf TYPE collision -- catalog type qualification,
lock m84. See docs/COMPILER_BUGS.md R39-R43.

Fixed this session: R28 (temporary `.value`, lock m82), R29 (Vec in match arm,
lock m83), R30 (bare variant pick parity -- the old R25 residual; bench IR
byte-identical at 5,687,052 bytes pre-R39, 5,764,620 after).

## Working after the split (R31 contract)

The compiler repo and the stdlib repo are separate; the compiler repo no
longer contains stdlib sources:

- Fetch the pinned checkout: CI checks out `xiom-lang/stdlib` at the ref in
  `STDLIB_VERSION` (`.github/workflows/ci.yml` `Checkout stdlib at the pin`,
  release.yml likewise). The `scripts/fetch-stdlib.ps1|.sh` helpers referenced
  by the round-83 handoff are NOT in this checkout -- release-lane follow-up.
  Locally, clone into `stdlib/` or point `XIOM_STDLIB` at a checkout.
- Every cross-repo path resolves through the ONE helper
  `xiom_graph::paths`: `stdlib_root()` (XIOM_STDLIB -> exe-relative -> CWD ->
  XIOM_HOME fallback -> baked checkout), `stdlib_smoke_dir()`
  (XIOM_STDLIB_SMOKES -> `<stdlib>/tests/smoke/` -> legacy
  `examples/stdlib_smoke/`), and `stdlib_or_skip()` / `skip_if_missing()`.
- Missing checkout: loud SKIP lines locally; `XIOM_REQUIRE_STDLIB=1` (CI
  sets it) turns every skip into a hard FAIL.
- The stdlib lane owns moving the smoke corpus into `<stdlib>/tests/smoke/`;
  until it lands the legacy path keeps working.
- `stdlib/.gitignore` was added (generated runtime/smoke artifacts); it
  travels with the stdlib repo -- the stdlib lane may fold it into their
  bootstrap.
- Housekeeping done pre-split (round 83): ~13 GB of generated test binaries
  removed (repo root, `.test_build/`, `.testlogs/`, `tmp/`); `.gitignore`
  rebuilt (its tail had a corrupted UTF-16 block, so `.xiom_ai.json`,
  `.xiom_ai_cache/` and `xiom_verify_output.smt2` were never actually
  ignored); `.xiom_ai.json` untracked; harness temp sources (`_e2e_*.xi`,
  `e2e_*.xi`) ignored.

## Cross-lane notes (2026-09-18, compiler-lane answers)

**Website `docs/language/compiler.md` review.** `xiom-lang/website@c3d74f24`
(main) does NOT yet contain the consolidated Architecture/Modes sections or
the three flags; no open PR carries them. Verified facts to land them:

- `--runtime-contracts` (main.rs:388, help:1139): forces runtime contract
  guards even in release (`check_contracts || runtime_contracts`,
  lib.rs:627/1039). Release strips contracts without it.
- `--keep-debug-checks` (main.rs:391): release strips debug intrinsics
  (`set_strip_debug_checks(release && !keep_debug_checks)`, lib.rs:631/1042);
  the flag retains them. It was MISSING from `xiom --help` -- fixed
  2026-09-18 (now listed).
- `--enable-unsafe-direct` (main.rs:420-426, help:1134): allows
  `#[unsafe_direct]` in USER code; stdlib/trusted packages may use it
  without the flag (codegen context.rs:109-116, decl.rs:1139). Prints a
  warning when set.
- Crate list: **20** crates (Cargo.toml members): xiom, xiom-ast,
  xiom-check, xiom-codegen, xiom-ctfe, xiom-dbg, xiom-display, xiom-doc,
  xiom-ffigen, xiom-fmt, xiom-graph, xiom-jit, xiom-lexer, xiom-lowering,
  xiom-lsp, xiom-mcp, xiom-parser, xiom-pkg, xiom-verify, xiom-wasm. The
  website's "Architecture Decisions" row says 19 and lists a non-existent
  `cli` crate while missing `xiom-lowering`; `xiom` IS the CLI.
- v0.57 row verified: `#[unsafe_no_retry]` / `#[unsafe_direct]`
  (codegen), guard-heap arena + Copy-Out + guard pages
  (emitter.rs:904-912), SEH `__try/__except` trampoline + once-only
  transient retry (expr.rs:4973/4999), zero-escape gates T002/T003/T005/
  T006/T007 (checker, 35 sites).
- v0.58 row verified: numeric policy "cannot mix Int with Float64 --
  convert explicitly with `as`" (checker lib.rs:5421+), labeled loops
  (`@label:` parser:1131), debug intrinsics `dbg!`/`todo!`/
  `unimplemented!` (parser:2329) + `assert`/`debugger`, release stripping +
  `--keep-debug-checks`, `else if` accepted (empirical: check+compile+run
  probe), sublib-prefix resolution (checker lib.rs:146).
- AI_CONTEXT.md metadata refresh (website owns the file): version
  **v0.61.0**; compiler gates e2e 2338/2338, checker 195/195,
  feature-reg 510, robustness 63, fuzz 24, perf/determinism 2/2, fmt 86,
  lsp 45; stdlib pin `stdlib-v0.60.0`: 517 `.xi` files under `xiom/`,
  509 check_modules probes, 6,532 `pub fn` lines (the "512 modules /
  6,379 pub fns" line is stale). v0.57/v0.58 "New in" rows stay accurate;
  add a v0.59-v0.61 row for the repo split, R39/R44/R46/R46b same-leaf
  qualification, supply chain closure, single-source version 0.61.0,
  cargo-vet gate, and `xiom-pkg` in the archives.

**Registry lane**: `xiom-pkg` IS shipped now. `release.yml` builds
`-p xiom -p xiom-pkg`, stages both binaries in the Windows/Linux/macOS
archives, and asserts the staged `--version` of each before archiving
(CRB-3); the installer wrapper also dispatches `xiom pkg`. Verified
locally: `release/xiom-v0.61.0/bin/xiom-pkg.exe --version` -> v0.61.0.

**Installer / z3 ownership (superseded by CRB-3b below)**: the compiler repo
owns the installer and the
bundle LAYOUT -- `package.ps1` / `package.sh` / `tools/installer/*` build
the portable folder and copy `z3.exe` into `bin/` when one exists at
`target\release\z3.exe` or `%TEMP%\z3.exe`; `xiom-verify::find_z3`
auto-detects a bundled z3, common install paths, or PATH. The compiler repo
does NOT download or vendor z3, and the CI release workflow stages no z3,
so ACQUISITION/provisioning (and whether official archives bundle it) is
`/ops` release-side. Point the question at ops for z3; installer scripts
stay here.

**Playground lane (2026-09-19 handoff, C18/C19) -- FIXED (R47)**: the
`.to_str()`/container Str garbage and Float64-bit-pattern outputs traced to
four codegen/checker defects (conversion methods never injected without
`use xiom.fmt`; `unwrap_or` phi dominance violation; `ptrtoint double`
invalid cast; erased i64 payload/ABI type loss on chained receivers). All
fixed on `main`; lock `e2e_m91_conversion_methods` covers the full matrix
without importing `xiom.fmt`. See docs/COMPILER_BUGS.md R47. Playground
after updating the toolchain: `node tools/generate-expected-outputs.js
--wsl` then `node tools/lesson-audit.js --baseline
tools/lesson-baseline.json`. C17 (31 lessons cannot run at all) is NOT
confirmed compiler-side -- the audit's suggested next step; ask them to
re-run it on the fixed toolchain before assigning.

**Installer / z3 ownership -- RESOLVED (CRB-3b, 2026-09-19)**: ops decided
every archive ships all nine CLI tools and the pinned z3 solver. The
compiler repo owns the pin (`tools/z3-pins.json`, z3-4.13.4 SHA256-verified
per platform, Linux glibc floor 2.35), the platform-aware
`Z3Runner::find_z3` (bundled sibling `z3[.exe]`, `Z3_PATH`, common installs,
PATH; `xiom doctor` prints the resolved path), and the release.yml
bundle/stage/assert steps. Windows archives carry the app-local VC runtime
DLLs z3.exe needs; macOS carries `libz3.dylib`; both carry `LICENSE-Z3`.
`package.ps1`/`package.sh` still copy a z3 found at `target/release` or
`%TEMP%`; the CI path is the pinned download.

**CRB-3b extras**: every tool now self-reports (`xiom-mcp`/`xiom-dbg`/
`xiom-lsp` gained `--version`/`--help` before their stdio loops), and
`[profile.release] strip = true` shrinks the nine release binaries. The
staged-layout assert loop runs `--version` on all nine plus `z3 --version`
before each archive is sealed; CRB-4 guard and CRB-4b dispatch unchanged.

**Website help-parity follow-ups -- DONE (2026-09-19)**: `--help-ai` now
states the real LLM timeout default (10 s, matching `--help` and the
`unwrap_or(10)` in main.rs); `xiom --help` gained a "MORE OPTIONS
(advanced)" block listing every real flag it omitted (--check,
--emit-tokens, --debug/-g, --release, --opt-level, --lto, --cache,
--no-cache, --jit, --lazy, --parallel-codegen, --strict, --strict-mode,
--strict-exhaustive, --static, --standalone, --max-depth, --count,
--bench-file, --test-dir, --test, --scaffold, --clean, --explain,
--registry, --locked, --frozen, --ai-silent, --batch) plus repl /
build-runtime subcommands and the launcher's tool dispatchers; `--strict-mode`
is now an accepted alias of `--strict` (docs spelling) with a unit test.

**CRB-3c -- XIOM_HOME alignment DONE (2026-09-19)**: `xiom_graph::paths`
now owns the installed-home resolver: `XIOM_HOME` wins when set, otherwise
the first EXISTING candidate from the canonical installer layout
(`%LOCALAPPDATA%\xiom` / `$XDG_DATA_HOME|~/.local/share/xiom`) then legacy
`~/.local/xiom`, `~/xiom`; when nothing exists it returns the canonical
default so diagnostics stop guessing `~/xiom`. `xiom doctor` and `xiom doc`
use it (`doctor` prints the searched list when the stdlib is missing), and
the root source installer now installs to `~/.local/share/xiom` like the
shipped installer. `xiom-graph` 31/31 (new candidate-order/dedup test);
doctor smoke: canonical install found with no env, XIOM_HOME override
honored.

**Ops release checklist state**: CRB-1 (single-source 0.61.0), CRB-2
(banner/literal sweep), CRB-4 (tag==workspace-version + on-main guard),
CRB-4b (compiler-release docs dispatch), CRB-5 (package scripts read the
workspace version, never rewrite Cargo.toml) are all DONE and on
`origin/main`; CRB-3 (xiom-pkg) and CRB-3b (nine tools + pinned z3) are
accepted. The ops list repeating 1/2/4/4b/5 predates those pushes.

**AI-mode audit (2026-09-19)**: the installer's AI configuration was DEAD
(it wrote `KEY=VALUE .xiom_ai_config` with `XIOM_AI_API_KEY`, while the
compiler only read JSON `.xiom_ai_config.json` from cwd/home), and
`--ai-local` was parsed but never enforced. Fixed:

- both installers write JSON to `$XIOM_DIR/.xiom_ai_config.json` (the
  compiler now also searches `XIOM_HOME`), chmod 600 on Unix plus a
  plaintext-key warning; the shell wrapper exports `XIOM_HOME`;
  `.xiom_ai_config*` is gitignored.
- `finalize_config` enforces the `--ai-local` contract: Ollama + loopback,
  cloud key dropped, cloud-default model swapped to codellama; a remote
  endpoint is refused.
- a non-empty API key is refused over plaintext http:// to non-loopback
  hosts (`XIOM_AI_ALLOW_HTTP=1` opt-out for trusted proxies); the
  non-silent run prints the endpoint and whether env or a config path
  supplied it.
- prompts now carry the diagnostic MESSAGE + file + compiler version, mark
  code snippets as untrusted input (prompt-injection hardening), and ask
  for `FIX: / WHY: / Confidence:`; `XIOM_AI_TIMEOUT` and
  `XIOM_AI_MAX_TOKENS` are wired and documented; docs/AI_PIPELINE corrected
  (the cache "random session salt" claim was false; install config and HTTP
  rule documented).
- legacy `xiom install` / `xiom update` / `xiom registry` now use
  `<XIOM_HOME>/packages` and `<XIOM_HOME>/registry.json` (an explicit
  XIOM_HOME always wins; an existing legacy `~/.xiom/registry.json` is
  still honored) so the old spelling sees `xiom pkg` installs.

**Playground R48 (2026-09-19) -- fixed 3 classes, cache hazard, fmt dispatch,
WASM asset**: interface dispatch through `&T` generic args was mono'ing as
`Int` (type_from_ast strips the ref; the bare-T branch had no Ref arm) and
now resolves the argument's type (L6-01..07/13/16/30 pass); match-result
slots zero-init pointer/double as `null`/`0.0` (clang rejects
`store i8* 0` / `store double 0`; L2-12/14/15/16 build); the script cache
key now includes the compiler build identity (a new build used to serve the
old build's binaries); `xiom fmt|lsp|mcp|pkg|dbg|verify|ffigen` dispatch
natively via the sibling binary; release.yml ships `xiom-wasm-<ver>.wasm`
plus `bin/xiom-wasm.wasm` (C8). Lock `e2e_m92_interface_dispatch_zero_init`.
OPEN with repros in docs/COMPILER_BUGS.md R48: L6-28/L6-40 (interface T from
struct generic), L5-40 (nested-generic mono name), L6-31 (qualified ctor
redefinition), L8-15/18 (i64 passed as ptr), L6-05/L2-19 runtime AVs, 26
C18/C19-residue lessons, and the fmt-closure perf item (reachable-function
peek, deferred to Stage 6).

## Remaining queue (compiler lane)

1. **Supply-chain tail -- CLOSED (2026-09-17)**: transitive dependency
   closure landed. `select_version` matches `>=,<,<=,>,=,^,~` specs over
   non-yanked versions (exact pins still resolve yanked releases); install
   walks a cycle-safe deterministic `install_closure` whose dependency
   source is the VERIFIED tarball's own `package.xi` (index metadata as
   fallback), and every transitive artifact goes through the full
   verification path (sha256 + signature + lockfile). `lock` pins the
   transitive closure with digests (`Lockfile::from_resolved`). Server-side
   publish authentication is registry-side; `publish_package` is
   implemented. Registry e2e **20/20** (new core fixture + closure install +
   transitive lock assertions).
2. **clap migration -- DONE (2026-09-18)**: `crates/xiom/src/cli.rs` is the
   parser of record (53 boolean, 3 optional-value, 18 required-value, `-o`,
   `-g`, positionals; lenient `ignore_errors`; help/version auto-flags
   disabled) and now returns a `Cli` whose matches drive the main-path reads
   (`flag`/`value`/`values`/`present`), while `Deref` to the original argv
   keeps the early dispatch, `run`'s mini-language, command words and the
   `--flag=value`-sensitive diagnostics/sandbox/graph checks byte-identical.
   5 unit tests pin the surface, representative invocations, and the reads.
   The two flag-scanning helpers were deleted. Verified by the full e2e.
3. **fmt: body-inline comment trivia attachment -- DONE (2026-09-18)**.
   `format_source_text` threads lexer trivia (comments) and closing-brace
   token positions into the Formatter: leading comments emit above the
   statement/decl, same-line comments stay trailing, comments before a
   block's `}` stay inside it; expression-internal comments attach to the
   next statement or block close (never dropped). Idempotent; 3 new tests
   + real-file round-trip. Remaining fmt polish: none tracked.
4. **LSP cross-file index -- DONE (2026-09-18)**: `Backend` keeps a
   `FileIndex` (symbol -> declaration sites) built lazily from the open
   document's project roots (document dir, `src/`, graph source roots),
   cached across requests and rebuilt after didOpen/didChange/didClose.
   `textDocument/definition` now resolves declarations in files that were
   never opened (open documents still win), and `path_to_uri` round-trips
   Windows drive letters. Root discovery deliberately does NOT walk broad
   parents (a temp-dir file must not index all of /tmp); traversal is
   bounded (4000 files / depth 24, skip dirs). lsp 45/45 (new
   `test_definition_cross_file_index_unopened_file` covers index hit +
   rebuild-after-change).
5. **Driver hygiene -- DONE (2026-09-18)**: per-invocation temp names are
   randomized across the driver (watch/REPL/script siblings, jit link
   artifacts) and the JIT temp directory now mixes pid with the
   time+counter suffix and is removed after the library is dropped (pid
   reuse plus a stale dir could previously hit the same paths).
6. **cargo-vet audits -- DONE (2026-09-18)**: `cargo vet` (0.10.2) is
   bootstrapped with `supply-chain/config.toml` exempting the current 171
   crates ("safe-to-deploy"); the CI hygiene job installs cargo-vet and runs
   `cargo vet` next to cargo-deny, so any new dependency or version bump is
   unvetted until audited or explicitly exempted. Imports/audits files are
   empty until the first upstream audit import.
7. **Same-leaf TYPE collision -> FIXED (R39, 2026-09-17)**: catalog injection
   module-qualifies colliding non-generic type leaves and rewrites references
   (`type_qualify.rs`; lock m84). The next bench clang error is the generic-mono
   Pair ABI mismatch -- see "Open compiler findings".
8. **Stage 6 continuation**: real incremental engine, parallel
   monomorphization profiles, linker strategy, more budget metrics.
9. **Stage 7 selfhost ladder**: v092..v11 are milestone emitters, not yet a
   full XIOM-in-XIOM compiler; zero-ICE self-build is a multi-phase project.
   The selfhost COMPILE gate is green.
10. **Release pipeline (website + ops lanes, 2026-09-18) -- DONE**:
    `docs/COMPILER_RELEASE_BATCH.md` (xiom-lang/ops) executed. Version is
    single-source at **0.61.0** (`Cargo.toml [workspace.package]`, 20
    workspace crates + the fuzz workspace): REPL/doctor, `xiom-pkg` help,
    `xiom-dbg` help, `xiom-wasm get_version` all print
    `env!("CARGO_PKG_VERSION")`; `xiom.bat`, ascii art, MCP manifest and the
    installers/package scripts carry no literals (installers resolve
    XIOM_VERSION env -> release dir name -> workspace Cargo.toml; package
    scripts default `-Version` from the workspace and no longer rewrite
    Cargo.toml). `release.yml` has a `guard` job (tag == workspace version
    AND tag is an ancestor of main; dry runs read the version), builds
    `xiom` + `xiom-pkg`, stages both binaries in Windows/Linux/macOS
    archives, asserts the staged `--version` output matches the label, and
    the `compiler-release` dispatch (client_payload {tag, stdlib_ref,
    compiler_ref=tag}) now uses `XIOM_RELEASE_TOKEN` -- extend that PAT to
    `xiom-lang/website` (Contents: read/write) or the dispatch 404s.
    Verified locally: workspace check --all-targets, release build, both
    `--version` + doctor, pkg/dbg/wasm/xiom tests, and a local
    `package.ps1` run (archive `xiom-v0.61.0-windows-x64.zip` carries
    bin/xiom.exe + bin/xiom-pkg.exe, both self-report 0.61.0).

Cross-lane pending (stdlib lane, pre-existing): `stdlib_api_freeze_no_removals`
RED (52 drifted signatures since the 2026-08-07 snapshot) and
`stdlib_tests::stdlib_all_modules_compile_to_ir` RED
(`xiom.encoding.ascii85` T001). `STDLIB_VERSION` is now `stdlib-v0.60.0`
(release lane swapped it in).

## Workflow rules

- e2e/stdlib harnesses spawn `target/debug/xiom.exe` (release fallback):
  `cargo build -p xiom` after ANY checker/codegen change or the suites test
  a stale driver.
- Full e2e is ~18 min; run it for any resolution/codegen change. Fast gates:
  `cargo test -p xiom-check`, `--test feature_regression_tests`,
  `--test stdlib_execution_tests`, `--test perf_budget_tests`.
- Shared target dir can go stale (`cargo check` disagreeing with an isolated
  build): `cargo clean -p xiom-codegen -p xiom`, then rebuild the driver.
- PowerShell: capture `$LASTEXITCODE` immediately after each native call;
  `1> file` writes UTF-16 (use `[System.IO.File]::WriteAllLines`/`-Raw` when
  byte-exact IR/output is needed).
- ASAN-compiled programs need the LLVM runtime dir on PATH
  (`C:\Program Files\LLVM\lib\clang\22\lib\windows`); cargo-fuzz targets run
  locally only with a version-matched ASAN DLL -- use CI/Linux for those.
- Corpus triage: `cargo test -p xiom-check catalog_corpus_is_clean`;
  `$env:XIOM_CATALOG_DUMP='1'` prints every site.
- Stdlib boundary: monorepo phase -- never stage `stdlib/**` /
  `examples/stdlib_smoke/**` (parallel lane's files); post-split -- `stdlib/`
  is a gitignored checkout, never commit it. Report stdlib findings in
  `docs/COMPILER_BUGS.md` and in chat.

## Paste-ready prompt for the next compiler session

```
Continue the XIOM compiler-lane campaign in E:\xiom-lang\xiom (the compiler
repo; post-split, branch `main`). Read SESSION.md -- the "Current state
(2026-09-19, post-R48)" section at the top -- and docs/COMPILER_BUGS.md R48
before touching code. Stage 5 is COMPLETE and all R-bugs through R48 are
fixed; this session's mission is to CLOSE ALL OPEN COMPILER BUGS (the R48
residue) with production-grade fixes, not to start new features.

Stdlib: clone `xiom-lang/stdlib` at `STDLIB_VERSION` into `stdlib/` or set
XIOM_STDLIB before any stdlib/smoke test; everything resolves through
`xiom_graph::paths` (CI uses XIOM_REQUIRE_STDLIB=1 with the pin checked
out). Never commit the `stdlib/` checkout.

State (verified on 56b6e0e3): e2e 2340/2340, feature-reg 510/510, checker
195/195, robustness 63/63, fuzz 24/24, perf/determinism 2/2, fmt 86/86,
lsp 45/45, stdlib-exec 85/85 (+2 ignored), xiom lib 32/32, pkg/dbg/mcp
63/34/39, cargo deny + cargo vet (175 exempted), ASCII guard, bench IR
5,808,645 bytes + `clang -c` exit 0. Workspace version 0.61.0. release.yml
ships all nine tools + pinned z3 + the wasm asset; the tag==version/on-main
guard and the compiler-release docs dispatch are wired. Git is clean and
pushed.

OPEN BUGS (docs/COMPILER_BUGS.md R48 has exact repros -- reproduce each
BEFORE fixing):
1. C17 interface residue -- L6-28 (`PriorityQueue[T: Priority].insert`:
   T must infer from the struct's generic), L6-40 (module-scoped
   `Runner[T: Plugin]` dispatch). Both still fail codegen with
   `C001: type 'Int' does not implement ...`.
2. C17 clang residue -- L5-40 (`alloca %struct.Option__Vec_Str_` nested
   generic mono name never defined), L6-31 (`invalid redefinition of
   function 'school.students.new_student'`, the C11 qualified nested
   constructor emitted twice), L8-15/L8-18
   (`%tmp defined with type 'i64' but expected 'ptr'` -- a Vec receiver
   reaches `get_Int(%struct.Vec* ...)` as an i64 handle).
3. C17 runtime access violations -- L6-05 and L2-19 build but crash with
   `0xC0000005` at run.
4. C18/C19 residue (26 lessons) -- 19 nondeterministic pointer prints
   (L0-11, L3-01/02/07/50, L5-02/05/07/09/20/24/26/29/31/35/36/43, L8-14,
   L8-20), 7 invalid-UTF-8 outputs (L0-34, L0-49, L0-50, L5-32, L5-34,
   L5-42, L7-09), L5-21 (Float64 `.to_str()` inside a generic over
   `Vec[T]` still prints IEEE bit patterns).
5. Perf -- the auto-injected `xiom.fmt` closure costs +2.3-2.5 s on the
   first `.to_str()` compile. Implement the reachable-function-only peek in
   `xiom-check::collect_external_decls` (peek the checker-resolved module
   shallow, run the reachability filter, then pull the deps named by the
   SELECTED decls to a fixpoint) and gate it with the full e2e.
6. C3 (script-mode flags) and C6 (stdlib `package.xi`): request the exact
   playground repro before changing semantics; document if deferred.

Repro recipe: `git clone --depth 1 https://github.com/xiom-lang/playground
tmp/playground`; every `lessons/**/Lx-yy.json` has a `.solution` field --
extract it to `tmp/lessons/Lx-yy.xi` and run `target\debug\xiom.exe -o
tmp\lessons\Lx-yy.exe tmp\lessons\Lx-yy.xi` (run `cargo build -p xiom`
after any checker/codegen change first). Determinism check: run the exe
twice and compare stdout hashes.

Method: reproduce -> minimize to a fixture under `tests/regression/m93_*`
-> fix with the smallest correct change -> add an e2e lock + the CI lock
line -> mark the finding FIXED in docs/COMPILER_BUGS.md (with the repro and
the evidence) -> update SESSION.md -> atomic commit (code + docs + locks +
CI lock line together). Drop temporary debug prints before committing.

Verification commands: `cargo test -p xiom-codegen --test e2e_tests`
(2340+, ~25 min; the stdlib-dependent tests need the checkout);
`--test feature_regression_tests` (510), `-p xiom-check --lib` (195),
`--test perf_budget_tests` (determinism canary + budgets),
`--test robustness_tests` (63), `--test fuzz_tests` (24),
`--test stdlib_execution_tests` (85 +2 ignored), `cargo test -p xiom --lib`
(32), `cargo test -p xiom-pkg -p xiom-dbg -p xiom-lsp -p xiom-mcp`,
`cargo vet`, `cargo deny check advisories licenses bans sources`, IR gate:
`xiom --emit-ir examples\benchmark\main.xi > b.ll; clang -c b.ll -o NUL`
must exit 0. Capture $LASTEXITCODE immediately after every native command.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Use --emit-ir /
clang -c / --sanitize=address for miscompile work. Never commit `stdlib/**`
or the `stdlib/` checkout. Conventional commits, atomic slices. The whole
suite must be green before each commit; a full e2e takes ~25 min, so batch
related fixes per run and always re-run it after a codegen change.
```

