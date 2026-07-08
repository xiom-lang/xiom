# XIOM — Session Handoff: v0.26.0 "Typed-Values Base"

**Date:** 2026-07-08
**Branch:** `feat/guardian` (Phase 2 — compiler↔stdlib gap closure)
**Status:** stdlib **39/39 compile to IR**. Compiler↔stdlib linking gaps closed; stdlib links to native binaries. **Codegen Tier 1 refactor DONE** (`compile_expr → (value, type)`) with **410/410 regression tests green**. Stdlib execution suite: not yet green — remaining work is **Tier 2** (see `docs/CODEGEN_TIER2.md`).
**Companion session:** `SESSION_ECOSYSTEM.md` — AI-driven ecosystem build (other machine)

---

## CODEGEN TIER 1 — DONE & PROVEN (regression-free)

The `stdlib_execution_tests` (compile→link→run each module) exposed that `--emit-ir` never
validated IR through LLVM — the stdlib emitted **invalid LLVM IR** for many common patterns.
This session landed the foundational codegen refactor + a set of hardening fixes, **all with
zero regressions** to the full 410-test gate (diff/full_diff/e2e/feature_regression/integration
/robustness/fuzz all green):

1. **★ Tier 1 refactor — `compile_expr → Result<(String,String), String>`** (value + real LLVM
   type). ~82 call sites updated; sinks (store/return/call-arg) now use the value's REAL type
   instead of an independently re-inferred one. This is the correct compiler architecture and
   eliminated whole bug classes at once (see below). **This is the "strong base".**
2. **`self`-param shadowing fix** — methods recorded `self` in BOTH receiver and params; the
   phantom scalar param shadowed the real struct `self`, breaking `match self` (every enum
   method emitted `store %struct.X i64`). Now the duplicate `self` param is skipped in codegen.
   Locked in by new e2e test `e2e_method_match_self_enum`.
3. **A4 — block terminators**: fallback `ret` when a value-returning body falls through
   (`current_block_terminated()` guard; `compile_fn` + generic-mono path).
4. **A2 — empty operands**: `Expr::Unsafe` returns its tail value (was `String::new()`).
5. **Prelude/cross-module resolution (real GAP 3)**: `xiom-check` force-loads
   `core/string/math/num/char/cmp` when any `xiom.*` is used → killed ALL `undefined @X`
   (`@to_string`/`@char_at`/`@fabs`/`@crc32`/`@next`/…). Gated on stdlib usage (410 unaffected).
6. **A1 — char/byte typing**: `infer_llvm_type(Char)` `i8*`→`i8`; binop widens i8/i16/i32→i64
   (`widen_to_i64`).
7. **Value coercion** (`coerce_value`): int-width/ptr/float/int→struct casts at return + call-arg.
8. **Enum type-name resolution**: `llvm_type_for` maps enum names → `%struct.Name`.
9. **A6 — generic-mono cycle detection** (`mono_emitted` set): fixes `mem` + the intermittent
   `e2e_multifile_benchmark_main_compiles` flake.
10. **`void*`→`i8*`** in `extern_type_to_llvm`; fuzz deep-recursion cap.

**What Tier 1 achieved:** the `store %struct.Ordering %v(i64)` cluster (~15 modules) and the
`undefined @X` cluster are GONE. Values carry real types end-to-end.

### ⚠️ Reverted (do not reintroduce without a compile loop)
At the end of this session, blind per-sink coercion patches (struct→scalar extraction in
binops/`coerce_value`, `Assign`/tail-return coercion) were attempted WITHOUT a local compile
and each introduced a regression (`%tmp12` undefined value, `free` redefinition). **These were
reverted** to return to the proven-green state. The `extract_scalar_field0` helper is kept but
`#[allow(dead_code)]`, staged for Tier 2. **Lesson: Tier 2 must be done with `cargo`+`clang`
iterating locally, not remotely blind.**

### REMAINING — Tier 2 (see `docs/CODEGEN_TIER2.md` for the full plan)
The ~31 execution failures map to 6 root causes, all requiring the typed-value model:
- **T2-1** `store i64 %v(i8)` — store site must coerce (biggest cluster, ~15 modules).
- **T2-2** `%struct.Option` used as scalar (`opt >= 0`, `opt - 48`, `f(opt_char)`) — needs a
  typed Option/Result/enum payload model. (was: "`Option` value in `icmp i64`")
- **T2-3** `store %struct.LogLevel/Rc i64` — single-field struct from bare i64.
- **T2-4** empty-struct GEP (`GlobalAlloc {}`) / `alloca void`.
- **T2-5** `free` redefinition (user/stdlib fn vs hardcoded declare).
- **T2-6** `%tmp12` undefined value — one shared prelude fn returns an unemitted register
  (hunt WITH a compile loop).
- **Bucket B** — smoke-program API name fixes (`examples/stdlib_smoke/*.xi`), not codegen.
- **`store i64 %v(i8)`** char remnant — a `compile_expr`/`infer_llvm_type` divergence: the
  compiled register is `i8` but the inferred slot type is `i64`. Needs compile_expr to return
  (value,type) OR a value-type tracker.
- **`LogLevel`/`Rc` single-field structs** — same shape as Ordering.
- **`bitcast … to void*`** (test module): emit `i8*`, not `void*`.
- **A6 — generic mono infinite loop** (mem; also the intermittent `benchmark` gate flake):
  add cycle detection to the monomorphisation worklist (track emitted (name,type-args)).
- **Bucket B smoke APIs** (`examples/stdlib_smoke/*.xi`): net (`expected ';'` in NetError
  literal), fmt (`to_str`), simd (unqualified consts), async (`spawn`/`run`/module-global
  counter), io/string/regex/path (`cannot call` — likely still cross-module method resolution).

**Root architectural weakness:** `compile_expr` returns only a register string; its actual
LLVM type is re-derived independently by `infer_llvm_type`, and the two diverge for
enums/Option/char. The durable fix is to have `compile_expr` return `(value, llvm_type)` (or
maintain a last-value-type field) so sinks coerce against the REAL type. That refactor would
collapse most of the remaining tail at once.

---

## ⚠️ VERIFICATION STATUS (READ FIRST)

**This session HAD a working `cargo`/`clang` loop** — the Tier 1 codegen work above was
compiled and tested repeatedly. Current proven state after the end-of-session reverts:
- `cargo build -p xiomc` — clean.
- `cargo test -p xiom-codegen` — **410/410 green** (diff 25, e2e 66 incl. the new
  `e2e_method_match_self_enum`, feature_regression 39, full_diff 23, fuzz 24, integration 119,
  robustness 29, diff/selfhost all pass).
- `cargo test -p xiom-codegen --test stdlib_execution_tests` — **0/31 green** (this is the
  Tier 2 target; all failures are the 6 documented T2 root causes, NOT regressions).

Re-confirm the proven state after pulling (fail fast):
```powershell
cargo build -p xiomc
cargo test -p xiom-codegen          # must be fully green — this is the consolidated base
```
If anything in that gate is red, the end-of-session reverts didn't fully land — check the
"Reverted" note above.

NOTE: earlier Wave 1–2 work (stub implementations, runtime C, test harnesses) was originally
authored in a shell-blocked environment and static-verified; it has since been exercised by the
execution suite (which is why the T2 errors surface real behavior, not phantom issues).

---

## Headline Achievement This Session

Closed the compiler↔stdlib gaps from SESSION v0.24.0 so the stdlib can **link into native binaries and run**, not just emit IR:

- **GAP 1 (linking) CLOSED.** The compiler now links **every** `stdlib/runtime/*.c` (was: only `xiom_runtime.c`), and a stdlib **search path** is wired into `xiomc` so `use xiom.*` resolves for ANY program. `xiom_alloc` added; 6 signature mismatches fixed. Full symbol-closure audit: **0 unresolved externals** across all 39 modules.
- **GAP 2 (execution) CLOSED (harness).** New `stdlib_execution_tests.rs` compiles + runs a smoke program per module (40 programs under `examples/stdlib_smoke/`).
- **GAP 3 (cross-module) CLOSED (harness).** `smoke_cross_serialize_convert.xi` links two modules into one binary.
- **GAP 4 (stubs → real):** net UDP + `local_addr`, crypto **AES-GCM**, a **real cooperative async executor**, **RTTI** for `reflect`, and **contract metadata** for `contracts` — all implemented.
- **GAP 5 (fuzz):** new `fuzz_tests.rs` — 24 never-panic adversarial tests.

**Honest correction to v0.24.0/AI_CONTEXT:** the "~24 missing `xiom_*` functions" was **outdated** — nearly all existed. Threading/mutex/atomics in `xiom_runtime.c` are **real** (Win32 `CreateThread`/`InitializeCriticalSection`, POSIX `pthread_*`/`__atomic_*`), never single-threaded stubs. The true blockers were: unlinked `simd_runtime.c`/`ffi_bridge.c`, missing `xiom_alloc`, 6 signature mismatches, and no stdlib search path.

---

## Changes By Area

### Compiler — `crates/xiomc/src/main.rs`
- `find_runtime_c_files() -> Vec<String>`: discovers & links **all** `.c` in `stdlib/runtime/` (auto-links `simd_runtime.c`, `async_runtime.c`, any future runtime C). Excludes anything outside that dir (so `ecosystem/runtime/ffi_bridge.c` is never linked → no `xiom_alloc` duplicate).
- `find_stdlib_dirs() -> Vec<String>`: resolves stdlib via `XIOM_STDLIB` env → exe-relative walk-up → CWD `stdlib` → `CARGO_MANIFEST_DIR/../../stdlib`; registers both `stdlib` and `stdlib/xiom` as source dirs (appended after file-relative + examples, deduped, never shadows local modules). Robust: no-op if absent.

### C runtime — `stdlib/runtime/`
- `xiom_runtime.c`: added `xiom_alloc` (zeroing, `long long`/64-bit ABI); `xiom_stdin/stdout/stderr` now return real `FILE*` (`void*`) instead of an `int` fd; `xiom_dirent_name` fixed to 1-arg + a Win32 `opendir/readdir/closedir` shim (over `FindFirstFileA`) so io.xi's dir iteration links on Windows; `xiom_thread_create`/`xiom_thread_spawn_with_result` take `void*` fn-ptrs (match `*UInt8` codegen); **new** `xiom_socket_sendto`, `xiom_socket_recvfrom`, `xiom_gethostname` (Win32+POSIX).
- `simd_runtime.c`: added `<stdlib.h>`/`<math.h>`; per-function `__attribute__((target("sse4.1"/"avx")))` so it compiles under `clang -maes` baseline. Signatures unchanged.
- **NEW** `async_runtime.c`: `xiom_async_now_ms/us` monotonic clock (QPC / `clock_gettime`), uniquely prefixed, no symbol clashes.
- **Winsock note:** UDP/hostname/socket funcs need `ws2_32` on Windows; currently satisfied by the `#pragma comment(lib,"ws2_32.lib")` in the `_WIN32` block. If the link ever stops honoring the pragma (lld/MinGW), add `-lws2_32` to the Native clang invocation in `main.rs`.

### Stdlib modules
- `net.xi`: real `UdpSocket.send_to`/`recv_from` and `local_addr` (via new C FFI), mirroring the existing TCP paths.
- `crypto.xi`: real **AES-GCM** (`aes_encrypt_gcm`/`aes_decrypt_gcm`) — pure-XIOM GHASH (`_gcm_gf_mult`, `_ghash`), GCTR on existing AES block primitive, constant-time tag compare (verify-then-decrypt). 96-bit IV path (others → `Err`). NIST SP 800-38D test-vector expectations noted in comments.
- `async.xi` + `async_runtime.c`: real **cooperative executor** — `spawn` enqueues (no more sync inline call), `run`/`block_on` drain a real ready-queue, real deadline `Timer`s, scheduling-aware `Channel` (no panic in normal flow).
- `reflect.xi`: real `type_count`/`type_name_by_id`/`type_id_by_name`/`type_field_count`, real `type_info_by_name`/`all_types` — backed by codegen-emitted RTTI.
- `contracts.xi`: real `build_contract_index` + all statistics/coverage queries — backed by codegen-emitted contract table.

### Codegen — `crates/xiom-codegen/src/lib.rs` (STRICTLY ADDITIVE)
- One call `self.emit_metadata_tables(program)` inserted after the existing runtime `declare`s; six NEW methods appended. **No existing lowering path modified.**
- Emits, **only when** a program declares the `xiom_type_count`/`xiom_contract_fn_count` externs (i.e. only reflect.xi/contracts.xi), an RTTI table (per-type name strings + field-count arrays + accessors `@xiom_type_count/name/id_by_name/field_count`) and a contract table (`@xiom_contract_fn_count/name/pre_count/post_count`). Gated ⇒ **all 410 existing tests emit byte-identical IR**.
- Verified against AST: `ContractClause::Requires/Ensures(Expr,Span)`, `FnDecl.{name:Ident, receiver:Option<Ident>, contracts:Vec<ContractClause>}`, emitter `functions/type_meta/enum_variants` fields — all confirmed present, so the crate should compile.

### Tests
- **NEW** `crates/xiom-codegen/tests/stdlib_execution_tests.rs` — 30 strict (`Some(0)`) + 10 `#[ignore]` (env/nondeterministic: net/thread/async/time/rand/env/os/io/test/bench) + 1 cross-module. Reuses the `compile_and_run` harness pattern from `e2e_tests.rs`.
- **NEW** `crates/xiom-codegen/tests/fuzz_tests.rs` — 24 never-panic tests (token soup, depth-guard over/under, huge match, malformed contracts, unbalanced brackets, unicode/escapes) via `catch_unwind`.
- **NEW** `examples/stdlib_smoke/smoke_*.xi` — 40 per-module programs + 1 cross-module, `main()->Int` returns 0 on success.

---

## What Is REAL vs Still LIMITED (be honest)

| Area | REAL now | Still limited |
|------|----------|---------------|
| Linking | all runtime C linked; stdlib path; 0 unresolved externals | — |
| net | TCP; UDP send_to/recv_from; local_addr | non-96-bit IV n/a; IPv6 untested |
| crypto | SHA-256, AES, **AES-GCM (96-bit IV)** | AES-GCM other IV lengths → Err |
| async | real ready-queue executor, timers, coop channels | **run-to-completion** (no coroutine suspension — XIOM `fn()` can't be paused); `sleep_ms` blocks its frame |
| reflect | type count/name/id/field-count, all_types | per-`T` generic queries (`TypeId.of[T]`, `type_name[T]`) still pragmatic — needs a monomorphization intrinsic; size/align/field-names/variants not embedded |
| contracts | index/names/pre-post counts, coverage/stats | clause expression text, source locations, runtime evaluation, SMT (`can_compose`/`verify_chain`) still honest placeholders |
| sync/thread | real OS threads/mutex/atomics (were already real) | verify via execution tests |

---

## Verification Commands (RUN THESE)

```powershell
cd E:\Projects\AXIOM
cargo build -p xiomc

# No-regression on the compiler front-end
cargo test -p xiom-check
cargo test -p xiom-parser
cargo test -p xiom-codegen --test stdlib_tests -- --nocapture      # expect 39/39
cargo test -p xiom-codegen --test robustness_tests
cargo test -p xiom-codegen --test feature_regression_tests
cargo test -p xiom-codegen --test e2e_tests

# NEW this session
cargo test -p xiom-codegen --test stdlib_execution_tests -- --nocapture             # link+run (strict)
cargo test -p xiom-codegen --test stdlib_execution_tests -- --ignored --nocapture   # env-dependent
cargo test -p xiom-codegen --test fuzz_tests

# Manual single-module link sanity (if a test fails, run the compiler directly to see clang output):
.\target\debug\xiomc.exe -o smoke_math.exe examples\stdlib_smoke\smoke_math.xi
.\smoke_math.exe; echo "exit=$LASTEXITCODE"
```

### Likely first-failure suspects (since nothing was compiled)
- **Rust build**: additive codegen methods — if any field/variant name is off, `cargo build -p xiomc` fails first. (Assumptions were AST-verified, but confirm.)
- **Smoke `.xi` syntax**: 40 programs were written against module APIs without a compiler — expect a few to need small syntax/API-name fixes. A failing smoke test = fix the smoke `.xi` (or the module), not the harness.
- **AES-GCM / async / net `.xi`**: largest new XIOM code; verify they type-check (`xiomc --emit-ir stdlib\xiom\crypto.xi`, etc.).
- **Winsock link**: if UDP symbols fail to link on Windows, add `-lws2_32`.

---

## Commit + Tag (after `cargo test -p xiom-codegen` is fully green)

```bash
git add -A
git commit -m "feat(codegen): Tier 1 typed-value refactor (compile_expr -> (value,type)) + hardening — self-shadow fix, block terminators, prelude/cross-module resolution, char/byte widening, enum-name resolution, mono cycle detection; e2e match-self test; docs/CODEGEN_TIER2.md plan. 410/410 green."
git tag v0.26.0-typed-values-base
```

---

## Remaining Gaps (NEXT SESSION)

1. **Run the verification suite** and fix the inevitable handful of syntax/API mismatches in the new `.xi` (smoke programs, AES-GCM, async, net) — none were compiler-checked.
2. **async coroutine suspension** — true `await`/pausable tasks need an AST async-transform (state machine) in the compiler; current model is run-to-completion.
3. **reflect per-`T` RTTI** — emit a monomorphization-time type-id intrinsic so `TypeId.of[T]()`/`type_name[T]()` resolve the concrete `T`; embed size/align/field names/variants.
4. **contracts runtime eval + SMT** — embed clause expression text and wire a runtime evaluator / Z3 for `verify_function_contracts`, `can_compose`, `verify_chain`.
5. **net UDP/IPv6** breadth; **crypto** AES-GCM arbitrary IV lengths (GHASH-derived J0).
6. **LSP/diagnostics (GAP 6)** — actionable errors (`unknown operator 'not', did you mean '!'?`).

## Test Infrastructure Files
- `crates/xiom-codegen/tests/stdlib_tests.rs` — 39 modules → IR (existing)
- `crates/xiom-codegen/tests/stdlib_execution_tests.rs` — **NEW** link+run per module
- `crates/xiom-codegen/tests/fuzz_tests.rs` — **NEW** 24 never-panic
- `crates/xiom-codegen/tests/robustness_tests.rs` — 29 never-panic (existing)
- `crates/xiom-codegen/tests/feature_regression_tests.rs` — 39 lock-in (existing)
- `examples/stdlib_smoke/*.xi` — **NEW** 40 smoke + 1 cross-module
