# XIOM — Session Handoff: v0.45.4 "Phase 5c.29–5c.30 — 101/101 E2E"

**Date:** 2026-07-17
**Branch:** `feat/architect`
**Status:** 47/47 parser, 74/74 checker, **101/101 e2e** (was 90/101), 36/41 stdlib-exec (was 30/41)
**Deterministic builds:** same IR ⇒ byte-identical binary (verified by SHA256)

---

## THE BIG PICTURE — What this session proved

The "manual pass / e2e fail" discrepancy was NOT a runner bug. Clang embeds the
input `.ll` path in the binary; different names shifted binary layout and a
LATENT memory-corruption bug (container-handle convention had readers but no
writers) manifested or hid depending on layout. Making builds deterministic
(5c.29) turned the heisenbug into a stable, debuggable crash — then the real
bug chain was fixed one root cause at a time.

---

## 5c.29 — Deterministic builds + container-handle convention

1. **Driver (`crates/xiomc/src/main.rs`)**: stage post-opt IR into the unique
   temp CWD under FIXED name `xiomc_input.ll`, pass RELATIVE path to clang,
   add `/Brepro`. Same source + different `-o` names ⇒ equal SHA256.
2. **Handle convention completed**: generic container fields (`Vec[T]`) are
   i64 HANDLES (5c.28h). Readers existed (Index inttoptr, val_to_struct
   memcpy) but NO writer ever produced a handle:
   - struct literals / field assignment now heap-box the header
     (`emit_box_struct_handle`) and store `ptrtoint`
   - `store_back_to_receiver` writes updated headers THROUGH the handle
   - all Vec builtin gates (push/pop/get/len/index/indexed-assign) accept
     handle receivers via `resolve_vec_receiver`
3. **Element widths**: `emit_elem_store/load` now use real 1/2/4/8-byte
   accesses (all non-8 widths were collapsed to 1 byte — destroyed
   Float32/Int32/Int16 elements). Float elements bit-reinterpret (raw-bits
   convention), never sitofp.
4. **Method ABI**: definitions no longer emit `%param_self` for ecosystem
   style methods (`fn T.method(h: &T, ...)`) — registration, call sites and
   definitions now agree (HTTP/SQLITE AV root cause: every arg was shifted).
5. **Inline `Vec.insert` / `Vec.remove`** (llvm.memmove, element-size
   agnostic) — generic stdlib dispatch misrouted container-field receivers
   to argument-less stubs (`@Map.insert()` called with 3 args).
6. **elif-without-else merge blocks**: stray `unreachable` before live code
   (HTTP from_str trap; also the FULL `while_accumulate` bug encoding).
7. Enum fixes: qualified variant patterns resolve discriminants
   (`SqliteValue.Integer(v)`), float payloads stored as raw bits (fptosi
   destroyed `Real(2.718)` → 2), match on unwrapped i64 enum payloads adopts
   the unique candidate enum, user-defined `Type.to_str` no longer hijacked.

## 5c.30 — Type-erasure recovery + payload flow tracking

- `local_vec_elem`: `var v = Vec[Point2D].new()` elem types per local
- `local_vec_handle`: match-arm container payload bindings
  (`JsonValue.Array(ref mut items)` — mutations alias the original enum)
- `local_boxed_struct` / `local_opt_payload`: pop/get/remove → unwrap flow
- `fn_return_xiom` + `type_string_full`: declared `Result[Vec[Int], Str]`
  return types survive erasure; unwrap bindings classified as handle or box
- `enum_variant_field_types`: per-variant payload types (type_meta dedups by
  name — JsonValue's `val` was Bool|Float64|Str|Vec[...])
- `struct_byte_size`: real layout size incl. nested by-value structs
  (JsonEntry = 24B, not fields×8 = 16B) at Vec.new/val_to_struct/boxing
- pop/get/remove box STRUCT payloads (`emit_elem_payload_load`)
- `&local.field` emits a real GEP (was: bare field name captured any local
  with that name — TFR test 7 bound `&addr3.ip` to local `ip`)
- this-based receivers from unwrap boxes: inttoptr directly to pointer
  receiver (loading through it read the discriminant as an address)

## Test-file corrections (encoded old miscompilations)

- `test_full.xi`: `safe_divide` had `requires: b != 0` while its tests pass
  b=0 expecting Err (contract trap fired first); `test_while_accumulate`
  expected 26 — the value only produced by the old elif-fallthrough bug
  (correct: 35); `agent_start` now restarts from Done/Failed (the run-cycle
  test has no reset transition and only "passed" under miscompiled enums).
- Fuzz/robustness harnesses compile on a 32MB stack thread (2MB test-thread
  default overflows on compile_expr debug frames).

---

## GATE STATUS

| Suite | Result | Notes |
|-------|--------|-------|
| e2e_tests | **101/101** ✅ | was 90/101 |
| parser / checker | 47/47, 74/74 ✅ | |
| stdlib_execution | 36/41 | 5 PRE-EXISTING: array, core, serialize, ptr, mem (checker: bare receiver-field refs) |
| feature_regression | 48/48 ✅ | |
| integration | 119/119 ✅ | |
| fuzz / robustness | 23 + 29 ✅ | big-stack harness |
| diff_tests | 24/25 | PRE-EXISTING: selfhost expects unqualified `call @compile_all` |
| full_diff | 23/23 ✅ | |
| stdlib_tests (module compile) | 2/39 | PRE-EXISTING checker gap (same errors on baseline f35a0cc) |

---

## KEY FILES

| File | Purpose |
|------|---------|
| `crates/xiom-codegen/src/lib.rs` | Main codegen (~10.1k lines) — all 5c.29/5c.30 fixes |
| `crates/xiomc/src/main.rs` | Deterministic staged-.ll clang invocation |
| `crates/xiom-codegen/tests/e2e_tests.rs` | E2E runner |
| `tests/ecosystem/test_*.xi` | Ecosystem tests |
| `docs/ROADMAP.md` | Updated gate table + 5c.29/5c.30 summary |

## DEBUGGER WORKFLOW (unchanged)

`C:\Users\lefte\AppData\Local\Microsoft\WindowsApps\cdbX64.exe`
Compile with `--debug`, script: `g` / `k 10` / `r rcx,rdx,r8` / `.exr -1` / `q`.
Diagnosis pattern used all session: per-test diagnostic mains that return the
1-based index of the first failing test (binary-search-free isolation).

---

## NEXT SESSION PRIORITIES

### P1 — stdlib module compilation (5 modules)
`array/core/serialize/ptr/mem` fail type-check: bare receiver-field
references (`len`, `cap`, `data`) inside generic `Vec.x[T]` methods are not
resolved by the checker. Fix in `xiom-check` (implicit-this field scope for
receiver-qualified generic fns) → unlocks stdlib_tests + the 5 exec tests.

### P2 — selfhost diff test
Emission is `call @codegen.compile_all` (module-qualified); the test expects
unqualified. Decide canonical policy (prefer qualified; update test).

### P3 — hardening depth
- untracked expression positions for local Vec[Float32] (payload typing map
  covers bindings, not arbitrary temporaries)
- enum-variant struct-literal path (`Expr::Struct` enum branch) does not
  heap-box container payloads yet (constructor path does)

## GIT LOG (this session)
```
88badd4 fix(codegen): 5c.30 Option/Result payload type tracking - CRYPTO 23/23, e2e 101/101
5494cd2 fix(tests): 5c.30 FULL - remove contradictory contract, correct elif expectation
10570e9 fix(codegen): 5c.30 enum payload conventions - JSON 29/29
ab3fcb0 fix(codegen): 5c.30 local Vec-of-struct element typing + boxed Option payloads (VOS)
c603de6 fix(codegen): 5c.30 &local.field emits real GEP
7cf7a5b fix(codegen): 5c.29 container-handle convention + 9 production fixes (90->96 e2e)
ab588e2 fix(xiomc): 5c.29 deterministic builds - fixed staged .ll name + /Brepro
```
