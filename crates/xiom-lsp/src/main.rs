// XIOM — Language Server
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Read, Write};
use std::sync::Arc;
use std::sync::Mutex;

use xiom_check::CheckError;
use xiom_lexer::Lexer;
use xiom_parser::{ParseError, Parser};

struct Backend {
    documents: Arc<Mutex<HashMap<String, String>>>,
}

impl Backend {
    fn new() -> Self { Self { documents: Arc::new(Mutex::new(HashMap::new())) } }

    fn publish_diagnostics(&self, uri: &str) -> Vec<serde_json::Value> {
        let text = { let docs = self.documents.lock().unwrap(); docs.get(uri).cloned() };
        let text = match text { Some(t) => t, None => return Vec::new() };
        let mut diagnostics = Vec::new();
        let mut lexer = Lexer::new(&text);
        let tokens = lexer.tokenize();
        let lex_errors: Vec<_> = tokens.iter().filter(|t| matches!(t.kind, xiom_lexer::TokenKind::Error(_))).collect();
        for tok in &lex_errors {
            if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
                let line = if tok.span.line > 0 { tok.span.line - 1 } else { 0 };
                let col = if tok.span.col > 0 { tok.span.col - 1 } else { 0 };
                diagnostics.push(serde_json::json!({"range": {"start": {"line": line, "character": col},"end": {"line": line, "character": col + 1}},"severity": "Error","message": format!("Lex error: {}", msg)}));
            }
        }
        if !lex_errors.is_empty() { return diagnostics; }
        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(program) => {
                let mut checker = xiom_check::Checker::new();
                if let Err(errors) = checker.check_program(&program) { for err in &errors { diagnostics.push(diagnostic_from_check_error(err)); } }
            }
            Err(err) => { diagnostics.push(diagnostic_from_parse_error(&err)); }
        }
        diagnostics
    }
}

fn make_range(span: &xiom_ast::Span) -> serde_json::Value {
    let line = if span.line > 0 { span.line - 1 } else { 0 };
    let col = if span.col > 0 { span.col - 1 } else { 0 };
    serde_json::json!({"start": {"line": line, "character": col},"end": {"line": line, "character": col + 1}})
}

fn diagnostic_from_parse_error(err: &ParseError) -> serde_json::Value {
    serde_json::json!({"range": make_range(&err.span),"severity": "Error","message": format!("Parse error: {}", err.message)})
}

fn diagnostic_from_check_error(err: &CheckError) -> serde_json::Value {
    serde_json::json!({"range": make_range(&err.span),"severity": "Error","message": format!("Type error: {}", err.message)})
}

struct LspReader { stdin: io::Stdin }
impl LspReader {
    fn new() -> Self { Self { stdin: io::stdin() } }
    fn read_message(&self) -> Option<String> {
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            self.stdin.lock().read_line(&mut line).ok()?;
            let trimmed = line.trim();
            if trimmed.is_empty() { break; }
            if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") { content_length = len_str.trim().parse::<usize>().ok(); }
        }
        let len = content_length?;
        let mut buf = vec![0u8; len];
        let mut handle = self.stdin.lock();
        let mut read_total = 0;
        while read_total < len { let n = handle.read(&mut buf[read_total..]).ok()?; if n == 0 { return None; } read_total += n; }
        String::from_utf8(buf).ok()
    }
}

fn extract_word(line: &str, col: usize) -> String {
    let bytes = line.as_bytes();
    let mut start = col; let mut end = col;
    while start > 0 && is_ident_char(bytes[start - 1]) { start -= 1; }
    while end < bytes.len() && is_ident_char(bytes[end]) { end += 1; }
    if start < end { line[start..end].to_string() } else { String::new() }
}

fn word_start_pos(line: &str, col: usize) -> usize { let bytes = line.as_bytes(); let mut start = col; while start > 0 && is_ident_char(bytes[start - 1]) { start -= 1; } start }

fn is_ident_char(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'_' }

fn extract_obj_expr(line: &str, dot_pos: usize) -> String {
    let bytes = line.as_bytes();
    let mut end = dot_pos; while end > 0 && bytes[end - 1] == b' ' { end -= 1; }
    let mut start = end; while start > 0 { let c = bytes[start - 1]; if is_ident_char(c) || c == b'.' || c == b':' { start -= 1; } else { break; } }
    if start < end { line[start..end].to_string() } else { String::new() }
}
