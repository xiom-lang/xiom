# XIOM Session Handoff — 2026-08-06 22:10

## ⚠️ CRITICAL CONSTRAINTS
1. **NEVER commit/modify `xiom-benchmark-chaos/`** — the owner is working in a PARALLEL session on it. It has uncommitted changes (Dockerfile, dashboard TSX, routes.js, config.yaml, reference/contracts, .dockerignore, stdlib-pin/) — LEAVE THEM ALONE. Only stage/commit files under `crates/`, `stdlib/`, `docs/`, `tests/`, `examples/`, `selfhost/` (except the `_diff_*` temp files).
2. **Parallel session rebuilds `target/debug/xiom.exe` frequently** — test-suite runs may fail with stale/half-written binaries. Verify failures by recompiling the specific smoke manually before assuming a regression.
3. **Selfhost tests are IGNORED until the selfhost phase** (user directive):
   - `test_selfhost_bootstrap_v050` (diff_tests.rs) — `#[ignore]`
   - All `full_diff_tests.rs` (23 tests) — `#[ignore]`
   - Reason: `xiomc_v050.xi` embeds a 31982-if `source_at` chain that exceeds the compiler's 300s compile timeout.

## CURRENT BRANCH
`feat/architect` — HEAD `5f926b7b` + the commit below

## ✅ THE 6 E2E BUGS ARE FIXED (this session's completion)

All 6 originally-failing E2E tests pass:
- `e2e_m21_borrow_004/008/010` — `*r` deref of `&Int` param loads i64 (not i8)
- `e2e_m33_b14` — generic `&T` deref
- `eco_algo_89_tests` — Vec[Str]/Vec[Int] + generic fns + `&Vec[T]` by-value
- `e2e_safety_probe` — `@null` undefined + `env.args()` + transitive imports

## THE FINAL FIX (root cause + solution)

**Root cause**: The module-wrapper preservation in `collect_external_decls` produced
`TopDecl::Module` decls that the driver merge (`crates/xiom/src/lib.rs:849`,
`_ => continue`) **DROPS entirely** → injected stdlib decls never reached codegen →
`array.len` became an auto-stub, `array.contains` hit the `@xiom_contains` contract
builtin (has_user_fn=false) → clang `%tmp44 defined with type i64 but expected ptr`.

**Solution (SESSION.md's Option A, refined)** — flat injection with leaf-qualified names:
1. `crates/xiom-check/src/lib.rs` — `collect_external_decls`:
   - Reverted the module-wrapper preservation → recurse FLAT (driver only accepts flat `TopDecl::Fn`).
   - FREE fns are leaf-qualified at injection: `array.contains`, `env.args`, `io.println`
     (methods keep receiver keys). Module context now rides on the name itself.
   - **User-shadow guard**: bare free-fn names declared in the USER program (incl. nested
     modules) block injection of same-leaf stdlib fns (`module sys { fn alloc }` blocks
     xiom.alloc's `alloc` — previously a duplicate `@alloc` definition, m35_z06).
   - Reachability filter matches LEAF names (`array.contains` matched by referenced `contains`).
2. `crates/xiom-codegen/src/context.rs` — new `MonoContext::bare_fn_aliases` map
   (bare `args` → qualified `env.args`).
3. `crates/xiom-codegen/src/decl.rs` — `register_functions` populates the alias map
   keep-first for leaf-qualified free fns (user bare fns registered earlier win).
4. `crates/xiom-codegen/src/call.rs` — bare internal stdlib calls (`args()` inside
   env.args_os) are rewritten through the alias so the emitted symbol matches the
   leaf-qualified definition. Removed FK2/FK_PATH debug prints.
5. `crates/xiom-codegen/src/coerce.rs` — **by-value struct param branch restored,
   STRUCT-ONLY**: `&Vec[T]`/`&Slice[T]` params receive the struct VALUE; `&mut Vec2`
   (pointer) and scalar `&T` (address-as-i64) are excluded. This fixed eco_algo's
   `binary_search(&arr, …)` (was passing the Vec data pointer → garbage struct → AV)
   without regressing m33_b18 (`&mut` struct must get the slot address).
6. `crates/xiom-codegen/src/lib.rs` — mono body param loop mirrors compile_fn's
   `param_locals`/`ref_params` tracking (generic `array.contains` body's `arr[i].eq(x)`
   needs the `&T` param deref). `main` argc/argv seeding is now **native-only**
   (wasm has no xiom_set_args runtime link — fixed 5 wasm E2E tests).
7. `crates/xiom-codegen/src/expr.rs` — unchanged this session (prior fixes stand).

## TEST SUITE STATE (verified with a stable build of this session's work)
| Suite | Result |
|-------|--------|
| checker | 156/156 |
| parser | 96/96 |
| feature-reg | 510/510 |
| integration | 128/128 |
| stdlib-compile | 40/40 |
| stdlib-exec | 41/41 |
| E2E (full) | 2231 pass; **`e2e_spawn_capture` is FLAKY** (see below) |
| diff | 24/24 + 1 ignored (selfhost) |
| full-diff | 23 ignored (selfhost phase) |

### ⚠️ KNOWN FLAKE (NOT this session's regression — verified failing 5/5 at HEAD too)
`e2e_spawn_capture` (spawn_capture.xi): ACCESS_VIOLATION ~50-100% of harness runs,
0/8 when run interactively. Root cause: `spawn move` threads are detached
(fire-and-forget in stdlib/runtime/xiom_runtime.c `xiom_thread_spawn`); when `main`
returns while the spawned thread is still writing via `io.println`, process teardown
races the CRT stdio lock → AV with piped stdout. Fixing it requires a runtime change
in `stdlib/runtime/xiom_runtime.c` (detach → join-on-exit) which is OUTSIDE the
allowed commit scope for this session.

## COMMIT (this session)
`git add crates/xiom-check/src/lib.rs crates/xiom-codegen/src/call.rs crates/xiom-codegen/src/coerce.rs crates/xiom-codegen/src/context.rs crates/xiom-codegen/src/decl.rs crates/xiom-codegen/src/expr.rs crates/xiom-codegen/src/lib.rs`
Message: "fix(codegen): &T ref-param deref widths, main argc/argv seeding, transitive stdlib imports, generic fn leaf-key disambiguation"
DO NOT add xiom-benchmark-chaos/ or selfhost/_diff_* files or .xiom_ai.json.

## DEBUG PRINTS
- FK2 / FK_PATH / RMC_C: **removed**.
- `crates/xiom-codegen/src/types.rs` CG02 DEBUG prints: PRE-EXISTING (committed in
  a98cdd70, always-fire in global_const_init). NOT removed — types.rs is not in the
  allowed commit list. If noise matters, remove them in a future session.

## IMPORTANT SEMANTIC FINDINGS (still valid)
- `&T` params carry the ADDRESS as i64 (NOT the value). `*r` must inttoptr+load.
- `&mut T` / `*T` are real pointers (i64*).
- `&Vec[T]`/`&Slice[T]` params are the Vec STRUCT by value (%struct.Vec).
- `&[N]T` fixed-array params are the Vec DATA pointer (i64*).
- Injected stdlib free fns are leaf-qualified (`array.contains`); bare internal calls
  resolve via `MonoContext::bare_fn_aliases` (keep-first).
- User program free fns SHADOW same-leaf stdlib fns at injection time.

## EARLIER COMMITS THIS SESSION (already in feat/architect)
- `5f926b7b` docs: RELEASE_PROCESS — E2E chaos/i2/safety internal fixtures
- `5ad7e439` fix(codegen): deref by-value &T params + decl-based generic call arg inference (PARTIALLY REVERTED in this session's work — the by-value no-op was wrong for scalars; struct-only by-value is correct)
- `edd8317a` fix(codegen): &[N]T array refs pass Vec data pointer; &T by-value ABI args coerce correctly
- `e78c4564` test: ignore selfhost-dependent tests
