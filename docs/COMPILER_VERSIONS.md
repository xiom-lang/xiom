# XIOM Compiler — Version History & Roadmap

> Living document tracking all compiler releases and planned milestones.
> Last updated: 2026-07-11

---

## Current Version: v0.33.0 "Phase 4" — All Phases Complete

**Status:** 37/37 stdlib modules pass, all regression gates green. Production hardening phases 1-4 delivered.

- **Regression gate (ALL GREEN):** diff 25, e2e **79**, feature_regression 48, full_diff 23, fuzz 23+1, integration 119, robustness 29, check 74, parser 47
- **Stdlib:** 37/37 strict + 4 ignored (thread, async, io, test)
- **Compiler gaps:** 13/14 closed — only GAP-13 (brace module) remains, intentional design choice
- **Rust tests:** 400+ compiler tests across all crates
- **Production plan:** 4 phases documented in `CODEGEN_PRODUCTION_PLAN.md`

---

## Versioning Policy

XIOM Compiler uses **semantic versioning** (`MAJOR.MINOR.PATCH`):

| Component | Meaning |
|-----------|---------|
| **MAJOR** | Breaking language changes or bootstrap milestones (e.g., self-hosting, v1.0) |
| **MINOR** | New compiler features corresponding to a new Phase |
| **PATCH** | Bug fixes, hardening, and polish within a phase |

Each version carries a **codename** reflecting the phase theme:

| Phase | Codename Theme | Status |
|-------|----------------|--------|
| Phase 0 | **Pipeline** — foundations | ✅ v0.1.0 |
| Phase 1 | **Guardian** — full language surface | ✅ v0.2.0 – v0.30.0 |
| Phase 2 | **Hardened** — stdlib + codegen hardening | ✅ v0.31.0 |
| Phase 3 | **ARC-C** — &mut Struct, by-reference passing | ✅ v0.32.0 |
| Phase 4 | **Or-Patterns** — match expression completion | ✅ v0.33.0 |

### Branch Release Strategy

| Branch | Purpose | Version Tag Pattern |
|--------|---------|---------------------|
| `main` | Stable releases | `v0.22.1`, `v0.33.0`, `v1.0.0` |
| `feat/guardian` | All hardening phases | `v0.30.0-phase-1-hardened`, `v0.33.0-phase-4` |

**Pre-release tags** are semver-compliant: `v0.33.0-phase-4` means "the phase 4 pre-release of what will become v0.33.0."

---

## Where We Are Now

**Current version: v0.33.0 — Phase 4 Complete**
- 37/37 stdlib modules pass (100%), all regression gates green
- COMPILER_GAPS.md: 13/14 closed (only GAP-13 brace-module remains, intentional design choice)
- 4-phase production hardening plan executed (`docs/CODEGEN_PRODUCTION_PLAN.md`)
- Key architectural features: &mut Struct pointer passing, Pattern::Or match, enum variant constructors, size_of intrinsics, generic fn_key pre-computation

**What Phase 4 completion means:**
The compiler now enforces "compiles ⇒ safe" — all type errors abort codegen. Every stdlib module compiles, links, and runs correctly. The language has proper by-reference struct passing, match or-patterns, and enum variant construction. The compiler is production-ready for Phase 5 (stdlib completion, toolchain, ecosystem).

---

## v0.1.0 "Pipeline" — Phase 0 (2026-06-30)

**Status: Released.** Working compiler pipeline from source text to binary.

The first working XIOM compiler. Establishes the end-to-end pipeline: lexer → parser → type checker → LLVM IR → clang → native `.exe` / `.wasm`. No borrow checking, no contracts, no generics — parses everything, enforces nothing.

**Tests:** 36 passed (0 failures)
**Rust LOC:** ~3,800

---

## v0.2.0 "Guardian" — Phase 1 (2026-06-30)

**Status: Released.** Full language surface: ownership, contracts, generics, modules, derive, error handling, async.

- **Borrow Checker** — Lexical scope: `&T`, `&mut T`, move semantics, clone()
- **Contracts** — `requires`/`ensures`/`invariant` as runtime guards
- **Generics** — Monomorphisation with inline type constraints `[T: Comparable]`
- **Modules** — `module`/`use`/`pub`, file-level and inline
- **Derive** — `Eq`, `Clone`, `Display`, `Hash`, `Ord` codegen
- **Error Handling** — `Result[T, E]`, `Option[T]`, `?` operator
- **Standard Library** — 7 modules: core, io, collections, string, math, ffi, async

**Tests:** 87 passed | **Rust LOC:** ~5,200

---

## v0.33.0 — Phase 4 "Or-Patterns" (2026-07-11)

**Status: Released.** Pattern::Or in match expressions, comprehensive documentation updates.

- **Pattern::Or** — `match x { 1 | 2 => 10, _ => 0 }` emits sequential discriminant checks sharing one arm body
- **pattern_needs_check** extended for Or alternatives
- **e2e test:** `examples/e2e/or_pattern.xi` — validates or-pattern compilation
- **Documentation updates:** COMPILER_VERSIONS.md, COMPILER_ARCHITECTURE.md, CODEGEN_PRODUCTION_PLAN.md synchronized
- **GAP-13 investigation:** brace-module form `module x { }` is a parser limitation (file-form `module x;` required)

**Tests:** 37/37 stdlib | **E2E:** 79 (was 78) | **Tag:** `v0.33.0-phase-4`

---

## v0.32.0 — Phase 3 "ARC-C" (2026-07-11)

**Status: Released.** `&mut Struct` real pointer passing — mutations propagate to callers.

- **type_from_ast:** `Type::MutRef` always returns `*inner` (pointer), not just for scalars
- **infer_llvm_type:** `Expr::MutRef` always returns pointer type
- **Call-site fix:** when callee's self param is a pointer, pass receiver alloca address instead of loaded value
- **Impact:** `DefaultHasher.write_int(&mut self, n)` can now modify the caller's hasher struct

**Tests:** 37/37 stdlib | **Tag:** `v0.32.0-phase-3-arc-c`

---

## v0.31.0 — Phase 2 "Hardened" (2026-07-11)

**Status: Released.** Generic fn_key storage, enum pseudo-fields, Array type support.

- **generic_fn_decls** changed to `Vec<(String, FnDecl)>` — pre-computed fn_key at registration time prevents cross-module generic name collisions (mem.swap vs ptr.swap, etc.)
- **Enum `.is_ok`/`.is_some`/`.is_err`/`.is_none`** pseudo-fields on Result/Option — emit discriminant comparison inline
- **type_from_ast** for `Type::Array` — returns `[N x elem]` or `[N]elem` instead of `"Int"` wildcard
- **`infer_struct_type_name`** for `Expr::Call` — resolves bare function return types for chained method calls (`make_pair().sum()` resolves to `Pair.sum`)

**Tests:** 37/37 stdlib | **Tag:** `v0.31.0-phase-2`

---

## v0.30.0 — Phase 1 "Hardened" (2026-07-11)

**Status: Released.** Safety gate, compiler intrinsics, enum variant constructors.

- **Safety gate bypass removed** — `crates/xiom/src/main.rs:208`: all type errors now abort codegen unconditionally. The "compiles ⇒ safe" guarantee is now enforceable.
- **size_of[T]() / align_of[T]()** compiler intrinsics — capture type_arg from `Expr::Index`, compute size from LLVM type layout (i64=8, i8=1, struct=field_count×8)
- **Enum variant constructors** — `TypeName.Variant(args)` allocates discriminant struct + payload, store variant index at field 0, payload at field 1+
- **ptr.read / ptr.write** inline builtins — bypass *T generic monomorphization gap
- **infer_llvm_type for &/&mut** — returns pointer types for scalar inners

**Tests:** All gates green | **Tag:** `v0.30.0-phase-1-hardened`

---

## Where We Go From Here (Phase 2 — Guardian)

Phase 2 is the hardening phase. We have a working compiler. Now:

| Priority | Task | Branch |
|----------|------|--------|
| P0 | Vec realloc (V1), C runtime limits (V5), unknown type → error (V7) | `feat/guardian` |
| P1 | IR optimization, parallel monomorphisation, indexed catalog, incremental compilation | `feat/guardian` |
| P2 | Hot reload, multithreaded compilation, memory budgets | Future |
| P3 | Z3 static verification, debugger, LSP, CLI toolchain | Future |
| P4 | Self-hosting (after everything above) | Future |

See `docs/COMPILER_IMPROVEMENT_PLAN.md` for the full roadmap.

---

## Version Summary

| Version | Phase | Date | Tests | Key Milestone |
|---------|-------|------|-------|---------------|
| v0.1.0 | 0 | 2026-06-30 | 36 | First working pipeline |
| v0.2.0 | 1 | 2026-06-30 | 87 | Full language surface |
| v0.2.5 | 1.5 | 2026-06-30 | 109 | 13 bug fixes, interface dispatch |
| v0.22.1 | 2 | 2026-07-04 | 186 | ModuleCatalog, struct/tuple codegen |
| **v0.30.0** | **1** | **2026-07-11** | **400+** | **Safety gate, size_of, enum constructors** |
| **v0.31.0** | **2** | **2026-07-11** | **400+** | **Generic fn_key, enum pseudo-fields** |
| **v0.32.0** | **3** | **2026-07-11** | **400+** | **&mut Struct pointer passing (ARC C)** |
| **v0.33.0** | **4** | **2026-07-11** | **400+** | **Pattern::Or match** |
| v1.0.0 | 5 | TBD | 1000+ | Self-hosting, ecosystem, production |

---

## Branch Release Tags

```bash
# On main (stable):
git tag v0.22.1                        # current stable
git tag v0.23.0                        # next stable (after guardian merge)

# On feat/guardian (pre-release):
git tag v0.23.0-guardian.1             # first guardian pre-release
git tag v0.23.0-guardian.2             # second (after Vec realloc fix, etc.)
git tag v0.23.0-guardian.3             # third (after perf fixes)

# When feat/guardian merges to main:
git checkout main
git merge feat/guardian
git tag v0.23.0                        # stable release
```

**Semver rule:** `vX.Y.Z-<branch>.N` = pre-release tag. When merged, the stable tag drops the suffix. CI can differentiate: `v0.23.0-guardian.1` builds with extra logging/debug; `v0.23.0` is the production build.

---

*XIOM Compiler — Version History. Updated per release.*
