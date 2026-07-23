// XIOM Language Server — Document symbols, workspace symbols, and definition lookup
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast;
use xiom_lexer::Lexer;
use xiom_parser::Parser;

use crate::resolver::{format_fn_signature, type_to_string};

// ============================================================================
// Document symbols
// ============================================================================

pub fn collect_document_symbols(
    item: &xiom_ast::TopDecl,
    symbols: &mut Vec<serde_json::Value>,
) {
    match item {
        xiom_ast::TopDecl::Fn(f) => {
            let line = if f.name.span.line > 0 { f.name.span.line as u64 - 1 } else { 0 };
            let col = if f.name.span.col > 0 { f.name.span.col as u64 - 1 } else { 0 };
            let sig = format_fn_signature(f);
            symbols.push(serde_json::json!({
                "name": f.name.name,
                "detail": sig,
                "kind": 12,
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
        xiom_ast::TopDecl::Type(td) => {
            let line = if td.name.span.line > 0 { td.name.span.line as u64 - 1 } else { 0 };
            let col = if td.name.span.col > 0 { td.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for field in &td.fields {
                let fline = if field.name.span.line > 0 { field.name.span.line as u64 - 1 } else { 0 };
                let fcol = if field.name.span.col > 0 { field.name.span.col as u64 - 1 } else { 0 };
                children.push(serde_json::json!({
                    "name": field.name.name,
                    "detail": type_to_string(&field.ty),
                    "kind": 8,
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
                "name": td.name.name, "detail": "type", "kind": 23,
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
        xiom_ast::TopDecl::Enum(ed) => {
            let line = if ed.name.span.line > 0 { ed.name.span.line as u64 - 1 } else { 0 };
            let col = if ed.name.span.col > 0 { ed.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for variant in &ed.variants {
                let vline = if variant.name.span.line > 0 { variant.name.span.line as u64 - 1 } else { 0 };
                let vcol = if variant.name.span.col > 0 { variant.name.span.col as u64 - 1 } else { 0 };
                children.push(serde_json::json!({
                    "name": variant.name.name, "kind": 22,
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
                "name": ed.name.name, "detail": "enum", "kind": 13,
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
        xiom_ast::TopDecl::Interface(id) => {
            let line = if id.name.span.line > 0 { id.name.span.line as u64 - 1 } else { 0 };
            let col = if id.name.span.col > 0 { id.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for member in &id.members {
                match member {
                    xiom_ast::InterfaceMember::Field(fd) => {
                        let fline = if fd.name.span.line > 0 { fd.name.span.line as u64 - 1 } else { 0 };
                        let fcol = if fd.name.span.col > 0 { fd.name.span.col as u64 - 1 } else { 0 };
                        children.push(serde_json::json!({
                            "name": fd.name.name,
                            "detail": type_to_string(&fd.ty),
                            "kind": 8,
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
                    xiom_ast::InterfaceMember::FnSignature(fs) => {
                        let fline = if fs.name.span.line > 0 { fs.name.span.line as u64 - 1 } else { 0 };
                        let fcol = if fs.name.span.col > 0 { fs.name.span.col as u64 - 1 } else { 0 };
                        let sig = format_fn_signature(fs);
                        children.push(serde_json::json!({
                            "name": fs.name.name,
                            "detail": sig,
                            "kind": 6,
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
                "name": id.name.name, "detail": "interface", "kind": 11,
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
        xiom_ast::TopDecl::Module(m) => {
            let line = if m.name.span.line > 0 { m.name.span.line as u64 - 1 } else { 0 };
            let col = if m.name.span.col > 0 { m.name.span.col as u64 - 1 } else { 0 };
            let mut children = Vec::new();
            for inner in &m.items {
                collect_document_symbols(inner, &mut children);
            }
            symbols.push(serde_json::json!({
                "name": m.name.name, "detail": "module", "kind": 2,
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
        xiom_ast::TopDecl::Const(cd) => {
            let line = if cd.name.span.line > 0 { cd.name.span.line as u64 - 1 } else { 0 };
            let col = if cd.name.span.col > 0 { cd.name.span.col as u64 - 1 } else { 0 };
            symbols.push(serde_json::json!({
                "name": cd.name.name,
                "detail": format!("const {}: {}", cd.name.name, type_to_string(&cd.ty)),
                "kind": 14,
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
        xiom_ast::TopDecl::Extern(_) => {}
        _ => {}
    }
}

// ============================================================================
// Workspace symbols
// ============================================================================

pub fn collect_workspace_symbols(
    item: &xiom_ast::TopDecl,
    uri: &str,
    query: &str,
    results: &mut Vec<serde_json::Value>,
) {
    let query_lower = query.to_lowercase();
    match item {
        xiom_ast::TopDecl::Fn(f) => {
            let name = &f.name.name;
            if query.is_empty() || name.to_lowercase().contains(&query_lower) {
                let line = if f.name.span.line > 0 { f.name.span.line as u64 - 1 } else { 0 };
                let col = if f.name.span.col > 0 { f.name.span.col as u64 - 1 } else { 0 };
                let len = name.len() as u64;
                results.push(serde_json::json!({
                    "name": name, "kind": 12,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + len }
                        }
                    }
                }));
            }
        }
        xiom_ast::TopDecl::Type(td) => {
            let name = &td.name.name;
            if query.is_empty() || name.to_lowercase().contains(&query_lower) {
                let line = if td.name.span.line > 0 { td.name.span.line as u64 - 1 } else { 0 };
                let col = if td.name.span.col > 0 { td.name.span.col as u64 - 1 } else { 0 };
                let len = name.len() as u64;
                results.push(serde_json::json!({
                    "name": name, "kind": 23,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + len }
                        }
                    }
                }));
            }
        }
        xiom_ast::TopDecl::Enum(ed) => {
            let name = &ed.name.name;
            if query.is_empty() || name.to_lowercase().contains(&query_lower) {
                let line = if ed.name.span.line > 0 { ed.name.span.line as u64 - 1 } else { 0 };
                let col = if ed.name.span.col > 0 { ed.name.span.col as u64 - 1 } else { 0 };
                let len = name.len() as u64;
                results.push(serde_json::json!({
                    "name": name, "kind": 13,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + len }
                        }
                    }
                }));
            }
        }
        xiom_ast::TopDecl::Interface(id) => {
            let name = &id.name.name;
            if query.is_empty() || name.to_lowercase().contains(&query_lower) {
                let line = if id.name.span.line > 0 { id.name.span.line as u64 - 1 } else { 0 };
                let col = if id.name.span.col > 0 { id.name.span.col as u64 - 1 } else { 0 };
                let len = name.len() as u64;
                results.push(serde_json::json!({
                    "name": name, "kind": 11,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + len }
                        }
                    }
                }));
            }
        }
        xiom_ast::TopDecl::Const(cd) => {
            let name = &cd.name.name;
            if query.is_empty() || name.to_lowercase().contains(&query_lower) {
                let line = if cd.name.span.line > 0 { cd.name.span.line as u64 - 1 } else { 0 };
                let col = if cd.name.span.col > 0 { cd.name.span.col as u64 - 1 } else { 0 };
                let len = name.len() as u64;
                results.push(serde_json::json!({
                    "name": name, "kind": 14,
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + len }
                        }
                    }
                }));
            }
        }
        xiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                collect_workspace_symbols(inner, uri, query, results);
            }
        }
        xiom_ast::TopDecl::Extern(_) => {}
        _ => {}
    }
}

pub fn parse_workspace_document(source: &str) -> Vec<xiom_ast::TopDecl> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => program.items,
        Err(_) => Vec::new(),
    }
}

// ============================================================================
// Definition lookup
// ============================================================================

pub fn find_definition(program: &xiom_ast::Program, name: &str) -> Option<(u64, u64)> {
    for item in &program.items {
        if let Some(pos) = find_def_in_item(item, name) {
            return Some(pos);
        }
    }
    None
}

fn find_def_in_item(item: &xiom_ast::TopDecl, name: &str) -> Option<(u64, u64)> {
    match item {
        xiom_ast::TopDecl::Fn(f) if f.name.name == name => {
            let line = if f.name.span.line > 0 { f.name.span.line as u64 - 1 } else { 0 };
            let col = if f.name.span.col > 0 { f.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        xiom_ast::TopDecl::Type(td) if td.name.name == name => {
            let line = if td.name.span.line > 0 { td.name.span.line as u64 - 1 } else { 0 };
            let col = if td.name.span.col > 0 { td.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        xiom_ast::TopDecl::Enum(ed) if ed.name.name == name => {
            let line = if ed.name.span.line > 0 { ed.name.span.line as u64 - 1 } else { 0 };
            let col = if ed.name.span.col > 0 { ed.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        xiom_ast::TopDecl::Interface(id) if id.name.name == name => {
            let line = if id.name.span.line > 0 { id.name.span.line as u64 - 1 } else { 0 };
            let col = if id.name.span.col > 0 { id.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        xiom_ast::TopDecl::Module(m) => {
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
        xiom_ast::TopDecl::Const(cd) if cd.name.name == name => {
            let line = if cd.name.span.line > 0 { cd.name.span.line as u64 - 1 } else { 0 };
            let col = if cd.name.span.col > 0 { cd.name.span.col as u64 - 1 } else { 0 };
            Some((line, col))
        }
        xiom_ast::TopDecl::Extern(_) => None,
        _ => None,
    }
}
