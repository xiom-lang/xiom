<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# JSON Diagnostics v1 (shared schema)

**Status:** v1, landed round 45 (2026-09-11). **Producer:** `xiom
--diagnostics=json` / `xiom::compile_with_diagnostics` (field
`CompileResult::diagnostics`). **Consumers:** CLI, MCP tools, CI scripts;
the LSP maps the same fields onto LSP `Diagnostic`/`Range` (UTF-16).

## Envelope

Every diagnostics payload from the compiler is one JSON object:

```json
{
  "schema_version": 1,
  "diagnostics": [ /* Diagnostic objects, oldest first */ ]
}
```

`xiom::diagnostics_json(&[Diagnostic])` is the single serializer
(`serde_json`), so escaping is always valid JSON -- the previous
hand-rolled printers emitted raw control characters and each call site
used a different object shape.

Check-only success is a **status object**, not a diagnostics list:

```json
{ "schema_version": 1, "status": "check_passed", "type_errors": 0,
  "borrow_warnings": 0, "time_ms": 0 }
```

## Diagnostic object

| Field | Type | Required | Meaning |
|---|---|---|---|
| `kind` | string | yes | `lex_error`, `parse_error`, `type_error`, `borrow_error`, `borrow_warning`, `codegen_error` |
| `code` | string | yes | stable id: `L001`, `P001`, `T001`, `E001`, `C001` |
| `message` | string | yes | human-readable, already localized to English |
| `line` | number | yes | 1-based source line (0 when not position-specific, e.g. codegen failures) |
| `col` | number | yes | 1-based Unicode-scalar column |
| `file` | string | yes | source path; `<unknown>` when the pipeline lost it |
| `suggestion` | string | no | machine-actionable fix hint |
| `help` | string | no | extended explanation |
| `note` | string | no | secondary context |

Unknown fields MUST be ignored by consumers. v1 changes only add optional
fields or new `kind`/`code` values; removing or renaming a field requires
`schema_version: 2`.

## Consumer notes

- **LSP:** positions are re-derived from XIOM spans via
  `xiom-lsp::position` (zero-based line + UTF-16 character), not taken
  from these 1-based scalar columns directly.
- **MCP:** `compile`/`check` tools return
  `CompileResult::diagnostics` as-is; the envelope is for CLI/CI output.
- **CI:** parse the envelope, assert `schema_version == 1`, and match on
  `code` (stable) rather than `message` (prose).

## Source of truth

- Object struct: `crates/xiom/src/lib.rs` (`Diagnostic`,
  `DiagnosticsEnvelope`, `diagnostics_json`).
- Test: `diagnostics_json_v1_schema_is_valid_and_escapes` (valid JSON,
  escaping of quotes/newlines/control characters, field presence).
