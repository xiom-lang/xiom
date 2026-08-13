# XIOM Compiler Session — Handoff (2026-08-13, evening)

Branch: `feat/architect`. All work committed. Parallel stdlib session: MISSION
COMPLETE (512 stdlib modules, 6,379 pub fns, 0 stubs — commits 830f705a +
9aa95d35). Their open items for the compiler session are listed at the end.

## This session's compiler work (all verified + committed)

### 1. BUG 25 #10 — xiom.crypto fully RESOLVED (commit 00154465..ca55a8fb)
The crypto module was unimportable (`use of undefined value '@_pkcs7_pad'`).
A CHAIN of 9 defects, each reproduced minimally and fixed:

| # | Defect | Fix location |
|---|--------|--------------|
| 1 | Unsupported `[0; 16]` literal broke aes_encrypt's parse → body leaked to module scope → `_pkcs7_pad` ctor stub | stdlib/xiom/crypto/crypto.xi |
| 2 | `&fixed_arr[i] as *UInt8` lowered to value-load + inttoptr → NULL ptr (AES-NI 0xC0000005) | codegen expr.rs (Ref arm: fixed-array element GEP) |
| 3 | Checker rejected `&x as *T` in user modules | checker (reference-to-pointer cast rule, unsafe-gated) |
| 4 | Match-bound Vec payloads bound as i64 handles → `&v` passed handle-SLOT address | codegen stmt.rs (re-materialize %struct.Vec) |
| 5 | Private structs used as LOCALS in catalog fn bodies degraded to i64 | checker (body struct-name scan → transitive type walk) |
| 6 | Const fixed-array reads returned the LENGTH slot (`_AES_SBOX[i]` garbage) | codegen expr.rs (const arrays join array-buffer path) |
| 7 | aes.xi `aes_add_round_key` transposed key mapping (self-consistent but non-FIPS) | stdlib/xiom/crypto/aes.xi |
| 8 | AES-NI C decrypt missing aesimc on middle round keys | stdlib/runtime/xiom_runtime.c |
| 9 | `requires` contracts on graceful-fallback fns trapped bad-key calls | crypto.xi (contracts removed per BUG 22 #5 rule) |

Verified: FIPS-197 Appendix B (AES-128) + C.2 (AES-192) EXACT match through
both implementations; AES-NI hardware encrypt + software decrypt roundtrip;
all 25 crypto smokes green at the time. 3 new regression tests:
`m37_const_array`, `m37_payload_ref`, `m37_ptr_cast` (+ `mk_optvec` in
m37_catmod). 12 M22-era crypto smokes repaired (stray braces, `.get`→indexing,
RSA key range, GCM tuple-pattern→`.0`/`.1`).

### 2. BUG 27 (compiler side) — sublib prefixes + generic-ctor injection (commit 4e95717e)
See docs/COMPILER_BUGS.md for the full write-up. Highlights:
- `use xiom.os; os.platform.platform_name()` now resolves (lazy catalog-peek
  descent; collision-safe submodule_aliases; checker-only local_module_paths).
- `Map[Str, Bool].new()` in module-global inits emits real bodies (base-type
  receiver inference, prelude-loads xiom.collections, ginit drain pass).
- CAREFUL: the eager submodule augmentation was tried and REVERTED — it
  perturbed bare-alias keep-first resolution (crypto sha256 broke). The lazy
  peek design is the one that stays.

### 3. Security review (user-approved, Q2) — commits pending (1aa93cc?)
- Release builds strip `assert`/`dbg!`/`debugger;`; `--keep-debug-checks`
  retains. dbg! keeps its value semantics. Verified in all three modes.
- Const-eval budget: CONST_EVAL_BUDGET=4096 depth guard in evaluate_const_init.
- asm(): verified already unsafe-gated; no stdlib asm usage.
- `--enable-unsafe-direct` prints a prominent warning every invocation.
- OPEN (multi-session): catalog fn bodies bypass the checker (Q2b); see
  ROADMAP items below.

### 4. Q1 — docs/AI_CONTEXT.md updated (user-approved override of IMMUTABLE)
v0.58.0: numeric policy (Int↔Float explicit `as`; int literals adopt float;
same-family widening auto), labeled loops (`@label: while` / `break @label;`),
debug intrinsics (assert/dbg!/todo!/unimplemented!/debugger; + release
stripping), `else if` desugared-form reconciliation, sublib-prefix resolution,
new CLI flags (--keep-debug-checks, --release notes, --enable-unsafe-direct
warning).

## Current verification state (isolated binary, `$env:TEMP\kilo\tgt_iso`)
- 34/34 regression sweep; checker 178/178; workspace zero warnings.
- Crypto 29/30 — ONLY `smoke_stress_crypto_aes_gcm` fails (0xC0000005).
  **REPRODUCED AT BASELINE (my crates stashed)** — it is the parallel stdlib
  session's in-flight "tuple+Vec heap corruption" (BUG 27 #12), NOT compiler.
  Do not chase it while they own the stdlib; re-test after their next wave.
- os.platform sublib + contracts/Map.new global-init + release-strip probes R=0.

## Open items (parallel session's list + mine) — see docs/ROADMAP.md
1. Q2b (approved, LARGE): type-check catalog module fn bodies (the checker
   gap that let `&ct_buf[0] as *UInt8` compile silently in crypto.xi). Phased
   plan in docs/ROADMAP.md. Do NOT rush — multi-session.
2. BUG 27 leftovers: Error reserved type, module-scope fn storage read-only,
   unsafe Int returns, Option[Vec] payloads (match-bound shape FIXED today;
   var-bound/unwrap shapes may remain), flat crypto sha512/md5/aes defects
   (stdlib files, theirs), high-bit mask AND (convert/utf8.xi, BUG 26 #5).
3. BUG 26 items: catalog-returned-Vec→&Vec C001 — VERIFIED GONE (lz4 chain
   compiles; runtime decompress is their in-flight file); bare prelude names
   in user modules; cross-module tuple destructuring (Pattern::Tuple binds Int
   — documented workaround `.0`/`.1` field access; full payload-aware binding
   is a roadmap item).
4. api_freeze path list + ~200 smokes for the exec harness (stdlib_tests.rs +
   stdlib_execution_tests.rs path sync) — stdlib layout is now FROZEN
   (9aa95d35 "MISSION COMPLETE"), so this backlog item is unblocked.
5. Vec.get dispatch hijack ("expected Box, found Int") — `.get` is not a Vec
   builtin; the checker resolves it to a Box-typed get. Roadmap: proper
   container-method dispatch or explicit T001 for unknown methods.
6. `[v; n]` array-repeat literal — NOT spec; crypto.xi used it. Language
   feature candidate (spec change requires user approval).
7. gcm smoke re-test after the stdlib session fixes tuple+Vec corruption.

## Workflow notes
- Parallel stdlib session is DONE (their words). If a new session starts,
  verify with the isolated binary; never stash/revert uncommitted stdlib work.
- The ginit drain + prelude-collections + lazy-peek are load-bearing: do not
  "simplify" them without re-running the crypto battery + sweep.
