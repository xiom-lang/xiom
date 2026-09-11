# LET-array Representation -- JOINT Decision (compiler <-> stdlib)

**Status:** DECIDED (compiler lane), input received from the stdlib lane (Q5
census, 2026-09-09). **Date:** 2026-09-11. **Owners:** compiler lane executes;
stdlib lane consumes. Cross-refs: `docs/COMPILER_READINESS_PLAN.md` Stage 4,
`docs/SESSION.md` Round-39 queue, `docs/stdlib_session.md` Q5,
`docs/REPORT_TO_COMPILER_SESSION.md` section 2.

## The question

What does a `let`-bound array literal bind to?

- **Fixed array `[N]T`** -- the same representation `var` uses and the one all
  `xiom.array` module functions take (`&[N]T`).
- **`Vec[T]`** -- the legacy M33 conversion the compiler applies to
  let-bound literals so the `xiom.core.slice`/collections API (`&Slice[T]`)
  accepts them.

Mixed representation in one language position is the conflict: `let a =
[1,2,3];` was silently a Vec while `var a = [1,2,3];` is a fixed array, so
`array.len(&a)` and `slice.len(&a)` depended on which conversion ran.

## Census (stdlib lane, Q5 -- authoritative)

- stdlib + examples contain **ZERO let-bound fixed arrays** (0 sites).
- All **69 fixed arrays** are `var` FFI staging buffers (net/socket/websocket/
  io/crypto/buffer/pipe/hash), which depend on address stability for
  `&buf[0]` extern calls.
- **85** `[N]T` / `&[N]T` parameters exist across stdlib modules.
- Consequence: changing let-array semantics has **no stdlib/examples surface
  today**; the `var [N]T` staging class is the only representation-sensitive
  workload, and it already uses fixed arrays.

## Decision

**`let`-bound array literals bind FIXED ARRAYS `[N]T`, the same as `var`.**

Rationale:

1. One array representation: type inspection (`a: [3]Int`), indexing,
   `.len()`, `&a` ABI, and `array.*` functions all agree.
2. Annotation and inference agree: `let c: [3]Int = [7,8,9];` is the same
   representation as an unannotated literal, so the typed form stops needing
   special-casing.
3. Immutability is a *binding* property (`let` vs `var`), not a
   representation property; using Vec for `let` conflated the two.
4. The M33 let->Vec conversion was a stopgap for `&Slice[T]` consumers; the
   stdlib census shows those consumers do not need it for let literals, and
   `xiom.array`'s `&[N]T` API is the production path.
5. Address stability: `let` arrays get a stack slot exactly like `var`; FFI
   staging stays legitimate for both.

## Current behavior at HEAD (probed 2026-09-11, canonical binary)

| Shape | Result |
|---|---|
| `let a = [1,2,3]; a[0]` + `array.len/first/get(&a)` | WORKS |
| `var b = [4,5,6]; b[1] = 50;` | WORKS |
| `var b = [...]; user_fn(&b)` with `fn user_fn(a: &[3]Int)` | COMPILE FAIL: invalid IR (`expected '(' in call`) |
| `let c: [3]Int = [7,8,9];` | COMPILE FAIL: invalid IR (`%tmp defined with type i64 but expected [3 x i64]`) |

Probes: `tmp/bug_probes/letarr1.xi` (annotated let),
`tmp/bug_probes/letarr2.xi` (user-fn `&[N]T` arg), `letarr_b.xi` (working
catalog API path). Both failures reproduce identically on the pre-Stage-2c
binary -- they are representation gaps, not regressions.

## Migration plan (compiler lane)

- **P1 (annotated let):** `let c: [N]T = [literals]` lowers to a fixed-array
  slot whose LLVM type is `[N x T]`; the literal elements use the annotated
  element width. Lock: `e2e_let_array_annotated`.
- **P2 (user-fn `&[N]T` args):** `&arr` where `arr: [N]T` passed to a user
  (non-catalog) `&[N]T` parameter lowers to the element pointer plus the
  const-N registration the mono path already uses for catalog fns. Lock:
  `e2e_let_array_user_fn_ref`.
- **P3 (delete M33 let->Vec):** unannotated `let a = [...]` binds `[N]T`.
  `&a` to a `&Slice[T]` parameter materializes an explicit Slice view
  (`data = &a[0], len = N`) at the call site, preserving source
  compatibility; `array.as_slice(&a)` remains the explicit form.
  Lock: `e2e_let_array_slice_bridge`.
- Order: P1, P2 land independently; P3 last, gated on a stdlib re-sweep
  (zero let sites today, so expected fallout is limited to compiler
  regression fixtures).

## Stdlib-lane implications

- No action required now: keep `var [N]T` for FFI staging buffers; do not
  introduce `let` fixed arrays until P3 lands.
- After P3, `let` arrays are `[N]T`: use `array.*` directly; use
  `array.as_slice(&a)` (or the automatic call-site bridge) for Slice APIs.
- Open confirmation requested (non-blocking): (1) no planned let-array
  usage; (2) acceptable that `&[N]T` -> `&Slice[T]` coercion is automatic
  for argument positions.
