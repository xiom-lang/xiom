// XIOM Language Server -- Diagnostics helpers
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom_check::types::CheckError;
use xiom_parser::ParseError;

/// LSP range for an XIOM span. Positions are zero-based line + UTF-16 code
/// units (spec), converted against the document text; a multibyte line used
/// to be reported with scalar/byte columns (wrong offsets, and nothing
/// clamped).
pub fn make_range(text: &str, span: &xiom_ast::Span) -> serde_json::Value {
    let (line, start) = crate::position::span_start_lsp(text, span);
    let line_len = crate::position::line_utf16_len(text, line);
    let end = if start < line_len { start + 1 } else { start };
    serde_json::json!({
        "start": { "line": line, "character": start },
        "end": { "line": line, "character": end }
    })
}

pub fn diagnostic_from_parse_error(text: &str, err: &ParseError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(text, &err.span),
        // AUDIT #9 FIX: LSP severities are INTEGERS (1=Error). The string
        // "Error" violated the spec and was dropped by conforming clients.
        "severity": 1,
        "message": format!("Parse error: {}", err.message)
    })
}

pub fn diagnostic_from_check_error(text: &str, err: &CheckError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(text, &err.span),
        "severity": 1,
        "message": format!("Type error: {}", err.message)
    })
}
