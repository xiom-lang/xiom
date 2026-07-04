# XIOM Compiler — Version History & Roadmap

> Living document tracking all compiler releases and planned milestones.
> Last updated: 2026-07-04

---

## Versioning Policy

XIOM Compiler uses **semantic versioning** (`MAJOR.MINOR.PATCH`):

| Component | Meaning |
|-----------|---------|
| **MAJOR** | Breaking language changes or bootstrap milestones (e.g., self-hosting, v1.0) |
| **MINOR** | New compiler features corresponding to a new Phase (e.g., Phase 1 full language surface) |
| **PATCH** | Bug fixes, hardening, and polish within a phase |

Each version carries a **codename** reflecting the phase theme:

| Phase | Codename Theme | Status |
|-------|----------------|--------|
| Phase 0 | **Pipeline** — foundations | ✅ v0.1.0 |
| Phase 1 | **Guardian** — full language surface | ✅ v0.2.0–v0.2.5 |
| Phase 2 | **Hardened** — hardening, performance, toolchain | 🔧 v0.22.1 (current) |
| Phase 3 | **Sovereign** — Z3, debugger, LSP, ecosystem | 📋 Planned |
| Phase 4 | **Rebirth** — self-hosting | 📋 After Phase 3 |

### Branch Release Strategy

| Branch | Purpose | Version Tag Pattern |
|--------|---------|---------------------|
| `main` | Stable releases | `v0.22.1`, `v0.23.0`, `v1.0.0` |
| `feat/guardian` | Phase 2 hardening | `v0.23.0-guardian.1`, `v0.23.0-guardian.2` (pre-release) |
| Future `feat/*` | Feature branches | `vX.Y.Z-<feature>.N` (pre-release) |

**Pre-release tags** are semver-compliant: `v0.23.0-guardian.1` means "the first guardian pre-release of what will become v0.23.0." These tags let you test branch features without polluting the stable version line. When the branch merges to main, the stable tag drops the pre-release suffix.

---

## Where We Are Now

**Current version: v0.22.1 "Hardened" — Phase 2**
- 186 Rust tests, zero warnings, zero failures
- Full language surface complete (ownership, contracts, generics, modules, derive, enums, closures, async, FFI)
- ModuleCatalog multi-file resolution with scan-fallback
- Struct return + tuple return codegen
- Module-qualified naming for collision-free merges
- 30-module benchmark suite compiles
- Str.len(), clang subsystem, v10 flake — all fixed

**What Phase 2 (Guardian) means:**
Phase 2 is the hardening phase. We have a working compiler with the full language surface. Now we make it fast, reliable, and production-ready. No new language features — only performance, safety, tooling, and ecosystem.

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

## v0.2.5 "Hardened" — Phase 1.5 (2026-06-30)

**Status: Released.** 13 bug fixes: match codegen, struct field types, enum derive, borrow false positives, target triples, div-zero guards, GEP syntax. New: interface dispatch, full match binding, Vec runtime, malloc/free.

**Tests:** 109 passed | **Rust LOC:** ~6,924

---

## v0.22.1 "Hardened" — Phase 2 (2026-07-04)

**Status: Released.** 186 tests, zero warnings, zero failures. The compiler is hardened and production-ready for Phase 2 development.

### ModuleCatalog — Multi-File Resolution
- `ModuleCatalog` + `CachedModule` with path-based + scan-based + last-segment fallback loading
- `collect_external_decls`: full-body type/function injection from cached modules
- `register_external_module`: merge without overwrite
- `fn_symbol`: collision-free naming for multi-file merges
- Resolve flow: `resolve_imports` → catalog prefix loading → `process_use` → injection → codegen

### Codegen Fixes
- Struct return types: `ret %struct.BenchResult %val` (was i64 mismatch)
- Tuple return types: `(Vec[Int], Vec[Int])` → anonymous tuple struct
- `Str.len()` → `@axiom_str_len`
- Module-qualified naming: `fn_symbol` + `resolve_module_call` for cross-module calls
- Destructure: alloca+store before GEP extraction
- Match: guard for empty check_labels array

### Warnings & Stability
- 12 compiler warnings eliminated (unused vars, unreachable patterns, dead code, useless comparisons)
- v10 selfhost e2e flake: unique output filenames
- clang linker: `/SUBSYSTEM:CONSOLE` for Windows
- Compiler architecture documented, improvement plan written

### Benchmark Suite
- 30-module suite compiles (25 original + 5 hardened: ownership, borrow, contracts_hard, generics_hard, monomorph)
- `benchmark_safe.xi` — exit 34
- `benchmark_stress.xi` — 10K lines, tuple return partition() works
- `test_mod/math.xi` — multi-file catalog: exit 34
- `benchmark/main.xi` — 30-module merge: valid IR

**Tests:** 186 passed (44 check + 141 codegen) | **Rust LOC:** ~11,700

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

| Version | Codename | Phase | Date | Tests | Key Milestone |
|---------|----------|-------|------|-------|---------------|
| **v0.1.0** | Pipeline | 0 | 2026-06-30 | 36 | First working pipeline |
| **v0.2.0** | Guardian | 1 | 2026-06-30 | 87 | Full language surface |
| **v0.2.5** | Hardened | 1.5 | 2026-06-30 | 109 | 13 bug fixes, interface dispatch, Vec |
| **v0.22.1** | **Hardened** | **2** | **2026-07-04** | **186** | **ModuleCatalog, tuple/struct codegen, multi-file, zero warnings** |
| v0.23.0 | Guardian | 2 | TBD | 200+ | Vec realloc, perf foundations |
| v0.24.0 | Guardian | 2 | TBD | 250+ | Hot reload, multithreaded |
| v1.0.0 | Sovereign | 4 | TBD | 1000+ | Self-hosting, ecosystem, production |

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
