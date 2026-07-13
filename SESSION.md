# XIOM — Session Handoff: v0.45.0 "Hardened"

**Date:** 2026-07-13
**Branch:** `feat/guardian`
**Status:** 37/37 smoke, 84/84 e2e, all gates green
**Tag:** `v0.45.0-hardened`

---

## CARRY-ON PROMPT

```
Continue XIOM compiler production hardening from SESSION.md (tag v0.45.0).
Branch: feat/guardian. 37/37 smoke tests pass, 84/84 e2e, all gates green.

BUGS TO RESOLVE:
1. async expression paths — read_module_header now handles preamble statements,
   checker has imported_items→modules bridge in check_module_field_access and
   resolve_module_function, but Expr::Ident handler at check_expr:2141 still
   doesn't find "async" in imported_items. Root: process_use may return early.
   Fix: investigate why process_use current.get("async") returns None.

2. SHA-256 known-vectors now correct via C FFI (sha256_sw.c). ✅

PHASE 5a ITEMS (from ROADMAP.md §5a):
- ARC A: Real pointer/reference types (design in ARC_A_POINTERS.md)
- Const-generics: [N]T arrays
- Match expression type unification
- Interface/trait dispatch

KEY FILES: docs/ROADMAP.md, docs/PRODUCTION_HARDENING_BUGS.md,
crates/xiom-check/src/lib.rs, crates/xiom-codegen/src/lib.rs

VERIFICATION: cargo test -p xiom-codegen --test stdlib_execution_tests
```
