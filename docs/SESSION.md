# XIOM Compiler Session — Handoff (2026-08-13, EOD)

Branch: `feat/architect`. Working tree: my changes all committed; the only
uncommitted files are the parallel stdlib session's in-flight edits
(`crates/xiom/src/lib.rs` whitespace, `examples/stdlib_smoke/smoke_net_address.xi`,
their untracked smokes) — DO NOT touch or commit them without asking.

**Parallel stdlib session: MISSION COMPLETE** (830f705a + 9aa95d35: 512
modules, 6,379 pub fns, 0 stubs, stdlib_tests 40/40, layout FROZEN).
My reply to their report: `docs/REPORT_TO_STDLIB_SESSION.md`.

## Committed this session (feat/architect)

```
1a7830a3 feat(security): release debug stripping, const-eval budget, unsafe-direct banner (Q2)
4e95717e fix(check/codegen): sublib-prefix resolution + generic-ctor injection (BUG 27)
ca55a8fb docs(compiler-bugs): BUG 25 #10 RESOLVED — full defect chain + verification
98e6ec79 test(regression): lock BUG 25 #10 fixes (const arrays, payload refs, ptr casts)
d067c8ab test(stdlib): fix 12 crypto stress smokes (M22-era, never compiled)
f372858b fix(stdlib): crypto AES correctness (BUG 25 #10 chain)
00154465 fix(codegen/check): crypto FFI + catalog boundary fixes (BUG 25 #10 chain)
```

## Current verification state (isolated binary: `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- **34/34 regression sweep** (m37_* + m33/m34/m19; 3 new tests: m37_const_array,
  m37_payload_ref, m37_ptr_cast).
- **Checker 178/178; workspace zero warnings.**
- **Crypto 29/30** — the ONLY failure is `smoke_stress_crypto_aes_gcm`
  (0xC0000005), **REPRODUCED AT BASELINE with all my crates stashed** → the
  parallel session's in-flight "tuple+Vec heap corruption" (BUG 27 #12), NOT
  the compiler. Re-test after their fix; do not chase while they own the stdlib.
- Probes R=0: os.platform sublib (`use xiom.os; os.platform.platform_name()`),
  contracts import (Map.new global init), Map.new in user globals, lz4 chain
  (compiles; runtime is their file), release-strip (all three modes).
- FIPS-197 AES-128 (Appendix B) + AES-192 (C.2) EXACT match; AES-NI roundtrip.

## Load-bearing fixes — do NOT "simplify" without re-running the full battery

1. **Lazy catalog-peek submodule descent** (checker): directory-module
   submodule segments resolve via `catalog.peek_owned()` (parse WITHOUT
   caching → no injection-set perturbation). The EAGER augmentation was tried
   and REVERTED — it broke crypto sha256 via bare-alias keep-first.
2. **Prelude force-loads xiom.collections** (checker) — container generic
   decls must reach the monomorphisation registry (Map.new in globals).
   Planned refinement: catalog reverse type-index (ROADMAP E).
3. **ginit drain** (codegen): second `compile_generic_monomorphisations()`
   after the @llvm.global_ctors emission — generic ctors in global inits.
4. **infer_struct_type_name base-type Index receivers** (codegen) — fn_key
   `Map.new` not bare-key hijack.
5. **Release stripping**: assert/dbg!/debugger; stripped in `--release`;
   `--keep-debug-checks` retains; dbg! keeps value semantics. Contracts were
   already release-stripped (`--runtime-contracts` forces).
6. **Const-eval budget**: CONST_EVAL_BUDGET=4096 depth in evaluate_const_init
   (bail → runtime evaluation). asm() verified unsafe-gated (T001).
   `--enable-unsafe-direct` banners every invocation.

## OPEN items (next sessions, in priority order)

### A. [HIGH — pre-selfhost] Catalog body type-checking (Q2b, user-approved)
Stdlib fn bodies are never type-checked — invalid casts compile silently
(crypto's `&ct_buf[0] as *UInt8` became UB). Phased plan in ROADMAP.md item A:
Phase 1 check reachable injected fns (report as user-program errors; measure
stdlib fallout); Phase 2 check all catalog bodies at load, hash-keyed cache;
Phase 3 `--strict-stdlib` CI gate. Multi-session; do not rush.

### B. [HIGH] api_freeze path sync + exec harness (unblocked — layout frozen)
`crates/xiom-codegen/tests/stdlib_tests.rs` + `stdlib_execution_tests.rs` still
reference the pre-refactor layout. The stdlib session owes the api_freeze path
list + ~200 smokes (see REPORT_TO_STDLIB_SESSION.md). Wire the harness once
they send it.

### C. [MEDIUM] Repro-driven fixes (waiting on stdlib session repros)
Error reserved type · module-scope fn storage read-only · unsafe Int returns ·
Option[Vec] via unwrap/var-bound (match-bound shape is FIXED). All simple
forms pass — need their exact snippets.

### D. [MEDIUM] BUG 26 leftovers
- Bare prelude names in user modules (checker resolution).
- Cross-module tuple destructuring: Pattern::Tuple binds Int; workaround
  `.0`/`.1` documented. Full payload-aware binding: ROADMAP B.

### E. [MEDIUM] MCP catalog/registry tools (user question — planned, ROADMAP G)
- **EXISTS already**: `xiom_stdlib_reference` MCP tool — live stdlib module
  list + per-module public API (signatures/contracts/types) parsed from
  source. This answers "can an agent inspect a loaded module's functions?"
  YES — via the checker/catalog live parse.
- **PLANNED**: (1) `xiom_module_introspect(module)` — checker-side export-map
  dump (pub fns + sigs + types) WITHOUT a full compile, for modules loaded in
  the current compile session; (2) `registry_search(query)` /
  `package_info(name)` — queries the package registry (ghcr.io/xiom-lang
  after the repo split) so agents can discover PACKAGES, not just stdlib.
  Implement after the split (registry is testable then — see below).

### F. [LOW] Roadmap E/F items
Reverse type-index (compile-time economy), contract policy decision.

## Sequencing per the user (pre-selfhost → split → registry)

1. **Selfhost gate**: stdlib 100% (their side) + compiler 100% (sweep,
   checker, stdlib_tests, e2e, formatter) + zero warnings + item A (catalog
   body checking) + item B (exec harness) before selfhost is declared done.
2. **Monorepo split** per `docs/REPO_SPLIT.md` (xiom-lang: xiom/stdlib/
   packages/website/playground/registry; xiom-foundation: pulse/benchmark/
   db/vector/debugger). Do the split ONLY after (1) is green — the split
   changes paths, so the frozen-layout harness (item B) must land first.
3. **Registry + packages**: after the split, test publish/install against
   ghcr.io/xiom-lang for real — then the MCP registry tools (item E) become
   implementable and testable end-to-end.

## How to verify anything you touch

- Rebuild the isolated binary: `$env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso";
  cargo build -p xiom` then use `tgt_iso\debug\xiom.exe` (NEVER the shared
  target/debug binary — the parallel session rebuilds it constantly).
- 34-test sweep list is in this doc's history; run it + `cargo test -p
  xiom-check` + the crypto battery (`Get-ChildItem examples\stdlib_smoke
  -Filter 'smoke*crypto*.xi'`) after any change.
- Never stash/revert uncommitted stdlib work without a fresh backup.
