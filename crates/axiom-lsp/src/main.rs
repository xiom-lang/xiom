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

fn word_start_pos(line: &str, col: usize) -> usize {
    let bytes = line.as_bytes();
    let mut start = col;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    start
}

fn is_ident_char(c: u8) -> bool {
    (c >= b'a' && c <= b'z')
        || (c >= b'A' && c <= b'Z')
        || (c >= b'0' && c <= b'9')
        || c == b'_'
}

fn extract_obj_expr(line: &str, dot_pos: usize) -> String {
    let bytes = line.as_bytes();
    let mut end = dot_pos;
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 {
        let c = bytes[start - 1];
        if is_ident_char(c) || c == b'.' || c == b':' {
            start -= 1;
        } else {
            break;
        }
    }
    if start < end {
        line[start..end].to_string()
    } else {
        String::new()
    }
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
        axiom_ast::TopDecl::Const(cd) => {
            let name = &cd.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 14, // Constant
                    "detail": "const",
                    "insertText": name
                }));
            }
        }
        _ => {}
    }
}

fn collect_document_symbols(
    item: &axiom_ast::TopDecl,
    symbols: &mut Vec<serde_json::Value>,
) {
    match item {
        axiom_ast::TopDecl::Fn(f) => {
            let line = if f.name.span.line > 0 { f.name.span.line as u64 - 1 } else { 0 };
            let col = if f.name.span.col > 0 { f.name.span.col as u64 - 1 } else { 0 };
            let sig = format_fn_signature(f);
            symbols.push(serde_json::json!({
                "name": f.name.name,
                "detail": sig,
                "kind": 12, // Function
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + f.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + f.name.name.len() as u64 }
                }
            }));
        }
        axiom_ast::TopDecl::Type(td) => {
            let line = if td.name.span.line > 0 { td.name.span.line as u64 - 1 } else { 0 };
            let col = if td.name.span.col > 0 { td.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for field in &td.fields {
                let fline = if field.name.span.line > 0 { field.name.span.line as u64 - 1 } else { 0 };
                let fcol = if field.name.span.col > 0 { field.name.span.col as u64 - 1 } else { 0 };
                children.push(serde_json::json!({
                    "name": field.name.name,
                    "detail": type_to_string(&field.ty),
                    "kind": 8, // Field
                    "range": {
                        "start": { "line": fline, "character": fcol },
                        "end": { "line": fline, "character": fcol + field.name.name.len() as u64 }
                    },
                    "selectionRange": {
                        "start": { "line": fline, "character": fcol },
                        "end": { "line": fline, "character": fcol + field.name.name.len() as u64 }
                    }
                }));
            }
            symbols.push(serde_json::json!({
                "name": td.name.name,
                "detail": "type",
                "kind": 23, // Struct
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + td.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + td.name.name.len() as u64 }
                },
                "children": children
            }));
        }
        axiom_ast::TopDecl::Enum(ed) => {
            let line = if ed.name.span.line > 0 { ed.name.span.line as u64 - 1 } else { 0 };
            let col = if ed.name.span.col > 0 { ed.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for variant in &ed.variants {
                let vline = if variant.name.span.line > 0 { variant.name.span.line as u64 - 1 } else { 0 };
                let vcol = if variant.name.span.col > 0 { variant.name.span.col as u64 - 1 } else { 0 };
                children.push(serde_json::json!({
                    "name": variant.name.name,
                    "kind": 22, // EnumMember
                    "range": {
                        "start": { "line": vline, "character": vcol },
                        "end": { "line": vline, "character": vcol + variant.name.name.len() as u64 }
                    },
                    "selectionRange": {
                        "start": { "line": vline, "character": vcol },
                        "end": { "line": vline, "character": vcol + variant.name.name.len() as u64 }
                    }
                }));
            }
            symbols.push(serde_json::json!({
                "name": ed.name.name,
                "detail": "enum",
                "kind": 13, // Enum
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + ed.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + ed.name.name.len() as u64 }
                },
                "children": children
            }));
        }
        axiom_ast::TopDecl::Interface(id) => {
            let line = if id.name.span.line > 0 { id.name.span.line as u64 - 1 } else { 0 };
            let col = if id.name.span.col > 0 { id.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for member in &id.members {
                match member {
                    axiom_ast::InterfaceMember::Field(fd) => {
                        let fline = if fd.name.span.line > 0 { fd.name.span.line as u64 - 1 } else { 0 };
                        let fcol = if fd.name.span.col > 0 { fd.name.span.col as u64 - 1 } else { 0 };
                        children.push(serde_json::json!({
                            "name": fd.name.name,
                            "detail": type_to_string(&fd.ty),
                            "kind": 8, // Field
                            "range": {
                                "start": { "line": fline, "character": fcol },
                                "end": { "line": fline, "character": fcol + fd.name.name.len() as u64 }
                            },
                            "selectionRange": {
                                "start": { "line": fline, "character": fcol },
                                "end": { "line": fline, "character": fcol + fd.name.name.len() as u64 }
                            }
                        }));
                    }
                    axiom_ast::InterfaceMember::FnSignature(fs) => {
                        let fline = if fs.name.span.line > 0 { fs.name.span.line as u64 - 1 } else { 0 };
                        let fcol = if fs.name.span.col > 0 { fs.name.span.col as u64 - 1 } else { 0 };
                        let sig = format_fn_signature(fs);
                        children.push(serde_json::json!({
                            "name": fs.name.name,
                            "detail": sig,
                            "kind": 6, // Method
                            "range": {
                                "start": { "line": fline, "character": fcol },
                                "end": { "line": fline, "character": fcol + fs.name.name.len() as u64 }
                            },
                            "selectionRange": {
                                "start": { "line": fline, "character": fcol },
                                "end": { "line": fline, "character": fcol + fs.name.name.len() as u64 }
                            }
                        }));
                    }
                }
            }
            symbols.push(serde_json::json!({
                "name": id.name.name,
                "detail": "interface",
                "kind": 11, // Interface
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + id.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + id.name.name.len() as u64 }
                },
                "children": children
            }));
        }
        axiom_ast::TopDecl::Module(m) => {
            let line = if m.name.span.line > 0 { m.name.span.line as u64 - 1 } else { 0 };
            let col = if m.name.span.col > 0 { m.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for inner in &m.items {
                collect_document_symbols(inner, &mut children);
            }
            symbols.push(serde_json::json!({
                "name": m.name.name,
                "detail": "module",
                "kind": 2, // Module
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + m.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + m.name.name.len() as u64 }
                },
                "children": children
            }));
        }
        axiom_ast::TopDecl::Const(cd) => {
            let line = if cd.name.span.line > 0 { cd.name.span.line as u64 - 1 } else { 0 };
            let col = if cd.name.span.col > 0 { cd.name.span.col as u64 - 1 } else { 0 };
            symbols.push(serde_json::json!({
                "name": cd.name.name,
                "detail": format!("const {}: {}", cd.name.name, type_to_string(&cd.ty)),
                "kind": 14, // Constant
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + cd.name.name.len() as u64 }
                },
                "selectionRange": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + cd.name.name.len() as u64 }
                }
            }));
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
        axiom_ast::TopDecl::Interface(id) if id.name.name == name => {
            let line = if id.name.span.line > 0 { id.name.span.line as u64 - 1 } else { 0 };
            let col = if id.name.span.col > 0 { id.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        axiom_ast::TopDecl::Module(m) => {
            if m.name.name == name {
                let line = if m.name.span.line > 0 { m.name.span.line as u64 - 1 } else { 0 };
                let col = if m.name.span.col > 0 { m.name.span.col as u64 - 1 } else { 0 };
                return Some((line, col));
            }
            for inner in &m.items {
                if let Some(pos) = find_def_in_item(inner, name) {
                    return Some(pos);
                }
            }
            None
        }
        axiom_ast::TopDecl::Const(cd) if cd.name.name == name => {
            let line = if cd.name.span.line > 0 { cd.name.span.line as u64 - 1 } else { 0 };
            let col = if cd.name.span.col > 0 { cd.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        _ => None,
    }
}

// ============================================================================
// Type helpers for hover / completion / signature help
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
        axiom_ast::Type::Fn(params, ret) => {
            let params_str: Vec<String> = params.iter().map(type_to_string).collect();
            format!("fn({}) -> {}", params_str.join(", "), type_to_string(ret))
        }
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
        axiom_ast::Expr::Ident(ident) => {
            // For simple ident references, we return the name itself
            // This is used for inference from binding patterns like let x = some_var;
            Some(ident.name.clone())
        }
        _ => None,
    }
}

// ============================================================================
// Recursive variable type lookup (traverses nested blocks)
// ============================================================================

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
            for param in &f.params {
                if param.name.name == var_name {
                    return Some(type_to_string(&param.ty));
                }
            }
            if let Some(body) = &f.body {
                return find_variable_type_in_block(body, var_name);
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

fn find_variable_type_in_block(block: &axiom_ast::Block, var_name: &str) -> Option<String> {
    for soe in &block.stmts {
        match soe {
            axiom_ast::StmtOrExpr::Stmt(stmt) => {
                if let Some(ty) = find_variable_type_in_stmt(stmt, var_name) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_variable_type_in_stmt(stmt: &axiom_ast::Stmt, var_name: &str) -> Option<String> {
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
        axiom_ast::Stmt::If(_, then_block, elifs, else_block, _) => {
            if let Some(ty) = find_variable_type_in_block(then_block, var_name) {
                return Some(ty);
            }
            for (_, elif_block) in elifs {
                if let Some(ty) = find_variable_type_in_block(elif_block, var_name) {
                    return Some(ty);
                }
            }
            if let Some(eb) = else_block {
                if let Some(ty) = find_variable_type_in_block(eb, var_name) {
                    return Some(ty);
                }
            }
        }
        axiom_ast::Stmt::While(_, body, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) {
                return Some(ty);
            }
        }
        axiom_ast::Stmt::Match(_, arms, _) => {
            for arm in arms {
                match &arm.body {
                    axiom_ast::MatchBody::Block(b) => {
                        if let Some(ty) = find_variable_type_in_block(b, var_name) {
                            return Some(ty);
                        }
                    }
                    _ => {}
                }
            }
        }
        axiom_ast::Stmt::For(_, _, body, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) {
                return Some(ty);
            }
        }
        axiom_ast::Stmt::Spawn(body, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) {
                return Some(ty);
            }
        }
        _ => {}
    }
    None
}

// ============================================================================
// Struct / method / enum / interface / module lookup helpers
// ============================================================================

fn find_struct_fields_in_program(program: &axiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for item in &program.items {
        recurse_find_struct_fields(item, type_name, &mut fields);
    }
    fields
}

fn recurse_find_struct_fields(item: &axiom_ast::TopDecl, type_name: &str, fields: &mut Vec<(String, String)>) {
    match item {
        axiom_ast::TopDecl::Type(td) => {
            if td.name.name == type_name {
                for field in &td.fields {
                    fields.push((field.name.name.clone(), type_to_string(&field.ty)));
                }
            }
        }
        axiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                recurse_find_struct_fields(inner, type_name, fields);
            }
        }
        _ => {}
    }
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

fn find_interface_methods_for_type(program: &axiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
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
    // Also check interfaces that this type might implement
    // (simplified: just include interface methods if the interface name matches)
    for item in &program.items {
        if let axiom_ast::TopDecl::Interface(id) = item {
            for member in &id.members {
                if let axiom_ast::InterfaceMember::FnSignature(fs) = member {
                    methods.push((format!("{}.{}", id.name.name, fs.name.name), format_fn_signature(fs)));
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
    if !f.generics.is_empty() {
        let gs: Vec<String> = f.generics.iter().map(|g| {
            if g.bounds.is_empty() {
                g.name.name.clone()
            } else {
                let bs: Vec<String> = g.bounds.iter().map(|b| b.name.clone()).collect();
                format!("{}: {}", g.name.name, bs.join(" + "))
            }
        }).collect();
        sig.push('[');
        sig.push_str(&gs.join(", "));
        sig.push(']');
    }
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

fn collect_enum_variants_for_type(program: &axiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut variants = Vec::new();
    for item in &program.items {
        if let axiom_ast::TopDecl::Enum(ed) = item {
            if ed.name.name == type_name {
                for variant in &ed.variants {
                    if variant.fields.is_empty() {
                        variants.push((format!("{}::{}", type_name, variant.name.name), String::new()));
                    } else {
                        let field_types: Vec<String> = variant.fields.iter()
                            .map(|f| type_to_string(&f.ty))
                            .collect();
                        variants.push((
                            format!("{}::{}", type_name, variant.name.name),
                            format!("({})", field_types.join(", ")),
                        ));
                    }
                }
            }
        }
    }
    variants
}

fn collect_module_members(
    program: &axiom_ast::Program,
    module_name: &str,
    prefix: &str,
    items: &mut Vec<serde_json::Value>,
) {
    for item in &program.items {
        if let axiom_ast::TopDecl::Module(m) = item {
            if m.name.name == module_name {
                collect_module_item_completions(&m.items, prefix, items);
                return;
            }
        }
    }
}

fn collect_module_item_completions(
    items: &[axiom_ast::TopDecl],
    prefix: &str,
    out: &mut Vec<serde_json::Value>,
) {
    for item in items {
        match item {
            axiom_ast::TopDecl::Fn(f) => {
                let name = &f.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 3,
                        "detail": format_fn_signature(f),
                        "insertText": format!("{}(", name)
                    }));
                }
            }
            axiom_ast::TopDecl::Type(td) => {
                let name = &td.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 23,
                        "detail": "type",
                        "insertText": name
                    }));
                }
            }
            axiom_ast::TopDecl::Enum(ed) => {
                let name = &ed.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 13,
                        "detail": "enum",
                        "insertText": name
                    }));
                }
            }
            axiom_ast::TopDecl::Interface(id) => {
                let name = &id.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 11,
                        "detail": "interface",
                        "insertText": name
                    }));
                }
            }
            axiom_ast::TopDecl::Module(m) => {
                let name = &m.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 2,
                        "detail": "module",
                        "insertText": name
                    }));
                }
            }
            axiom_ast::TopDecl::Const(cd) => {
                let name = &cd.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name,
                        "kind": 14,
                        "detail": format!("const {}: {}", cd.name.name, type_to_string(&cd.ty)),
                        "insertText": name
                    }));
                }
            }
            _ => {}
        }
    }
}

fn resolve_obj_type_text(program: &axiom_ast::Program, obj_expr: &str) -> Option<String> {
    let parts: Vec<&str> = obj_expr.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let first = parts[0];

    // Try as a variable name
    if let Some(ty) = find_variable_type_in_program(program, first) {
        let mut current_type = ty;
        for part in &parts[1..] {
            let fields = find_struct_fields_in_program(program, &current_type);
            let mut found = false;
            for (fname, ftype) in &fields {
                if fname == part {
                    current_type = ftype.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }
        return Some(current_type);
    }

    // Try as a type name (for static-like access or enum variants)
    if parts.len() == 1 {
        for item in &program.items {
            match item {
                axiom_ast::TopDecl::Type(td) if td.name.name == first => {
                    return Some(first.to_string());
                }
                axiom_ast::TopDecl::Enum(ed) if ed.name.name == first => {
                    return Some(first.to_string());
                }
                _ => {}
            }
        }
    }

    // Try as a module-qualified path
    if let Some(ty) = resolve_module_qualified_type(program, &parts) {
        return Some(ty);
    }

    None
}

fn resolve_module_qualified_type(program: &axiom_ast::Program, parts: &[&str]) -> Option<String> {
    if parts.len() < 2 {
        return None;
    }
    let module_name = parts[0];
    for item in &program.items {
        if let axiom_ast::TopDecl::Module(m) = item {
            if m.name.name == module_name {
                // Try to walk through the module hierarchy
                let mut current_items = &m.items;
                for i in 1..parts.len() - 1 {
                    let segment = parts[i];
                    let mut found = false;
                    for inner in current_items.iter() {
                        if let axiom_ast::TopDecl::Module(sub) = inner {
                            if sub.name.name == segment {
                                current_items = &sub.items;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        // Could be a type/function in a module
                        // For simplicity, just try direct lookup
                        return None;
                    }
                }
                let last = parts[parts.len() - 1];
                for inner in current_items.iter() {
                    match inner {
                        axiom_ast::TopDecl::Type(td) if td.name.name == last => {
                            return Some(last.to_string());
                        }
                        axiom_ast::TopDecl::Enum(ed) if ed.name.name == last => {
                            return Some(last.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

fn find_enum_variant_info(program: &axiom_ast::Program, variant_name: &str) -> Option<(String, String)> {
    for item in &program.items {
        if let axiom_ast::TopDecl::Enum(ed) = item {
            for variant in &ed.variants {
                if variant.name.name == variant_name {
                    let mut detail = String::new();
                    if !variant.fields.is_empty() {
                        let fds: Vec<String> = variant.fields.iter()
                            .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                            .collect();
                        detail = format!("({})", fds.join(", "));
                    }
                    return Some((ed.name.name.clone(), detail));
                }
            }
        }
    }
    None
}

// ============================================================================
// Text editing utility for incremental sync
// ============================================================================

fn apply_text_edit(text: &str, start_line: usize, start_char: usize, end_line: usize, end_char: usize, new_text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();

    let start_offset = lines.iter()
        .take(start_line)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        .min(text.len())
        + start_char.min(lines.get(start_line).map_or(0, |l| l.len()));

    let end_offset = lines.iter()
        .take(end_line)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        .min(text.len())
        + end_char.min(lines.get(end_line).map_or(0, |l| l.len()));

    let start_offset = start_offset.min(text.len());
    let end_offset = end_offset.max(start_offset).min(text.len());

    let mut result = String::with_capacity(text.len() + new_text.len().saturating_sub(end_offset - start_offset));
    result.push_str(&text[..start_offset]);
    result.push_str(new_text);
    result.push_str(&text[end_offset..]);
    result
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
            "textDocumentSync": {
                "openClose": true,
                "change": 2
            },
            "hoverProvider": true,
            "completionProvider": {
                "triggerCharacters": [".", ":"]
            },
            "definitionProvider": true,
            "signatureHelpProvider": {
                "triggerCharacters": ["(", ","]
            },
            "documentSymbolProvider": true
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

            "initialized" => {
                // No-op, but required by protocol
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
                        {
                            let mut docs = backend.documents.lock().unwrap();
                            let text = docs.entry(uri.clone()).or_default();
                            for change in changes {
                                if change.get("range").and_then(|r| r.as_object()).is_some() {
                                    let start_line = change["range"]["start"]["line"].as_u64().unwrap_or(0) as usize;
                                    let start_char = change["range"]["start"]["character"].as_u64().unwrap_or(0) as usize;
                                    let end_line = change["range"]["end"]["line"].as_u64().unwrap_or(0) as usize;
                                    let end_char = change["range"]["end"]["character"].as_u64().unwrap_or(0) as usize;
                                    let new_text = change["text"].as_str().unwrap_or("");
                                    let updated = apply_text_edit(text, start_line, start_char, end_line, end_char, new_text);
                                    *text = updated;
                                } else if let Some(text_str) = change["text"].as_str() {
                                    *text = text_str.to_string();
                                }
                            }
                        }
                        publish_diagnostics(&backend, &uri);
                    }
                }
            }

            "textDocument/didClose" => {
                let params = &msg["params"];
                if let Some(uri) = params["textDocument"]["uri"].as_str() {
                    let mut docs = backend.documents.lock().unwrap();
                    docs.remove(uri);
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
                    drop(docs);
                    let line_str = text.lines().nth(line)?;
                    let word = extract_word(line_str, character);
                    if word.is_empty() {
                        return None;
                    }

                    let bytes = line_str.as_bytes();
                    let wstart = word_start_pos(line_str, character);

                    // Check if preceded by a dot (field/method access)
                    let is_field_access = wstart > 0
                        && wstart <= bytes.len()
                        && bytes[wstart - 1] == b'.';

                    if is_field_access {
                        let dot_pos = wstart - 1;
                        let obj_expr = extract_obj_expr(line_str, dot_pos);

                        let mut lexer = Lexer::new(&text);
                        let tokens = lexer.tokenize();
                        let mut parser = Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            if let Some(obj_type) = resolve_obj_type_text(&program, &obj_expr) {
                                // Check struct fields
                                let fields = find_struct_fields_in_program(&program, &obj_type);
                                for (fname, ftype) in &fields {
                                    if fname == &word {
                                        return Some(serde_json::json!({
                                            "contents": {
                                                "kind": "markdown",
                                                "value": format!("**field** `{}`\n```axiom\n{}: {}\n```", fname, fname, ftype)
                                            }
                                        }));
                                    }
                                }
                                // Check methods
                                let methods = find_methods_in_program(&program, &obj_type);
                                for (mname, sig) in &methods {
                                    if mname == &word {
                                        return Some(serde_json::json!({
                                            "contents": {
                                                "kind": "markdown",
                                                "value": format!("**method**\n```axiom\n{}\n```", sig)
                                            }
                                        }));
                                    }
                                }
                                // Check interface methods
                                let iface_methods = find_interface_methods_for_type(&program, &obj_type);
                                for (mname, sig) in &iface_methods {
                                    if mname == &word {
                                        return Some(serde_json::json!({
                                            "contents": {
                                                "kind": "markdown",
                                                "value": format!("**method**\n```axiom\n{}\n```", sig)
                                            }
                                        }));
                                    }
                                }
                            }
                        }
                        // Fallback for field access
                        Some(serde_json::json!({
                            "contents": {
                                "kind": "markdown",
                                "value": format!("**member** `{}`", word)
                            }
                        }))
                    } else {
                        // Not a field access — check various possibilities
                        let mut lexer = Lexer::new(&text);
                        let tokens = lexer.tokenize();
                        let mut parser = Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            // Check function
                            if let Some(sig) = find_function_signature(&program, &word) {
                                return Some(serde_json::json!({
                                    "contents": {
                                        "kind": "markdown",
                                        "value": format!("**function**\n```axiom\n{}\n```", sig)
                                    }
                                }));
                            }
                            // Check variable (including params in functions)
                            if let Some(ty) = find_variable_type_in_program(&program, &word) {
                                return Some(serde_json::json!({
                                    "contents": {
                                        "kind": "markdown",
                                        "value": format!("**variable** `{}`\n```axiom\n{}: {}\n```", word, word, ty)
                                    }
                                }));
                            }
                            // Check type declarations
                            for item in &program.items {
                                if let axiom_ast::TopDecl::Type(t) = item {
                                    if t.name.name == word {
                                        let fields: Vec<String> = t.fields.iter()
                                            .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                                            .collect();
                                        let detail = if fields.is_empty() {
                                            format!("type `{}`", word)
                                        } else {
                                            format!("type `{}` {{\n  {}\n}}", word, fields.join("\n  "))
                                        };
                                        return Some(serde_json::json!({
                                            "contents": {
                                                "kind": "markdown",
                                                "value": format!("**type**\n```axiom\n{}\n```", detail)
                                            }
                                        }));
                                    }
                                }
                                if let axiom_ast::TopDecl::Enum(e) = item {
                                    if e.name.name == word {
                                        let variants: Vec<String> = e.variants.iter()
                                            .map(|v| {
                                                if v.fields.is_empty() {
                                                    v.name.name.clone()
                                                } else {
                                                    let fds: Vec<String> = v.fields.iter()
                                                        .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                                                        .collect();
                                                    format!("{}({})", v.name.name, fds.join(", "))
                                                }
                                            })
                                            .collect();
                                        return Some(serde_json::json!({
                                            "contents": {
                                                "kind": "markdown",
                                                "value": format!("**enum** `{}`\n```axiom\nenum {} {{\n  {}\n}}\n```", word, word, variants.join("\n  "))
                                            }
                                        }));
                                    }
                                }
                            }
                            // Check enum variant
                            if let Some((enum_name, _)) = find_enum_variant_info(&program, &word) {
                                return Some(serde_json::json!({
                                    "contents": {
                                        "kind": "markdown",
                                        "value": format!("**variant** of `{}`", enum_name)
                                    }
                                }));
                            }
                        }
                        // Fallback
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

                // Parse the completion context
                let (word_prefix, is_dot_completion, obj_name, member_prefix) = uri.as_ref().and_then(|u| {
                    let docs = backend.documents.lock().unwrap();
                    let text = docs.get(u)?;
                    let line_str = text.lines().nth(line)?;
                    let before_cursor = &line_str[..character.min(line_str.len())];
                    if let Some(dot_pos) = before_cursor.rfind('.') {
                        let mut obj_start = dot_pos;
                        let bytes = line_str.as_bytes();
                        while obj_start > 0 && (is_ident_char(bytes[obj_start - 1]) || bytes[obj_start - 1] == b'.') {
                            obj_start -= 1;
                        }
                        let obj_name = line_str[obj_start..dot_pos].to_string();
                        let member_prefix = line_str[dot_pos + 1..character].to_string();
                        Some((String::new(), true, obj_name, member_prefix))
                    } else if before_cursor.ends_with("::") {
                        // :: triggers for enum variant completion
                        let colon_pos = before_cursor.rfind("::").unwrap_or(0);
                        let mut obj_start = colon_pos;
                        let bytes = line_str.as_bytes();
                        while obj_start > 0 && is_ident_char(bytes[obj_start - 1]) {
                            obj_start -= 1;
                        }
                        let obj_name = line_str[obj_start..colon_pos].to_string();
                        let member_prefix = line_str[colon_pos + 2..character].to_string();
                        Some((String::new(), true, obj_name, member_prefix))
                    } else {
                        Some((extract_word(line_str, character), false, String::new(), String::new()))
                    }
                }).unwrap_or_default();

                // Keywords and primitives (for non-dot completions)
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

                // Add keyword completions for regular (non-dot) context
                if !is_dot_completion {
                    for kw in keywords.iter().chain(primitives.iter()) {
                        if kw.starts_with(&word_prefix) || word_prefix.is_empty() {
                            items.push(serde_json::json!({
                                "label": kw,
                                "kind": 14,
                                "insertText": kw
                            }));
                        }
                    }
                }

                // Add snippets for non-dot context
                if !is_dot_completion {
                    let snippets: Vec<(Vec<&str>, &str, &str, u32)> = vec![
                        (vec!["fn"], "function", "fn ${1:name}(${2:params})${3: -> ${4:ReturnType}} {\n\t${0}\n}", 3),
                        (vec!["if"], "if", "if ${1:condition} {\n\t${0}\n}", 3),
                        (vec!["elif", "else if"], "else if", "elif ${1:condition} {\n\t${0}\n}", 3),
                        (vec!["else"], "else", "else {\n\t${0}\n}", 3),
                        (vec!["while"], "while", "while ${1:condition} {\n\t${0}\n}", 3),
                        (vec!["for"], "for", "for ${1:ident} in ${2:expr} {\n\t${0}\n}", 3),
                        (vec!["match"], "match", "match ${1:expr} {\n\t${2:pattern} => ${0},\n}", 3),
                        (vec!["let"], "let binding", "let ${1:name}${2: : ${3:Type}} = ${4:expr}${0};", 3),
                        (vec!["var"], "var binding", "var ${1:name}${2: : ${3:Type}} = ${4:expr}${0};", 3),
                        (vec!["type"], "type", "type ${1:Name} = {\n\t${2:field}: ${3:Type},\n}", 3),
                        (vec!["enum"], "enum", "enum ${1:Name} {\n\t${2:Variant},\n}", 3),
                        (vec!["interface"], "interface", "interface ${1:Name} {\n\t${2:fn ${3:method}(${4:params})${5: -> ${6:Type}};}\n}", 3),
                        (vec!["module"], "module", "module ${1:name} {\n\t${0}\n}", 3),
                        (vec!["use"], "use", "use ${1:path};", 3),
                    ];
                    for (triggers, label, snippet, kind) in &snippets {
                        for trigger in triggers {
                            if trigger.starts_with(&word_prefix) || word_prefix.is_empty() {
                                items.push(serde_json::json!({
                                    "label": format!("{}\t({})", trigger, label),
                                    "kind": kind,
                                    "detail": *label,
                                    "insertText": snippet,
                                    "insertTextFormat": 2
                                }));
                            }
                        }
                    }

                    // Add `self` for method bodies
                    items.push(serde_json::json!({
                        "label": "self",
                        "kind": 14,
                        "detail": "method receiver",
                        "insertText": "self"
                    }));

                    // Add standard library module names
                    let std_modules = vec![
                        "axiom", "io", "math", "string", "collections",
                        "fs", "net", "time", "json", "test",
                    ];
                    let alias_modules = std_modules.iter().map(|m| {
                        if m.starts_with(&word_prefix) || word_prefix.is_empty() {
                            Some(serde_json::json!({
                                "label": m,
                                "kind": 2,
                                "detail": "module",
                                "insertText": m
                            }))
                        } else {
                            None
                        }
                    });
                    for m in alias_modules.flatten() {
                        items.push(m);
                    }
                }

                // Parse document for program-level completions
                if let Some(ref u) = uri {
                    let docs = backend.documents.lock().unwrap();
                    if let Some(text) = docs.get(u) {
                        let mut lexer = axiom_lexer::Lexer::new(text);
                        let tokens = lexer.tokenize();
                        let mut parser = axiom_parser::Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            // Non-dot: add all program symbols
                            if !is_dot_completion {
                                for item in &program.items {
                                    collect_symbols(item, &mut items, &word_prefix);
                                }

                                // Add enum variants as standalone completions
                                for item in &program.items {
                                    if let axiom_ast::TopDecl::Enum(ed) = item {
                                        for variant in &ed.variants {
                                            let label = format!("{}::{}", ed.name.name, variant.name.name);
                                            if word_prefix.is_empty() || label.starts_with(&word_prefix) {
                                                let mut detail = format!("variant of {}", ed.name.name);
                                                if !variant.fields.is_empty() {
                                                    let fds: Vec<String> = variant.fields.iter()
                                                        .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                                                        .collect();
                                                    detail = format!("{}({})", detail, fds.join(", "));
                                                }
                                                items.push(serde_json::json!({
                                                    "label": &label,
                                                    "kind": 22,
                                                    "detail": detail,
                                                    "insertText": &label
                                                }));
                                            }
                                        }
                                    }
                                }
                            }

                            // Dot completion
                            if is_dot_completion && !obj_name.is_empty() {
                                // Try to resolve the type of the object expression
                                if let Some(type_name) = resolve_obj_type_text(&program, &obj_name) {
                                    // Struct fields
                                    let fields = find_struct_fields_in_program(&program, &type_name);
                                    for (field_name, field_type) in &fields {
                                        if member_prefix.is_empty() || field_name.starts_with(&member_prefix) {
                                            items.push(serde_json::json!({
                                                "label": field_name,
                                                "kind": 5,
                                                "detail": field_type,
                                                "insertText": field_name
                                            }));
                                        }
                                    }

                                    // Methods (receiver-based)
                                    let methods = find_methods_in_program(&program, &type_name);
                                    for (method_name, sig) in &methods {
                                        if member_prefix.is_empty() || method_name.starts_with(&member_prefix) {
                                            items.push(serde_json::json!({
                                                "label": method_name,
                                                "kind": 2,
                                                "detail": sig,
                                                "insertText": format!("{}(", method_name)
                                            }));
                                        }
                                    }

                                    // Interface methods
                                    let iface_methods = find_interface_methods_for_type(&program, &type_name);
                                    for (method_name, sig) in &iface_methods {
                                        if member_prefix.is_empty() || method_name.starts_with(&member_prefix) {
                                            items.push(serde_json::json!({
                                                "label": method_name,
                                                "kind": 2,
                                                "detail": sig,
                                                "insertText": format!("{}(", method_name)
                                            }));
                                        }
                                    }

                                    // Enum variants (for enum types)
                                    let enum_variants = collect_enum_variants_for_type(&program, &type_name);
                                    for (variant_label, variant_detail) in &enum_variants {
                                        if member_prefix.is_empty() || variant_label.starts_with(&member_prefix) {
                                            items.push(serde_json::json!({
                                                "label": variant_label,
                                                "kind": 22,
                                                "detail": variant_detail,
                                                "insertText": variant_label
                                            }));
                                        }
                                    }
                                }

                                // Try module-qualified members
                                collect_module_members(&program, &obj_name, &member_prefix, &mut items);
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
                let mut active_parameter = 0;

                let docs = backend.documents.lock().unwrap();
                if let Some(text) = docs.get(uri) {
                    let line_str = text.lines().nth(line).unwrap_or("");
                    let before_cursor = &line_str[..character.min(line_str.len())];
                    if let Some(paren_pos) = before_cursor.rfind('(') {
                        // Extract function name
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

                        // Count commas between `(` and cursor to determine activeParameter
                        let after_paren = &before_cursor[paren_pos + 1..];
                        let mut comma_count = 0;
                        let mut depth_paren = 0;
                        let mut depth_brace = 0;
                        let mut in_string = false;
                        let mut string_char = '"';

                        for c in after_paren.chars() {
                            if in_string {
                                if c == string_char {
                                    in_string = false;
                                }
                                continue;
                            }
                            match c {
                                '"' | '\'' => {
                                    in_string = true;
                                    string_char = c;
                                }
                                '(' | '[' => depth_paren += 1,
                                ')' | ']' => {
                                    if depth_paren > 0 { depth_paren -= 1; }
                                }
                                '{' => depth_brace += 1,
                                '}' => {
                                    if depth_brace > 0 { depth_brace -= 1; }
                                }
                                ',' if depth_paren == 0 && depth_brace == 0 => {
                                    comma_count += 1;
                                }
                                _ => {}
                            }
                        }
                        active_parameter = comma_count;
                    }
                }

                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "signatures": signatures,
                        "activeSignature": 0,
                        "activeParameter": active_parameter
                    }
                }));
            }

            "textDocument/documentSymbol" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
                let mut symbols = Vec::new();

                if let Some(ref u) = uri {
                    let docs = backend.documents.lock().unwrap();
                    if let Some(text) = docs.get(u) {
                        let mut lexer = axiom_lexer::Lexer::new(text);
                        let tokens = lexer.tokenize();
                        let mut parser = axiom_parser::Parser::new(tokens);
                        if let Ok(program) = parser.parse_program() {
                            for item in &program.items {
                                collect_document_symbols(item, &mut symbols);
                            }
                        }
                    }
                }

                let id = msg["id"].clone();
                write_lsp_message(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": symbols
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
