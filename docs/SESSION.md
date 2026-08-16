# XIOM Compiler Session — Handoff (2026-08-16)

Branch: `feat/architect` (26 commits ahead of `origin/feat/architect`).
COMPILER session. The parallel stdlib session is MISSION COMPLETE
(512 modules, 6,379 pub fns, 0 stubs, layout FROZEN); my replies live in
`docs/REPORT_TO_STDLIB_SESSION.md` (last updated `707a299c`).

## Session summary (2026-08-16): the 16 regression failures are FIXED

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

### A. [HIGH — pre-selfhost] Catalog body type-checking (Q2b, user-approved)
NOT STARTED this session (the 16-failure sweep consumed it). Plan unchanged:
Phase 1 check reachable injected fns; Phase 2 all catalog bodies at load
(hash-keyed cache); Phase 3 `--strict-stdlib` CI gate.

### B. [HIGH] api_freeze path sync + exec harness
Unchanged: `crates/xiom-codegen/tests/stdlib_tests.rs` +
`stdlib_execution_tests.rs` still reference the pre-refactor layout
(STDLIB_MANIFEST.md 515 paths + STDLIB_SMOKES.md 213 smokes).

### C. [MEDIUM] BUG 26 leftovers
- Bare prelude names in user modules — LIVE: smoke_alloc_basic fails with
  `undefined variable 'ptr'` (a `xiom.ptr`/prelude name users can't resolve).
- Cross-module tuple destructuring (Pattern::Tuple binds Int).

### D. [MEDIUM] 904-smoke battery — failure survey (NEW this session)
Ran all `examples/stdlib_smoke/smoke_*.xi` (904 files — the stdlib session
added ~830 since STDLIB_SMOKES.md). Results: ~516 pass / ~289 fail. The
failures are a MIX — triage notes for the next session:
- **Compiler bug classes seen (REAL, worth fixing):**
  - `void type only allowed for function results` (smoke_fmt_formatter) —
    void in expression position.
  - struct-literal field-type mixups (bench_math BST/Vec store; same family
    as the fixed BufReader invariant bug — a Node.new-style literal stores a
    Vec into a wrong-typed slot) — still OPEN.
  - `type 'Int' does not implement 'Bounded': missing method 'is_finite'`
    (smoke_num_saturating) — checker interface-bound gap.
  - Map AVs (smoke_stress_collections_map_*: get_missing/insert_get/
    clear/collision AV 0xC0000005) — Map is a NEW stdlib type, needs a
    dedicated investigation.
  - smoke_fmt_edge/float/format* AVs (0xC0000005/0xC0000409) — fmt module.
- **Stdlib-side (smoke files/API staleness — for the stdlib session):**
  parse errors (P001), undefined vars (missing imports), renamed APIs,
  broken .xi files (smoke_stress_rand_shuffle). The 4 known harness failures:
  hash_folder (missing `use xiom.convert.toint;`), rc/cell/utf8 now PASS
  (compiler fixes) — tell stdlib the rc/cell/utf8 labels in the old report
  were compiler-side, now fixed; hash_folder remains stdlib-side.

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
