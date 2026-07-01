// AXIOM — Language Server
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::collections::HashMap;
use std::env;
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
// Type helpers for dot-completion / signature help
// ============================================================================

fn type_to_string(ty: &axiom_ast::Type) -> String {
    match ty {
        axiom_ast::Type::Named(ident, args) => {
            if args.is_empty() {
                ident.name.clone()
            } else {
                let args_str: Vec<String> = args.iter().map(type_to_string).collect();
                format!("{}[{}]", ident.name, args_str.join(", "))
            }
        }
        axiom_ast::Type::Ref(t) => format!("&{}", type_to_string(t)),
        axiom_ast::Type::MutRef(t) => format!("&mut {}", type_to_string(t)),
        axiom_ast::Type::Option(t) => format!("Option[{}]", type_to_string(t)),
        axiom_ast::Type::Result(t, e) => format!("Result[{}, {}]", type_to_string(t), type_to_string(e)),
        axiom_ast::Type::Vec(t) => format!("Vec[{}]", type_to_string(t)),
        axiom_ast::Type::Slice(t) => format!("Slice[{}]", type_to_string(t)),
        axiom_ast::Type::Map(k, v) => format!("Map[{}, {}]", type_to_string(k), type_to_string(v)),
        axiom_ast::Type::Set(t) => format!("Set[{}]", type_to_string(t)),
        axiom_ast::Type::Tuple(types) => {
            let items: Vec<String> = types.iter().map(type_to_string).collect();
            format!("({})", items.join(", "))
        }
        axiom_ast::Type::Ptr(t) => format!("*{}", type_to_string(t)),
        axiom_ast::Type::Array(_, _) => "Array".to_string(),
    }
}

fn infer_type_from_expr(expr: &axiom_ast::Expr) -> Option<String> {
    match expr {
        axiom_ast::Expr::Struct(ident, _, _) => Some(ident.name.clone()),
        axiom_ast::Expr::Some(_, _) => Some("Option".to_string()),
        axiom_ast::Expr::None(_) => Some("Option".to_string()),
        axiom_ast::Expr::Ok(_, _) => Some("Result".to_string()),
        axiom_ast::Expr::Err(_, _) => Some("Result".to_string()),
        axiom_ast::Expr::Int(_, _) => Some("Int".to_string()),
        axiom_ast::Expr::Float(_, _) => Some("Float64".to_string()),
        axiom_ast::Expr::Str(_, _) => Some("Str".to_string()),
        axiom_ast::Expr::Bool(_, _) => Some("Bool".to_string()),
        axiom_ast::Expr::Char(_, _) => Some("Char".to_string()),
        _ => None,
    }
}

fn find_variable_type_in_program(program: &axiom_ast::Program, var_name: &str) -> Option<String> {
    for item in &program.items {
        if let Some(ty) = find_variable_type_in_item(item, var_name) {
            return Some(ty);
        }
    }
    None
}

fn find_variable_type_in_item(item: &axiom_ast::TopDecl, var_name: &str) -> Option<String> {
    match item {
        axiom_ast::TopDecl::Fn(f) => {
            if let Some(body) = &f.body {
                for soe in &body.stmts {
                    if let axiom_ast::StmtOrExpr::Stmt(stmt) = soe {
                        match stmt {
                            axiom_ast::Stmt::Let(ident, ty, expr, _)
                            | axiom_ast::Stmt::Var(ident, ty, expr, _) => {
                                if ident.name == var_name {
                                    if let Some(t) = ty {
                                        return Some(type_to_string(t));
                                    }
                                    return infer_type_from_expr(expr);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            None
        }
        axiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                if let Some(ty) = find_variable_type_in_item(inner, var_name) {
                    return Some(ty);
                }
            }
            None
        }
        _ => None,
    }
}

fn find_struct_fields_in_program(program: &axiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for item in &program.items {
        if let axiom_ast::TopDecl::Type(td) = item {
            if td.name.name == type_name {
                for field in &td.fields {
                    fields.push((field.name.name.clone(), type_to_string(&field.ty)));
                }
            }
        }
    }
    fields
}

fn find_methods_in_program(program: &axiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut methods = Vec::new();
    for item in &program.items {
        if let axiom_ast::TopDecl::Fn(f) = item {
            if let Some(receiver) = &f.receiver {
                if receiver.name == type_name {
                    methods.push((f.name.name.clone(), format_fn_signature(f)));
                }
            }
        }
    }
    methods
}

fn format_fn_signature(f: &axiom_ast::FnDecl) -> String {
    let mut sig = String::new();
    if f.is_pub { sig.push_str("pub "); }
    if f.is_async { sig.push_str("async "); }
    sig.push_str("fn ");
    if let Some(receiver) = &f.receiver {
        sig.push_str(&receiver.name);
        sig.push('.');
    }
    sig.push_str(&f.name.name);
    sig.push('(');
    let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name.name, type_to_string(&p.ty))).collect();
    sig.push_str(&params.join(", "));
    sig.push(')');
    if let Some(rt) = &f.return_type {
        sig.push_str(" -> ");
        sig.push_str(&type_to_string(rt));
    }
    sig
}

fn find_function_signature(program: &axiom_ast::Program, fn_name: &str) -> Option<String> {
    for item in &program.items {
        if let axiom_ast::TopDecl::Fn(f) = item {
            if f.name.name == fn_name {
                return Some(format_fn_signature(f));
            }
        }
    }
    None
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
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return;
    }

    let backend = Arc::new(Backend::new());
    let reader = LspReader::new();

    let init_result = serde_json::json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true,
            "completionProvider": {
                "triggerCharacters": ["."]
            },
            "definitionProvider": true,
            "signatureHelpProvider": {
                "triggerCharacters": ["(", ","]
            }
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

                let (prefix, is_dot_completion, obj_name) = uri.as_ref().and_then(|u| {
                    let docs = backend.documents.lock().unwrap();
                    let text = docs.get(u)?;
                    let line_str = text.lines().nth(line)?;
                    let bytes = line_str.as_bytes();
                    if character > 0 && character <= bytes.len() && bytes[character - 1] == b'.' {
                        let dot_pos = character - 1;
                        let mut start = dot_pos;
                        while start > 0 && is_ident_char(bytes[start - 1]) {
                            start -= 1;
                        }
                        Some((String::new(), true, line_str[start..dot_pos].to_string()))
                    } else {
                        Some((extract_word(line_str, character), false, String::new()))
                    }
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

                if !is_dot_completion {
                    for kw in keywords.iter().chain(primitives.iter()) {
                        if kw.starts_with(&prefix) || prefix.is_empty() {
                            items.push(serde_json::json!({
                                "label": kw,
                                "kind": 14, // Keyword
                                "insertText": kw
                            }));
                        }
                    }
                }

                if let Some(ref u) = uri {
                    let docs = backend.documents.lock().unwrap();
                    if let Some(text) = docs.get(u) {
                        let mut lexer = axiom_lexer::Lexer::new(text);
                        let tokens = lexer.tokenize();
                        let mut parser = axiom_parser::Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            if !is_dot_completion {
                                for item in &program.items {
                                    collect_symbols(item, &mut items, &prefix);
                                }
                            }

                            if is_dot_completion && !obj_name.is_empty() {
                                if let Some(type_name) = find_variable_type_in_program(&program, &obj_name) {
                                    let fields = find_struct_fields_in_program(&program, &type_name);
                                    for (field_name, field_type) in &fields {
                                        items.push(serde_json::json!({
                                            "label": field_name,
                                            "kind": 5,
                                            "detail": field_type,
                                            "insertText": field_name
                                        }));
                                    }
                                    let methods = find_methods_in_program(&program, &type_name);
                                    for (method_name, sig) in &methods {
                                        items.push(serde_json::json!({
                                            "label": method_name,
                                            "kind": 2,
                                            "detail": sig,
                                            "insertText": format!("{}(", method_name)
                                        }));
                                    }
                                }
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

            "textDocument/signatureHelp" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
                let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let mut signatures = Vec::new();

                let docs = backend.documents.lock().unwrap();
                if let Some(text) = docs.get(uri) {
                    let line_str = text.lines().nth(line).unwrap_or("");
                    let before_cursor = &line_str[..character.min(line_str.len())];
                    if let Some(paren_pos) = before_cursor.rfind('(') {
                        let before_paren = &before_cursor[..paren_pos];
                        let fn_name = before_paren.split_whitespace()
                            .last()
                            .unwrap_or("")
                            .trim();
                        if !fn_name.is_empty() {
                            let mut lexer = axiom_lexer::Lexer::new(text);
                            let tokens = lexer.tokenize();
                            let mut parser = axiom_parser::Parser::new(tokens);
                            if let Ok(program) = parser.parse_program() {
                                if let Some(sig) = find_function_signature(&program, fn_name) {
                                    signatures.push(serde_json::json!({
                                        "label": sig,
                                        "documentation": ""
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
                    "result": {
                        "signatures": signatures,
                        "activeSignature": 0,
                        "activeParameter": 0
                    }
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

fn print_usage() {
    eprintln!("AXIOM Language Server v0.10.1");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  axiom lsp");
    eprintln!();
    eprintln!("The AXIOM Language Server provides diagnostics, hover, completion,");
    eprintln!("and go-to-definition for .ax files. Launch from editor configuration.");
    eprintln!();
    eprintln!("VS Code: editors/vscode/package.json");
}
