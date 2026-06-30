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
// Symbol collection for completion
// ============================================================================

fn collect_symbols(item: &axiom_ast::TopDecl, items: &mut Vec<serde_json::Value>, prefix: &str) {
    match item {
        axiom_ast::TopDecl::Fn(f) => {
            let name = &f.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 3, // Function
                    "detail": "function",
                    "insertText": name
                }));
            }
        }
        axiom_ast::TopDecl::Type(td) => {
            let name = &td.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 23, // Struct
                    "detail": "type",
                    "insertText": name
                }));
            }
        }
        axiom_ast::TopDecl::Enum(ed) => {
            let name = &ed.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 13, // Enum
                    "detail": "enum",
                    "insertText": name
                }));
            }
        }
        axiom_ast::TopDecl::Interface(id) => {
            let name = &id.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 11, // Interface
                    "detail": "interface",
                    "insertText": name
                }));
            }
        }
        axiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                collect_symbols(inner, items, prefix);
            }
        }
        _ => {}
    }
}

fn find_definition(program: &axiom_ast::Program, name: &str) -> Option<(u64, u64)> {
    for item in &program.items {
        if let Some(pos) = find_def_in_item(item, name) {
            return Some(pos);
        }
    }
    None
}

fn find_def_in_item(item: &axiom_ast::TopDecl, name: &str) -> Option<(u64, u64)> {
    match item {
        axiom_ast::TopDecl::Fn(f) if f.name.name == name => {
            let line = if f.name.span.line > 0 { f.name.span.line as u64 - 1 } else { 0 };
            let col = if f.name.span.col > 0 { f.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        axiom_ast::TopDecl::Type(td) if td.name.name == name => {
            let line = if td.name.span.line > 0 { td.name.span.line as u64 - 1 } else { 0 };
            let col = if td.name.span.col > 0 { td.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        axiom_ast::TopDecl::Enum(ed) if ed.name.name == name => {
            let line = if ed.name.span.line > 0 { ed.name.span.line as u64 - 1 } else { 0 };
            let col = if ed.name.span.col > 0 { ed.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        axiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                if let Some(pos) = find_def_in_item(inner, name) {
                    return Some(pos);
                }
            }
            None
        }
        _ => None,
    }
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
            "hoverProvider": true,
            "completionProvider": {
                "triggerCharacters": ["."]
            },
            "definitionProvider": true
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
                        "message": "AXIOM Language Server v0.6.6"
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

            "textDocument/completion" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
                let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let mut items = Vec::new();

                let prefix = uri.as_ref().and_then(|u| {
                    let docs = backend.documents.lock().unwrap();
                    let text = docs.get(u)?;
                    let line_str = text.lines().nth(line)?;
                    Some(extract_word(line_str, character))
                }).unwrap_or_default();

                let keywords = vec![
                    "fn", "let", "var", "return", "if", "else", "elif", "while",
                    "match", "for", "in", "module", "use", "pub", "type", "enum",
                    "interface", "derive", "requires", "ensures", "invariant",
                    "async", "await", "spawn", "true", "false", "Some", "None", "Ok", "Err",
                ];
                let primitives = vec![
                    "Int", "Float64", "Bool", "Str", "Char", "Int8", "Int16", "Int32", "Int64",
                    "UInt", "UInt8", "Float32", "Option", "Result", "Vec", "Map", "Set", "Slice",
                ];

                for kw in keywords.iter().chain(primitives.iter()) {
                    if kw.starts_with(&prefix) || prefix.is_empty() {
                        items.push(serde_json::json!({
                            "label": kw,
                            "kind": 14, // Keyword
                            "insertText": kw
                        }));
                    }
                }

                if let Some(ref u) = uri {
                    let docs = backend.documents.lock().unwrap();
                    if let Some(text) = docs.get(u) {
                        let mut lexer = axiom_lexer::Lexer::new(text);
                        let tokens = lexer.tokenize();
                        let mut parser = axiom_parser::Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            for item in &program.items {
                                collect_symbols(item, &mut items, &prefix);
                            }
                        }
                    }
                }

                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": items
                }));
            }

            "textDocument/definition" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
                let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let mut location = None;

                if let Some(ref u) = uri {
                    let word = {
                        let docs = backend.documents.lock().unwrap();
                        if let Some(text) = docs.get(u) {
                            let line_str = text.lines().nth(line).unwrap_or("");
                            extract_word(line_str, character)
                        } else {
                            String::new()
                        }
                    };

                    if !word.is_empty() {
                        let docs = backend.documents.lock().unwrap();
                        if let Some(text) = docs.get(u) {
                            let mut lexer = axiom_lexer::Lexer::new(text);
                            let tokens = lexer.tokenize();
                            let mut parser = axiom_parser::Parser::new(tokens);
                            if let Ok(program) = parser.parse_program() {
                                if let Some(pos) = find_definition(&program, &word) {
                                    location = Some(serde_json::json!({
                                        "uri": u,
                                        "range": {
                                            "start": { "line": pos.0, "character": pos.1 },
                                            "end": { "line": pos.0, "character": pos.1 + word.len() as u64 }
                                        }
                                    }));
                                }
                            }
                        }
                    }
                }

                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": location
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
