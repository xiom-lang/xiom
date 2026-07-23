// XIOM Language Server — Diagnostics helpers
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_check::types::CheckError;
use xiom_parser::ParseError;

pub fn make_range(span: &xiom_ast::Span) -> serde_json::Value {
    let line = if span.line > 0 { span.line - 1 } else { 0 };
    let col = if span.col > 0 { span.col - 1 } else { 0 };
    serde_json::json!({
        "start": { "line": line, "character": col },
        "end": { "line": line, "character": col + 1 }
    })
}

pub fn diagnostic_from_parse_error(err: &ParseError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(&err.span),
        "severity": "Error",
        "message": format!("Parse error: {}", err.message)
    })
}

pub fn diagnostic_from_check_error(err: &CheckError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(&err.span),
        "severity": "Error",
        "message": format!("Type error: {}", err.message)
    })
}
