# XIOM — Session Handoff: v0.27.0 "ARC-A-ptr-refs"

**Date:** 2026-07-09
**Branch:** `feat/guardian` (Phase 2 — compiler↔stdlib gap closure)
**Status:** stdlib execution **25/36 strict + 4 ignored**. Tier 1 done. ARC A+B landed. All gates green.
**Tag:** `v0.27.0-arc-A-ptr-refs`
**Companion session:** `SESSION_ECOSYSTEM.md` — AI-driven ecosystem build (other machine)

---

## HEADLINE ACHIEVEMENT — 0 → 25/36 stdlib modules link + run correctly

The compiler now faithfully lowers the stdlib through LLVM, with zero regressions to the
410-test gate throughout two major architectural features:
- **ARC A — Real pointer/ref types** (`*T`/`*mut T` → LLVM pointers, `&mut Scalar` → pointer params, deref read/write, address-of via `from_ref`/`from_mut`, call-site `&x`)
- **ARC B — Generic-type method monomorphization** (pub generic methods on `Cell`/`Rc`/etc emit real specialized bodies)

### Strict-passing (25)
alloc, char, cmp, collections, compress, contracts, convert, core, encoding, error, ffi, fmt,
log, math, num, ptr, reflect, simd, string, os, time, env, bench, rand, net

### Ignored but passing
thread, async, io, test

### Regression gate (ALL GREEN)
| Suite | Count |
|---|---|
| diff_tests | 25 ✅ |
| e2e_tests | **75** ✅ (was 66) |
| feature_regression | **48** ✅ (was 39) |
| full_diff | 23 ✅ |
| fuzz | 23 + 1 ignored ✅ |
| integration | 119 ✅ |
| robustness | 29 ✅ |
| check | **74** ✅ (was 72) |
| parser | 47 ✅ |
| stdlib_execution | 25 strict + 4 ignored |

---

## ARC A — Real Pointer/Ref Types (production-grade, design doc in `docs/ARC_A_POINTERS.md`)

### What it does
- `type_from_ast(Type::Ptr(inner))` → `"*inner"` pointer-encoded name
- `type_from_ast(Type::MutRef(inner))` for scalar inners → `"*inner"` (by-ref)
- `llvm_type_for("*T")` → `<T>*` (real LLVM pointer)
- `compile_expr` `UnaryOp::Deref` → `load` through pointer
- `Stmt::Assign` on `*p` → `store` through pointer
- `ptr.from_ref(x)`/`from_mut(x)` → returns lvalue address (alloca/GEP) or forwards existing pointer
- `&x` at scalar-ref call sites → passes address-of-x slot
- Param binding for `*T`/`&mut Scalar` params → pointer-typed local
- Registration: pointer params typed correctly; call-arg coercion matches

### Modules advanced
- `serialize`: was `clang: 'i64' but expected 'ptr'` → now compiles+links (runtime crash is separate)
- `sync`: was `clang: 'i64' but expected 'ptr'` → now compiles+links (invariant exit 1 remains)
- 6 modules benefit from real pointer typing

### E2E hardening
- `ref_mut_param.xi` — `fn inc(p: &mut Int){ *p = *p + 1 }` round-trip
- `ptr_deref.xi` — raw pointer deref read/write, caller mutation visible

---

## ARC B — Generic-Type Method Monomorphization

### What it does
- Checker (`xiom-check`) injects methods on **pub** generic types (Cell, Rc, etc.) via `collect_pub_generic_types`
- Codegen detects `recv_is_pub_generic` and emits specialized bodies
- `Cell.get[T](self)` → specialized `Cell.get(self: %struct.Cell) -> i64` body with GEP+load field 0
- `Cell.new[T](value)` → specialized body returning `%struct.Cell`
- Bare `value` → resolves to `self.value` inside method bodies

### Modules advanced
- `cell`/`rc`: methods emit real bodies (was stubs `ret i64 0`)
- Remaining: `set` body deref-field-write (`(*raw).value = v`) not emitted; self-by-pointer for mutation (TAIL-TODO)

---

## Key Infrastructure Fixes

### Core module (now strict-passing)
- `is_sorted`/`contains` made `pub` in `core.xi`
- Runtime intrinsics added to `xiom_runtime.c`: `xiom_is_sorted`, `xiom_contains`, `xiom_all`, `xiom_none`
- `Expr::Array` materializes i64 buffer (alloca + store elements at data[1..N], count at data[0])
- Contract-collection receiver guard: module-names not treated as value receivers

### sync GEP fix
- Excluded pointer-to-struct types from struct equality handler (`lt.starts_with("%struct.") && !lt.ends_with('*')`)
- Auto-deref pointer operands in `Lt`/`Gt`/`Le`/`Ge` comparisons

### Ecosystem gaps
- 13 of 14 `ecosystem/COMPILER_GAPS.md` gaps verified CLOSED (only GAP-13 brace-module won't-fix)
- GAP-3 (`pub const` resolution) fixed: checker `global_consts` map + pre-pass registration
- `feature_regression_tests.rs`: 9 ecosystem-gap lock-in tests (gap1/2/5/6/8/9/10/11/12)
- Checker `test_gap3_*` forward-reference tests

### Other
- clang `.obj` race fix: per-invocation unique temp CWD for clang intermediates (abs paths for all inputs)
- checker builtin methods: `len`/`unwrap`/`is_some`/`get`/`clone`/`to_str` on Str/Vec/Slice/Option/Result/wrappers
- module-level mutable `var` globals: `ConstDecl.is_mut` → LLVM `@global` emit
- `xiom-fmt` + `xiom-doc` build fixes (missing `UnaryOp`/`BinOp` arms)
- Full workspace `cargo build` compiles clean

---

## REMAINING (12 modules — see `docs/CODEGEN_TIER2.md` for full plan)

### Cluster 1 — Trait-bounded generic fn not monomorphized (ARC B follow-on)
**Modules:** `hash`, `iter`
**Root:** `hash.hash[T: Hash](value: T) -> UInt64` is generic with trait bound. Monomorphizer may skip
trait-bounded fns. The call resolves to a non-existent `@hash.hash`, which `emit_undefined_symbol_stubs`
replaces with `ret i64 0` → `hash.hash(42)` constant-folds to 0 → `hash.hash(42) == hash.hash(42)`
becomes `0 == 0` → but `hash.hash(true) != hash.hash(false)` also evaluates to `0 != 0` → false → exit 1.
Also: `hash.DefaultHasher.new()` may return a stub/zeroinitializer struct if not monomorphized per
concrete DefaultHasher.

### Cluster 2 — Deref-field-write / set body (ARC B TAIL-TODO)
**Modules:** `cell`, `rc`
**Root:** `Cell.set[T](self, value: T)` body `(*raw).value = value` — the `Stmt::Assign` for
`(*ptr).field = v` doesn't emit a store through pointer. Also, `self` passed by value → mutation
doesn't persist. Need: store-through-pointer for deref-field-write; self-by-pointer ABI for generic
methods that mutate.

### Cluster 3 — Const-generic `[N]T` arrays
**Modules:** `array`
**Root:** `array.len[T; N](arr: &[T; N]) -> Int` where `N` is a const generic. The const-generic
parameter isn't propagated → `len()` returns 0. Also: `Expr::Array` type must carry element count.

### Cluster 4 — Runtime crashes (deeper)
**Modules:** `mem` (stack overflow), `path`, `crypto`, `regex` (illegal instruction/SIGILL)
**Likely roots:** `size_of[T]()` returning 0; `Layout.new` resolving through wrong module path;
`xiom_str_*` runtime calls mis-wired. Need per-module crash investigation.

### Cluster 5 — Safety gate bypass
**Location:** `crates/xiomc/src/main.rs:208` — "continuing to codegen despite type errors"
**Action:** Remove once stdlib type-checks cleanly with zero `T001` errors. Enforces "compiles ⇒ safe."

---

## HANDOFF PROMPT (paste into next session)

```
Continue the stdlib execution work from SESSION.md (tag v0.27.0).
Branch: feat/guardian. All regression gates are green (diff25, e2e75, feature_regression48,
full_diff23, fuzz23, integration119, robustness29, check74, parser47). 25/36 strict modules pass.

Start with Cluster 1 (hash/iter trait-bounded generic fn emission) — inspect the
emit_undefined_symbol_stubs + monomorphization to fix @hash.hash not being emitted. Then
Cluster 2 (cell.set/rc.set deref-field-write) — the Stmt::Assign path for (*ptr).field = v.
Then Cluster 3 (array const-generic N propagation).

After each fix, run the full gate:
  cargo build -p xiomc
  cargo test -p xiom-codegen 2>&1 | Select-String "test result:"
  cargo test -p xiom-check 2>&1 | Select-String "test result:"
  cargo test -p xiom-parser 2>&1 | Select-String "test result:"
  cargo test -p xiom-codegen --test stdlib_execution_tests 2>&1 | Select-String "test result:"

Gate must stay green. Write e2e hardening tests for any pattern found working.
Update SESSION.md and CODEGEN_TIER2.md with progress.

Key files:
- codegen: crates/xiom-codegen/src/lib.rs
- checker: crates/xiom-check/src/lib.rs
- runtime: stdlib/runtime/xiom_runtime.c
- stdlib: stdlib/xiom/*.xi
- smoke: examples/stdlib_smoke/smoke_*.xi
- e2e tests: crates/xiom-codegen/tests/e2e_tests.rs (+ examples/e2e/*.xi)
- feature regression: crates/xiom-codegen/tests/feature_regression_tests.rs
- tier 2 plan: docs/CODEGEN_TIER2.md
- arc A design: docs/ARC_A_POINTERS.md
- ecosystem gaps: ecosystem/COMPILER_GAPS.md (13/14 closed)
```
