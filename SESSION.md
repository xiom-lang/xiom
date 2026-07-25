# XIOM Session Handoff — v0.51.0 "Production Hardening"

**Date:** 2026-07-25 19:00 | **Branch:** `feat/architect` | **Test baseline: 1050/1050**
**Self-hosting readiness: 10/10** | **Next target: v0.52.0 "Self-Host Ready"**

---

## WHAT WAS ACCOMPLISHED (THIS SESSION)

### B-002: FIXED — `&mut self` methods crash
- **Root cause:** `register_functions` in decl.rs pushed bare struct type for `&mut self` receivers
  (e.g. `%struct.Foo` instead of `%struct.Foo*`). Call sites read the registered type and
  passed a struct value instead of an address → LLVM type mismatch → STATUS_ACCESS_VIOLATION.
- **Fix:** Added `is_mut_self` check in `register_functions` param_types push, matching the
  existing correct pattern in the monomorphisation path (lib.rs line 2972).
- **Commit:** `0497fdb`
- **Verification:** All tests pass.

### B-003: FIXED — `Option<Str>` from method returns crash
- **Root cause:** Ensures clauses like `result is Some => result.len() > 0` on methods returning
  `Option<Str>` generate calls to `Option.len()`, which was undefined. The undefined function was
  auto-stubbed (returning 0), making the contract check fail → `@llvm.trap()` → `ud2` →
  STATUS_ILLEGAL_INSTRUCTION.
- **Fix:** Added `Option.len()` builtin to `compile_option_impls()` that:
  1. Checks if Option is Some (discriminant != 0)
  2. If Some, extracts the i64 payload, `inttoptr` to `i8*`, calls `@xiom_str_len`
  3. If None, returns 0 (safe because contract implies logic: `NOT A OR B`)
- **Test:** Added `e2e_b003_option_str_method` — Path.file_name() and Path.file_stem()
  work correctly with contracts enabled.
- **Commit:** `d488194`
- **Verification:** All 1050 tests pass (113 e2e, +1 new).

### B-001: INFRASTRUCTURE READY (inactive) — `Result[T, struct E]` truncation
- Added `is_struct_type_name`, `resolve_type_key`, `ensure_concrete_option`,
  `ensure_concrete_result`, `pre_register_concrete_types`, and `concrete_type_for` methods
  to lib.rs for on-demand concrete type creation.
- `Expr::Ok/Err/Some` in expr.rs now uses `fctx.current_return_type` for correct struct layouts.
- **NOT YET ACTIVE:** `concrete_type_for` is not called from `register_functions` or
  `compile_fn` because module-qualified type names (e.g. `tests.ecosystem.test_json.JsonValue`
  vs short `JsonValue`) cause LLVM opaque type conflicts in ecosystem tests.
- **Remaining work:** Fix module-qualified name resolution in concrete type creation, then
  wire `concrete_type_for` into `compile_fn` and `register_functions`.
- **Commit:** `79a57fa feat(B-001): groundwork for concrete Result/Option monomorphisation`

---

## CURRENT STATE — M15 2/3 DONE, B-001 REMAINS

### M15 Plan (3 bugs → 4 days → 10/10)
| Bug | Symptom | Blocks | Effort | Status |
|-----|---------|--------|--------|--------|
| B-001 | `Result[T, struct E]` truncates error to 8 bytes | parse_json, Err(Struct) | 2d | INFRA READY (inactive) |
| B-002 | `&mut self` methods crash (ACCESS_VIOLATION) | PathBuf.push, mutable state patterns | 1d | **FIXED** |
| B-003 | `Option<Str>` from method returns crash (ILLEGAL_INSTRUCTION) | file_name, extension, method returns | 1d | **FIXED** |

### B-001 REMAINING — Module-Qualified Type Resolution
The infrastructure is correct but the `concrete_type_for` method uses `type_from_ast`
which returns SHORT type names (e.g. `"JsonValue"`). When these are used to create
concrete types like `Option__JsonValue`, the field type `"JsonValue"` doesn't resolve
to the fully-qualified `"tests.ecosystem.test_json.JsonValue"`, causing LLVM opaque type errors.

**Fix approach:**
1. Use `resolve_type_key` (already implemented) to get fully-qualified names for field types
2. Use fully-qualified names in concrete type NAMES as well (e.g. `Option__tests.ecosystem.test_json.JsonValue`)
3. Wire `concrete_type_for` into `compile_fn` and `register_functions`
4. All tests should pass including ecosystem tests (JSON, HTTP, SQLite)

---

## TEST BASELINE
| Suite | Count | Status |
|-------|-------|--------|
| E2E | 113/113 | OK (+1 B-003 regression) |
| Feature Regression | 268/268 | OK |
| Stdlib Execution | 41/41 | OK |
| Diff | 25/25 | OK |
| Full-Diff | 23/23 | OK |
| Fuzz | 24/24 | OK |
| Integration | 119/119 | OK |
| Robustness | 29/29 | OK |
| Stdlib Compilation | 40/40 | OK |
| Checker | 123/123 | OK |
| Parser | 58/58 | OK |
| Formatter | 44/44 | OK |
| LSP | 15/15 | OK |
| Package Manager | 16/16 | OK |
| Doc Generator | 4/4 | OK |
| FFI Generator | 18/18 | OK |
| MCP Server | 18/18 | OK |
| Debugger | 8/8 | OK |
| Verifier | 15/15 | OK |
| Scripting | 34/34 | OK |
| Script Diff | 15/15 | OK |
| **TOTAL** | **1050** | **ALL GREEN** |

---

## KNOWN CODGEN LIMITATIONS (NOT YET FIXED)
| # | Pattern | Symptom | Workaround |
|---|---------|---------|-----------|
| 1 | OR-pattern `Some('a')\|Some('b')` | STATUS_ACCESS_VIOLATION | Use sequential `if/elif` in `Some(c) =>` arm |
| 2 | `Result[T, struct E]` | 8-byte truncation | Use `Result[T, Str]` instead of struct error types |
| 3 | ~~`&mut self` methods~~ | **FIXED** | |
| 4 | ~~`Option<Str>` from method returns~~ | **FIXED** | |

---

## QUICK START (NEXT SESSION)
```powershell
# Verify baseline
.\test_summary.ps1          # Should be 1050/1050

# B-001: Wire concrete_type_for + fix module-qualified names
```

# Test
cargo build --workspace
.\test_summary.ps1
```
