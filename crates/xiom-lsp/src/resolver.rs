// XIOM Language Server — Type resolution, context helpers, and symbol lookup
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast;
use crate::backend::Backend;

// ============================================================================
// Context helpers
// ============================================================================

pub fn extract_word(line: &str, col: usize) -> String {
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

pub fn word_start_pos(line: &str, col: usize) -> usize {
    let bytes = line.as_bytes();
    let mut start = col;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    start
}

pub fn is_ident_char(c: u8) -> bool {
    (c >= b'a' && c <= b'z')
        || (c >= b'A' && c <= b'Z')
        || (c >= b'0' && c <= b'9')
        || c == b'_'
}

pub fn extract_obj_expr(line: &str, dot_pos: usize) -> String {
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

/// Find the identifier at a given line/col position.
pub fn find_ident_at(backend: &Backend, uri: &str, line: usize, col: usize) -> Option<String> {
    let docs = backend.documents.lock().unwrap();
    let text = docs.get(uri)?;
    let target_line = text.lines().nth(line)?;

    let chars: Vec<char> = target_line.chars().collect();
    if col >= chars.len() { return None; }

    let mut start = col;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    let mut end = col;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

// ============================================================================
// Type helpers for hover / completion / signature help
// ============================================================================

pub fn type_to_string(ty: &xiom_ast::Type) -> String {
    match ty {
        xiom_ast::Type::Named(ident, args) => {
            if args.is_empty() {
                ident.name.clone()
            } else {
                let args_str: Vec<String> = args.iter().map(type_to_string).collect();
                format!("{}[{}]", ident.name, args_str.join(", "))
            }
        }
        xiom_ast::Type::Ref(t) => format!("&{}", type_to_string(t)),
        xiom_ast::Type::MutRef(t) => format!("&mut {}", type_to_string(t)),
        xiom_ast::Type::Option(t) => format!("Option[{}]", type_to_string(t)),
        xiom_ast::Type::Result(t, e) => format!("Result[{}, {}]", type_to_string(t), type_to_string(e)),
        xiom_ast::Type::Vec(t) => format!("Vec[{}]", type_to_string(t)),
        xiom_ast::Type::Slice(t) => format!("Slice[{}]", type_to_string(t)),
        xiom_ast::Type::Map(k, v) => format!("Map[{}, {}]", type_to_string(k), type_to_string(v)),
        xiom_ast::Type::Set(t) => format!("Set[{}]", type_to_string(t)),
        xiom_ast::Type::Tuple(types) => {
            let items: Vec<String> = types.iter().map(type_to_string).collect();
            format!("({})", items.join(", "))
        }
        xiom_ast::Type::Ptr(t) => format!("*{}", type_to_string(t)),
        xiom_ast::Type::Array(_, _) => "Array".to_string(),
        xiom_ast::Type::Fn(params, ret) => {
            let params_str: Vec<String> = params.iter().map(type_to_string).collect();
            format!("fn({}) -> {}", params_str.join(", "), type_to_string(ret))
        }
        xiom_ast::Type::ImplTrait(traits) => {
            let names: Vec<String> = traits.iter().map(|t| t.name.clone()).collect();
            format!("impl {}", names.join(" + "))
        }
    }
}

pub fn infer_type_from_expr(expr: &xiom_ast::Expr) -> Option<String> {
    match expr {
        xiom_ast::Expr::Struct(ident, _, _, _) => Some(ident.name.clone()),
        xiom_ast::Expr::Some(_, _) => Some("Option".to_string()),
        xiom_ast::Expr::None(_) => Some("Option".to_string()),
        xiom_ast::Expr::Ok(_, _) => Some("Result".to_string()),
        xiom_ast::Expr::Err(_, _) => Some("Result".to_string()),
        xiom_ast::Expr::Int(_, _) => Some("Int".to_string()),
        xiom_ast::Expr::Float(_, _) => Some("Float64".to_string()),
        xiom_ast::Expr::Str(_, _) => Some("Str".to_string()),
        xiom_ast::Expr::Bool(_, _) => Some("Bool".to_string()),
        xiom_ast::Expr::Char(_, _) => Some("Char".to_string()),
        xiom_ast::Expr::Ident(ident) => Some(ident.name.clone()),
        xiom_ast::Expr::Unsafe(_, _) => None,
        _ => None,
    }
}

// ============================================================================
// Recursive variable type lookup
// ============================================================================

pub fn find_variable_type_in_program(program: &xiom_ast::Program, var_name: &str) -> Option<String> {
    for item in &program.items {
        if let Some(ty) = find_variable_type_in_item(item, var_name) {
            return Some(ty);
        }
    }
    None
}

fn find_variable_type_in_item(item: &xiom_ast::TopDecl, var_name: &str) -> Option<String> {
    match item {
        xiom_ast::TopDecl::Fn(f) => {
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
        xiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                if let Some(ty) = find_variable_type_in_item(inner, var_name) {
                    return Some(ty);
                }
            }
            None
        }
        xiom_ast::TopDecl::Extern(_) => None,
        _ => None,
    }
}

fn find_variable_type_in_block(block: &xiom_ast::Block, var_name: &str) -> Option<String> {
    for soe in &block.stmts {
        if let xiom_ast::StmtOrExpr::Stmt(stmt) = soe {
            if let Some(ty) = find_variable_type_in_stmt(stmt, var_name) {
                return Some(ty);
            }
        }
    }
    None
}

fn find_variable_type_in_stmt(stmt: &xiom_ast::Stmt, var_name: &str) -> Option<String> {
    match stmt {
        xiom_ast::Stmt::Let(ident, ty, expr, _)
        | xiom_ast::Stmt::Var(ident, ty, expr, _) => {
            if ident.name == var_name {
                if let Some(t) = ty {
                    return Some(type_to_string(t));
                }
                return infer_type_from_expr(expr);
            }
        }
        xiom_ast::Stmt::If(_, then_block, elifs, else_block, _) => {
            if let Some(ty) = find_variable_type_in_block(then_block, var_name) { return Some(ty); }
            for (_, elif_block) in elifs {
                if let Some(ty) = find_variable_type_in_block(elif_block, var_name) { return Some(ty); }
            }
            if let Some(eb) = else_block {
                if let Some(ty) = find_variable_type_in_block(eb, var_name) { return Some(ty); }
            }
        }
        xiom_ast::Stmt::While(_, body, _, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) { return Some(ty); }
        }
        xiom_ast::Stmt::Match(_, arms, _) => {
            for arm in arms {
                if let xiom_ast::MatchBody::Block(b) = &arm.body {
                    if let Some(ty) = find_variable_type_in_block(b, var_name) { return Some(ty); }
                }
            }
        }
        xiom_ast::Stmt::For(_, _, body, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) { return Some(ty); }
        }
        xiom_ast::Stmt::Spawn(body, _) => {
            if let Some(ty) = find_variable_type_in_block(body, var_name) { return Some(ty); }
        }
        _ => {}
    }
    None
}

// ============================================================================
// Struct / method / enum / interface / module lookup helpers
// ============================================================================

pub fn format_fn_signature(f: &xiom_ast::FnDecl) -> String {
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

pub fn find_struct_fields_in_program(program: &xiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for item in &program.items {
        recurse_find_struct_fields(item, type_name, &mut fields);
    }
    fields
}

fn recurse_find_struct_fields(item: &xiom_ast::TopDecl, type_name: &str, fields: &mut Vec<(String, String)>) {
    match item {
        xiom_ast::TopDecl::Type(td) => {
            if td.name.name == type_name {
                for field in &td.fields {
                    fields.push((field.name.name.clone(), type_to_string(&field.ty)));
                }
            }
        }
        xiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                recurse_find_struct_fields(inner, type_name, fields);
            }
        }
        xiom_ast::TopDecl::Extern(_) => {}
        _ => {}
    }
}

pub fn find_methods_in_program(program: &xiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut methods = Vec::new();
    for item in &program.items {
        if let xiom_ast::TopDecl::Fn(f) = item {
            if let Some(receiver) = &f.receiver {
                if receiver.name == type_name {
                    methods.push((f.name.name.clone(), format_fn_signature(f)));
                }
            }
        }
    }
    methods
}

pub fn find_interface_methods_for_type(program: &xiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut methods = Vec::new();
    for item in &program.items {
        if let xiom_ast::TopDecl::Fn(f) = item {
            if let Some(receiver) = &f.receiver {
                if receiver.name == type_name {
                    methods.push((f.name.name.clone(), format_fn_signature(f)));
                }
            }
        }
    }
    for item in &program.items {
        if let xiom_ast::TopDecl::Interface(id) = item {
            for member in &id.members {
                if let xiom_ast::InterfaceMember::FnSignature(fs) = member {
                    methods.push((format!("{}.{}", id.name.name, fs.name.name), format_fn_signature(fs)));
                }
            }
        }
    }
    methods
}

pub fn find_function_signature(program: &xiom_ast::Program, fn_name: &str) -> Option<String> {
    for item in &program.items {
        if let xiom_ast::TopDecl::Fn(f) = item {
            if f.name.name == fn_name {
                return Some(format_fn_signature(f));
            }
        }
    }
    None
}

pub fn find_enum_variant_info(program: &xiom_ast::Program, variant_name: &str) -> Option<(String, String)> {
    for item in &program.items {
        if let xiom_ast::TopDecl::Enum(ed) = item {
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

pub fn collect_enum_variants_for_type(program: &xiom_ast::Program, type_name: &str) -> Vec<(String, String)> {
    let mut variants = Vec::new();
    for item in &program.items {
        if let xiom_ast::TopDecl::Enum(ed) = item {
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

pub fn resolve_obj_type_text(program: &xiom_ast::Program, obj_expr: &str) -> Option<String> {
    let parts: Vec<&str> = obj_expr.split('.').collect();
    if parts.is_empty() { return None; }

    let first = parts[0];

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
            if !found { return None; }
        }
        return Some(current_type);
    }

    if parts.len() == 1 {
        for item in &program.items {
            match item {
                xiom_ast::TopDecl::Type(td) if td.name.name == first => return Some(first.to_string()),
                xiom_ast::TopDecl::Enum(ed) if ed.name.name == first => return Some(first.to_string()),
                _ => {}
            }
        }
    }

    if let Some(ty) = resolve_module_qualified_type(program, &parts) {
        return Some(ty);
    }

    None
}

fn resolve_module_qualified_type(program: &xiom_ast::Program, parts: &[&str]) -> Option<String> {
    if parts.len() < 2 { return None; }
    let module_name = parts[0];
    for item in &program.items {
        if let xiom_ast::TopDecl::Module(m) = item {
            if m.name.name == module_name {
                let mut current_items = &m.items;
                for i in 1..parts.len() - 1 {
                    let segment = parts[i];
                    let mut found = false;
                    for inner in current_items.iter() {
                        if let xiom_ast::TopDecl::Module(sub) = inner {
                            if sub.name.name == segment {
                                current_items = &sub.items;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found { return None; }
                }
                let last = parts[parts.len() - 1];
                for inner in current_items.iter() {
                    match inner {
                        xiom_ast::TopDecl::Type(td) if td.name.name == last => return Some(last.to_string()),
                        xiom_ast::TopDecl::Enum(ed) if ed.name.name == last => return Some(last.to_string()),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

// ============================================================================
// Symbol collection for completion
// ============================================================================

pub fn collect_symbols(item: &xiom_ast::TopDecl, items: &mut Vec<serde_json::Value>, prefix: &str) {
    match item {
        xiom_ast::TopDecl::Fn(f) => {
            let name = &f.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 3,
                    "detail": "function",
                    "insertText": name
                }));
            }
        }
        xiom_ast::TopDecl::Type(td) => {
            let name = &td.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 23,
                    "detail": "type",
                    "insertText": name
                }));
            }
        }
        xiom_ast::TopDecl::Enum(ed) => {
            let name = &ed.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 13,
                    "detail": "enum",
                    "insertText": name
                }));
            }
        }
        xiom_ast::TopDecl::Interface(id) => {
            let name = &id.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 11,
                    "detail": "interface",
                    "insertText": name
                }));
            }
        }
        xiom_ast::TopDecl::Module(m) => {
            for inner in &m.items {
                collect_symbols(inner, items, prefix);
            }
        }
        xiom_ast::TopDecl::Const(cd) => {
            let name = &cd.name.name;
            if name.starts_with(prefix) || prefix.is_empty() {
                items.push(serde_json::json!({
                    "label": name,
                    "kind": 14,
                    "detail": "const",
                    "insertText": name
                }));
            }
        }
        xiom_ast::TopDecl::Extern(_) => {}
        _ => {}
    }
}

pub fn collect_module_members(
    program: &xiom_ast::Program,
    module_name: &str,
    prefix: &str,
    items: &mut Vec<serde_json::Value>,
) {
    for item in &program.items {
        if let xiom_ast::TopDecl::Module(m) = item {
            if m.name.name == module_name {
                collect_module_item_completions(&m.items, prefix, items);
                return;
            }
        }
    }
}

fn collect_module_item_completions(
    items: &[xiom_ast::TopDecl],
    prefix: &str,
    out: &mut Vec<serde_json::Value>,
) {
    for item in items {
        match item {
            xiom_ast::TopDecl::Fn(f) => {
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
            xiom_ast::TopDecl::Type(td) => {
                let name = &td.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name, "kind": 23, "detail": "type", "insertText": name
                    }));
                }
            }
            xiom_ast::TopDecl::Enum(ed) => {
                let name = &ed.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name, "kind": 13, "detail": "enum", "insertText": name
                    }));
                }
            }
            xiom_ast::TopDecl::Interface(id) => {
                let name = &id.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name, "kind": 11, "detail": "interface", "insertText": name
                    }));
                }
            }
            xiom_ast::TopDecl::Module(m) => {
                let name = &m.name.name;
                if prefix.is_empty() || name.starts_with(prefix) {
                    out.push(serde_json::json!({
                        "label": name, "kind": 2, "detail": "module", "insertText": name
                    }));
                }
            }
            xiom_ast::TopDecl::Const(cd) => {
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
            xiom_ast::TopDecl::Extern(_) => {}
            _ => {}
        }
    }
}
