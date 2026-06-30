use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::sync::Arc;
use std::sync::Mutex;

use axiom_check::CheckError;
use axiom_lexer::Lexer;
use axiom_parser::{ParseError, Parser};

// ============================================================================
// LSP Backend
// ============================================================================

struct Backend {
    documents: Arc<Mutex<HashMap<String, String>>>,
}

impl Backend {
    fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn publish_diagnostics(&self, uri: &str) -> Vec<serde_json::Value> {
        let text = {
            let docs = self.documents.lock().unwrap();
            docs.get(uri).cloned()
        };
        let text = match text {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();

        let mut lexer = Lexer::new(&text);
        let tokens = lexer.tokenize();

        let lex_errors: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, axiom_lexer::TokenKind::Error(_)))
            .collect();
        for tok in &lex_errors {
            if let axiom_lexer::TokenKind::Error(msg) = &tok.kind {
                let line = if tok.span.line > 0 { tok.span.line - 1 } else { 0 };
                let col = if tok.span.col > 0 { tok.span.col - 1 } else { 0 };
                diagnostics.push(serde_json::json!({
                    "range": {
                        "start": { "line": line, "character": col },
                        "end": { "line": line, "character": col + 1 }
                    },
                    "severity": "Error",
                    "message": format!("Lex error: {}", msg)
                }));
            }
        }

        if !lex_errors.is_empty() {
            return diagnostics;
        }

        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(program) => {
                let mut checker = axiom_check::Checker::new();
                if let Err(errors) = checker.check_program(&program) {
                    for err in &errors {
                        diagnostics.push(diagnostic_from_check_error(err));
                    }
                }
            }
            Err(err) => {
                diagnostics.push(diagnostic_from_parse_error(&err));
            }
        }

        diagnostics
    }
}

// ============================================================================
// Diagnostics helpers
// ============================================================================

fn make_range(span: &axiom_ast::Span) -> serde_json::Value {
    let line = if span.line > 0 { span.line - 1 } else { 0 };
    let col = if span.col > 0 { span.col - 1 } else { 0 };
    serde_json::json!({
        "start": { "line": line, "character": col },
        "end": { "line": line, "character": col + 1 }
    })
}

fn diagnostic_from_parse_error(err: &ParseError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(&err.span),
        "severity": "Error",
        "message": format!("Parse error: {}", err.message)
    })
}

fn diagnostic_from_check_error(err: &CheckError) -> serde_json::Value {
    serde_json::json!({
        "range": make_range(&err.span),
        "severity": "Error",
        "message": format!("Type error: {}", err.message)
    })
}

// ============================================================================
// Content-Length header parsing
// ============================================================================

struct LspReader {
    stdin: io::Stdin,
}

impl LspReader {
    fn new() -> Self {
        Self {
            stdin: io::stdin(),
        }
    }

    fn read_message(&self) -> Option<String> {
        let mut content_length: Option<usize> = None;

        loop {
            let mut line = String::new();
            self.stdin.lock().read_line(&mut line).ok()?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
                content_length = len_str.trim().parse::<usize>().ok();
            }
        }

        let len = content_length?;
        let mut buf = vec![0u8; len];
        let mut handle = self.stdin.lock();
        let mut read_total = 0;
        while read_total < len {
            let n = handle.read(&mut buf[read_total..]).ok()?;
            if n == 0 {
                return None;
            }
            read_total += n;
        }
        String::from_utf8(buf).ok()
    }
}

// ============================================================================
// Context helpers
// ============================================================================

fn extract_word(line: &str, col: usize) -> String {
    let bytes = line.as_bytes();
    let mut start = col;
    let mut end = col;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }
    if start < end {
        line[start..end].to_string()
    } else {
        String::new()
    }
}

fn is_ident_char(c: u8) -> bool {
    (c >= b'a' && c <= b'z')
        || (c >= b'A' && c <= b'Z')
        || (c >= b'0' && c <= b'9')
        || c == b'_'
}

// ============================================================================
// LSP message I/O
// ============================================================================

fn write_lsp_message(body: &serde_json::Value) {
    let body_str = serde_json::to_string(body).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body_str.len());
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(header.as_bytes());
    let _ = stdout.write_all(body_str.as_bytes());
    let _ = stdout.flush();
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let backend = Arc::new(Backend::new());
    let reader = LspReader::new();

    let init_result = serde_json::json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true
        }
    });

    while let Some(raw) = reader.read_message() {
        let msg: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let method = match msg["method"].as_str() {
            Some(m) => m.to_string(),
            None => continue,
        };

        match method.as_str() {
            "initialize" => {
                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": init_result
                }));
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "window/logMessage",
                    "params": {
                        "type": 3,
                        "message": "AXIOM Language Server v0.6.2"
                    }
                }));
            }

            "shutdown" => {
                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": null
                }));
                break;
            }

            "textDocument/didOpen" => {
                let params = &msg["params"];
                if let (Some(uri), Some(text)) = (
                    params["textDocument"]["uri"].as_str(),
                    params["textDocument"]["text"].as_str(),
                ) {
                    let uri = uri.to_string();
                    {
                        let mut docs = backend.documents.lock().unwrap();
                        docs.insert(uri.clone(), text.to_string());
                    }
                    publish_diagnostics(&backend, &uri);
                }
            }

            "textDocument/didChange" => {
                let params = &msg["params"];
                if let Some(uri) = params["textDocument"]["uri"].as_str() {
                    let uri = uri.to_string();
                    if let Some(changes) = params["contentChanges"].as_array() {
                        if let Some(last) = changes.last() {
                            if let Some(text) = last["text"].as_str() {
                                {
                                    let mut docs = backend.documents.lock().unwrap();
                                    docs.insert(uri.clone(), text.to_string());
                                }
                                publish_diagnostics(&backend, &uri);
                            }
                        }
                    }
                }
            }

            "textDocument/hover" => {
                let uri = msg["params"]["textDocument"]["uri"]
                    .as_str()
                    .map(|s| s.to_string());
                let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let character = msg["params"]["position"]["character"]
                    .as_u64()
                    .unwrap_or(0) as usize;

                let hover = uri.and_then(|u| {
                    let docs = backend.documents.lock().unwrap();
                    let text = docs.get(&u)?.clone();
                    let line_str = text.lines().nth(line)?;
                    let word = extract_word(line_str, character);
                    if word.is_empty() {
                        None
                    } else {
                        Some(serde_json::json!({
                            "contents": {
                                "kind": "markdown",
                                "value": format!("AXIOM identifier: `{}`", word)
                            }
                        }))
                    }
                });

                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": hover
                }));
            }

            _ => {}
        }
    }
}

fn publish_diagnostics(backend: &Backend, uri: &str) {
    let diagnostics = backend.publish_diagnostics(uri);
    write_lsp_message(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    }));
}
