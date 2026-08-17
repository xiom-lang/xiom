# XIOM Compiler Session — Handoff (2026-08-17)

## Session handoff (2026-08-17) — CLEAN STATE, work committed

Branch: `feat/architect` (**37 commits ahead** of `origin/feat/architect`).
All work this session is COMMITTED (5 commits below). Working tree has ONLY
pre-existing third-party modifications — `.xiom_ai.json`, `Cargo.lock`,
`docs/AI_CONTEXT.md` (NOT mine; leave them alone).

### Commits this session (in order)

```
e477815b docs: report BUG 32-38 queue results + stdlib-side list to the stdlib session
802e9fd2 docs: BUG 32-38 queue status - 38/38b/32/33/31/34 fixed, 35 verified, 37/36 open, P001 resolved
b15d0d3b fix(codegen): BUG 34 - nested Vec[Vec[T]] element writes
74bcc28b fix(codegen): BUG 32, 33, 31 - Int-to-ptr cast, Option[Float128] payload, fp128 fneg
9042e8a2 fix(codegen): BUG 38 family - is-Some double-check, generic-receiver mono, mutating self ABI
```

### Fixed & verified (isolated binary `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- **BUG 38** — bare `is Some` scrutinee payload rebind fires ONLY inside an
  Imply left side (sx4 pos=5; iter.xi collect works).
- **BUG 38b (new)** — generic-receiver methods (`Iterator[T].collect`): the
  parser DROPPED receiver generics → erased i64 ABI → never monomorphised
  (45 decls in iter/core/collections/sync/rc/memory). Four-part fix: parser
  receiver-generics capture; receiver-keyed mono dispatch; find_generic_decl
  prefers UNDECLARED abstract receivers; mutating-self ABI (block_mutates_self
  recursion + new block_mutates_receiver_state). iter smokes 0/19 → 6/19.
- **BUG 32** (Int-var→ptr cast now load+inttoptr), **BUG 33** (Option[Float128]
  payload → fp128), **BUG 31** (fp128 fneg), **BUG 34** (nested Vec[Vec[T]]
  writes: i64-element dispatch + element-address ABI + elem_size 32 +
  elem-type tracking).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  [1,1,1], no crash).
- **P001** not reproducible; indented decls reject in 0.14s; former 15-min
  hang smoke compiles in 4.9s.
- Suites: checker **178/178**, codegen **2263**, feature-reg **510**, parser
  **24** — all green. `stdlib_api_freeze_no_removals` FAILS IDENTICALLY AT
  BASELINE HEAD (905 frozen signatures missing — item B family, stdlib
  layout moved; NOT a regression).

### Re-triage of the stdlib session's remaining lists (238 files, current binary)

**43 pass / 82 compilefail / 113 runfail.** Results: `%TEMP%\kilo\resweep_new0-3.txt`.
Remaining families: (a) receiver-CALL chains for generic methods (iter
adapters — reproduces at baseline, B-007-adjacent); (b) BUG 24/36 shape
miscompiles; (c) stale-API smokes (`.get(0)` on Vec, char.from_digit
contract false-fire, is_empty). Full breakdown: COMPILER_BUGS.md
(2026-08-16 late section) + REPORT_TO_STDLIB_SESSION.md.

### OPEN queue (next sessions, priority order)

1. **BUG 37/36 (HIGH)** — fp128 + BigFloat-chain shape AV. Minimal
   deterministic repro in `%TEMP%\kilo\` (t_chainloop.xi, t_b37f.xi,
   t_b37k.xi): `var n = v.significand.digits.len();` (chain Vec-len) used
   as a loop bound + ANY fp128 op in the loop body → 0xC0000005 even at
   clang -O0 with verifiably sound IR. `bigfloat_to_float128` still crashes
   for non-zero values (the stdlib's three-shape workaround did not dodge
   it — the main fn still mixes chain + fp128). Probe prints change loop
   semantics — classic shape miscompile.
2. **Item B (HIGH)** — exec harness wiring: stdlib_tests.rs +
   stdlib_execution_tests.rs + stdlib_api_freeze_tests.rs still reference
   the pre-refactor layout (STDLIB_MANIFEST.md 515 paths, STDLIB_SMOKES.md
   213 smokes, frozen-signature snapshot). The 5-test-worktree merge wave
   waits on this. **The stdlib session has the GREEN LIGHT to wrap up** —
   its remaining items are the stdlib-side list in
   REPORT_TO_STDLIB_SESSION.md (char.xi from_digit requires-contract on a
   fallback fn; Vec.is_empty broken — baseline-confirmed; stale `.get(0)`
   smoke usage → use indexing; smoke_num_saturating Bounded/Ord impls;
   smoke_alloc_basic `use xiom.ptr;`).
3. **Item C (HIGH, user-approved)** — catalog body type-checking: Phase 1
   reachable injected fns; Phase 2 all catalog bodies (hash-keyed cache);
   Phase 3 `--strict-stdlib` gate.
4. **Iter adapter chains (MEDIUM)** — receiver-CALL chains
   (`iter.range(1,6).max()`, `.map(fn...).collect()`) + fn-value params:
   reproduces at baseline; B-007-adjacent (closure-through-fn-slot design
   decision pending).

### Scratch / repro files this session (in `%TEMP%\kilo\`)

t_b37f/t_b37k/t_chainloop (BUG 37/36), t_b35/b35b (BUG 35 verified),
t_b34 (BUG 34), t_b33/t_b33u (BUG 33), pdb3 (BUG 32), t_fneg (BUG 31),
t_iter38/t_b1/t_clike (BUG 38b), sx4 (BUG 38). All exit 0 with the current
binary except the t_b37* family (AV) and t_vecloop (my test's inverted
expectation — values correct).

---

## Session update (2026-08-16 late): BUG 32-38 queue progress

Commits this session: `9042e8a2` (BUG 38 + 38b generic-receiver family),
`74bcc28b` (BUG 32/33/31), `b15d0d3b` (BUG 34).

- **BUG 38 FIXED** — bare `is Some` scrutinee rebind now fires ONLY inside
  an Imply left side (sx4 pos=5).
- **BUG 38b (new) FIXED** — generic-receiver methods (`Iterator[T].collect`)
  lost their receiver generics in the parser; receiver-keyed mono dispatch +
  abstract-receiver decl preference + mutating-self ABI (block_mutates_self
  recursion + block_mutates_receiver_state). iter smokes 0/19 → 6/19.
- **BUG 32 FIXED** (Int→ptr cast inttoptr), **BUG 33 FIXED** (Option[Float128]
  payload → fp128), **BUG 31 FIXED** (fp128 fneg), **BUG 34 FIXED** (nested
  Vec[Vec[T]] writes: dispatch + element-address ABI + elem_size 32).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  correct); extreme variant needs the stdlib repro.
- **BUG 37/36 OPEN** — minimal deterministic repro: BigFloat field-chain
  Vec-len as a loop bound + any fp128 op in the loop → AV even at clang -O0
  with sound IR (t_chainloop/t_b37f). `bigfloat_to_float128` still crashes
  for non-zero values. Deep-dive needed.
- **P001 not reproduced** — indented decls reject in 0.14s; the former
  15-min hang smoke compiles in 4.9s. The 238-file re-sweep completed
  without hangs.
- Re-triage of the stdlib session's two lists: 238 files → 43 pass, 82
  compilefail, 113 runfail (details in COMPILER_BUGS.md + the report file).

Full statuses: docs/COMPILER_BUGS.md (2026-08-16 late section). The queue
items B (exec harness) and C (catalog body checking) are still OPEN.

Branch: `feat/architect` (32 commits ahead of `origin/feat/architect`).
COMPILER session. The parallel stdlib session is MISSION COMPLETE
(512 modules, 6,379 pub fns, 0 stubs, layout FROZEN); my replies live in
`docs/REPORT_TO_STDLIB_SESSION.md` (last updated `707a299c`).

## Session summary (2026-08-16): the 16 regression failures are FIXED + BUG 31 batch

The 04:03 test run (`session_20260814_040310.txt`) had **12 e2e failures +
4 stdlib-exec failures + 1 diff failure**. All compiler-fixable ones are now
green; the remaining items are stdlib-side or latent.

### Commits this session

```
5940ba2c fix(codegen): struct_type_from_expr resolves Type.method static calls without the bare alias
0ddc4500 fix(codegen/check): BUG 30 batch - 13 regressions from stdlib layout era (e2e 12 + stdlib-exec rc/cell/utf8)
e1bd8284 chore: remove TEMP-DEBUG vec2_dbg probe from xiom.geom.vec; bump session metadata
36bfb32a test(stdlib-smoke): restore full lz4/snappy round-trip coverage
```

### The 14 fixes (commit 0ddc4500 + 5940ba2c)

Compiler (xiom-codegen):
1. **Imply scoping** — contract-ensure `is Some/Ok/Err` payload rebinds no
   longer poison later return-site checks (the payload slot local persisted
   → i64-form Is inttoptr+load → AV). Fixes b003 / m19_read_file /
   file_stem / file_name chains.
2. **Is() payload-slot hoisting** (struct + i64 forms) — `&&` guard chains
   bind in one block, read in a later one → "Instruction does not dominate
   all uses" (m18_guard_0086).
3. **Is() bare-scrutinee rebind in the enum-variants form** — the fallback
   path had it; the enum-variants path didn't → ensure `result.len()`
   resolved to Map.len (utf8).
4. **struct_type_from_expr Call-arm resolution** — bare/module-qualified
   callees resolved through call-site key machinery (caller module →
   bare_fn_aliases → use_alias_map). Match-on-catalog-call dropped the
   scrutinee → no tag checks + payload literal 0 → AV (m35_z24/z29,
   eco_algo).
5. **Mono path flushes hoisted allocas** — `var t = b;` in a generic fn's
   while loop emitted only the store (m35_z24 "undefined value %tmp55").
6. **Enum literal ctors zero-init unused payload slots** (bare/qualified
   struct forms + Ok/Err) — LLVM poison → clang -O2 miscompile →
   deterministic AV (m35_z10/z29, bisect_z29_m). The driver runs `opt -O2`
   + `clang -O2` — poison is exploited.
7. **coerce_arg_for_param `&array_local`** — data pointer only for `&[N]T`
   params; `%struct.Vec*` params get the header alloca (eco_algo
   len()==element-0).
8. **len() dispatch for boxed Vec-handle locals** — contract rebind payload
   with Vec XIOM type unboxes via inttoptr+load (utf8 ensure).
9. **Ensure `result` scoped per-check** — a USER local named `result`
   shadowed the synthetic return-slot binding (utf8_encode AV).
10. **decl.rs result local_xiom_types uses type_string_full** — the payload
    args survived (Result[Vec[UInt8], Str] not just "Result").
11. **collect_block_free_vars sorts captures by name** — HashSet iteration
    order differed between the block fn / ctx type / caller stores in
    different passes (Rc.new slot swap — fields written through VALUE 42 as
    a pointer).
12. **Generic receiver ABI** — by-value `self` passes the struct VALUE not a
    pointer (smoke_rc Rc.get garbage: `%struct.Rc` vs `%struct.Rc*`).
13. **Checker wildcard method lookup deterministic** (sorted candidates,
    receiver-base preferred) AND **clone skip only applies to the fallback**
    — the v0.56 clone "fix" discarded the DIRECT hit too, so `r.clone()`
    typed `_` and `r2.get()` hit BTreeMap.get → Option (smoke_rc).
14. **struct_type_from_expr Type.method static calls** — `Vec[Str]::new()`
    must resolve `Vec.new` (exact / module-qualified suffix); when the typed
    key is unregistered (catalog compile ORDER: io.xi before
    xiom.collections) return None — NEVER the bare alias (first registrant
    BufReader.new) → BufReader.invariant_check(%struct.Vec …) → invalid IR
    (smoke_net_http, the io module graph).

File-side (same commits): bench_math Float64 casts (644/656), selfhost
xiom-check pub+use for 7 test fns, xiomc_v11_test `elif pfound {`, m21
Vec.pop Option handling.

## BUG 31 batch (2026-08-16, second half — the 904-sweep queue)

Commits: `2b238da4` (fmt), `b28ac72b` (Map), `a7571ac7` (variant hijack),
`99f894b7` (tuple destructure), `de639432` (docs).

- **fmt cluster (8 fixes, 8 smokes green):** Unit fields in
  Result[Unit, FmtError] literals (`store void 0, void*` invalid IR —
  generic literals adopt the fn's concrete return type; field types
  degrade Unit→i64); Str.to_str passthrough read the first BYTE (prologue
  treated i8* as a struct pointer — only %struct.X* receivers take the
  pointer branch); primitive/variable method receivers resolved to bare
  stubs (infer_struct_type_name learns literals/negated/parens, primitive
  locals via local_xiom_types, generic params via param_concrete_types,
  infer_value_xiom_type learns literals); mutating `self` methods lost
  mutations (block_mutates_self → pointer ABI in compile_fn + signature
  registration).
- **Map cluster (4 fixes, 7 smokes green):** `&K` scalar params passed the
  VALUE — four coordinated fixes (param_llvm_type, mono subst_type Ref arm,
  generic-call inference, coerce materializes plain values for pointer
  params).
- **Variant-hijack:** struct literals whose name collides with an enum
  VARIANT (Node{...} vs enum BST { Node(...) }) — bare-variant search
  yields to known types. bench_math native IR now valid (was
  store %struct.BST %vecval).
- **Tuple destructure (BUG 26 #3):** the checker bound every name to the
  whole Tuple__A__B; now splits element types.

**Verified fixed:** BUG 26 #2 (bare prelude names), BUG 26 #5 (high-bit
mask AND), BUG 24 residual (smoke_num_precision). **Stdlib-side:**
smoke_num_saturating needs real Bounded/Ord impls; smoke_alloc_basic needs
`use xiom.ptr;`; smoke_hash_folder needs `use xiom.convert.toint;`.
The stdlib session also filed BUG 32-38 (fp128/ptr-cast/nested-Vec-write/
is-Some-double-check families) — the next compiler queue.

### Verified green (isolated binary `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- Checker suite: **178/178**.
- All 14 previously-failing e2e/stdlib-exec tests run exit 0
  (b003, m19_read_file, m18_guard_0086, m35_z24/z10/z29, eco_algo,
  smoke_rc, smoke_cell, smoke_utf8, smoke_compress_lz4_snappy,
  m21_vec_edge_004, xiom-check.xi).
- diff-test expectations hold (the 04:03 f=1 was binary-churn race).
- bench_math: `--emit-ir` passes (harness is emit-ir-only) — native `-o`
  still fails on a LATENT struct-literal field-type bug (below).

## Open items (next sessions, in priority order)

### 0. [HIGH — NOW] Rebuild + re-triage the remaining smoke failures (2026-08-16 late)

The stdlib session's 905-smoke sweep (621 pass / 300 fail, done against the
STALE 17:36 binary) produced exact lists:
`%TEMP%\kilo\remaining_compilefail.txt` (59) + `remaining_runfail.txt` (190).

**BEFORE triaging anything:**
1. Rebuild the isolated binary — my BUG 31 fixes (`2b238da4`, `b28ac72b`,
   `a7571ac7`, `99f894b7`) are NOT in the 17:36 binary. The fmt/io/regex
   clang clusters and ALL `bisect_*` entries (deleted scratch) should drop
   out of the lists immediately.
2. Re-run both lists against the new binary; filter:
   - `bisect_*` — my deleted scratch, ignore.
   - `smoke_fmt_*`, `smoke_stress_fmt_*` — fixed (fmt cluster).
   - `smoke_rc/cell/utf8`, `smoke_stress_collections_map_*` — fixed.
3. The ~60 remaining compile-fails and the run-fails cluster into families
   that need triage: **array/char/cell/cmp/collections/convert/core/error/
   io/num/ptr/rc** — each is either a REAL compiler bug (like the fmt/Map
   clusters were) or stdlib-side API drift. Bisect each family like the
   fmt/Map clusters were done (small repro → root cause → fix).
4. Known stdlib-side (hand to the stdlib session): smoke_num_saturating
   (Bounded/Ord impls needed), smoke_alloc_basic (`use xiom.ptr;`).

**Edit-race caution (stdlib session's warning):** their realignment script
ran while smoke crypto/net/stress files showed as modified; if any of my
uncommitted smoke content is missing from the tree, re-apply it. Current
tree has NO uncommitted smoke changes (all committed) — verify with
`git status --short examples/stdlib_smoke` before mass-editing.

### A. [HIGH — pre-selfhost] Catalog body type-checking (Q2b, user-approved)
NOT STARTED. Plan unchanged:
Phase 1 check reachable injected fns; Phase 2 all catalog bodies at load
(hash-keyed cache); Phase 3 `--strict-stdlib` CI gate.

### B. [HIGH] api_freeze path sync + exec harness
Unchanged: `crates/xiom-codegen/tests/stdlib_tests.rs` +
`stdlib_execution_tests.rs` still reference the pre-refactor layout
(STDLIB_MANIFEST.md 515 paths + STDLIB_SMOKES.md 213 smokes).
**The merge wave (5 idle test worktrees) WAITS on this** — the stdlib
session is ready to run it once item B lands.

### C. [MEDIUM] BUG 32-38 — the stdlib session's new compiler queue
Filed in COMPILER_BUGS.md (2026-08-16 evening), priority by blocker:
1. **BUG 37** — fp128 RETURNED from a catalog fn crashes the caller
   (0xC0000005) — blocks any consumer of `bigfloat_to_float128`.
2. **BUG 32** — `x as *T` (Int VARIABLE to pointer) emits address-of-local,
   not inttoptr — blocks pointer-handle designs (glob).
3. **BUG 38** — `if x is Some { match x { Some(v) => ... } }` double-check
   binds the payload as 0 — BUG 30 #1/#2 family; semver worked around it.
4. **BUG 34** — nested Vec element WRITES via `&mut` AV (reads work).
5. **BUG 35** — Int128 index math inside a Vec-writing fn shape AV.
6. **BUG 36** — fp128 Horner-loop fn shape AV when sign statements follow.
7. **BUG 33** — Option[Float128] unwrap loads undefined `%struct.Float128`
   (payload type lookup must map XIOM Float128 → LLVM fp128).
8. **BUG 31 (stdlib's)** — unary minus on Float128 emits `sub i64 0, fp128`
   (fneg path must handle fp128).
9. **P001 hang** — indented module-level declarations HANG the compiler
   (stdlib realigned 158 files; the parser should reject, not hang).

### D. [DONE] 904-smoke battery — BUG 31 batch closed the compiler-side
fmt (8 smokes), Map (7 smokes), variant-hijack (bench_math native), tuple
destructure (BUG 26 #3) all fixed; BUG 26 #2/#5 + BUG 24 residual verified
fixed. Remaining queue is item 0 (re-triage) + C above.

### E. [MEDIUM] MCP catalog/registry tools (ROADMAP G) — unchanged.
### F. [LOW] Roadmap E/F: reverse type-index, contract policy decision.

## Sequencing per the user (pre-selfhost → split → registry) — unchanged:
1. Selfhost gate: stdlib 100% + compiler 100% + zero warnings + item A + B.
2. Monorepo split per docs/REPO_SPLIT.md ONLY after (1) green.
3. Registry + packages after the split.

## How to verify anything you touch

- Rebuild the isolated binary: `$env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso";
  cargo build -p xiom` then use `tgt_iso\debug\xiom.exe` (NEVER the shared
  target/debug binary — parallel sessions rebuild it constantly).
- After any change: checker (`cargo test -p xiom-check --target-dir
  "$env:TEMP\kilo\tgt_check"`) + the 14-test battery above + the 904-smoke
  sweep (run in 8 parallel batches of ~115; each batch ~25 min).
- Never stash/revert uncommitted stdlib work without a fresh backup.
- The stdlib session's next message should include the D-section survey so
  they can fix the stdlib-side half of the smoke failures.
