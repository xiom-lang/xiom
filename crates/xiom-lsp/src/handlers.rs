// XIOM Language Server -- LSP method handlers
// Each handler is extracted from the monolith handle_lsp_message dispatch.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ai::get_ai_insight_for_line;
use crate::backend::Backend;
use crate::resolver::{
    collect_enum_variants_for_type, collect_module_members, collect_symbols,
    extract_obj_expr, extract_word, find_enum_variant_info,
    find_function_signature, find_ident_at, find_interface_methods_for_type,
    find_methods_in_program, find_struct_fields_in_program,
    find_variable_type_in_program, resolve_obj_type_text, word_start_pos,
};
use crate::semantic_tokens::compute_semantic_tokens;
use crate::symbols::{
    collect_document_symbols, collect_workspace_symbols, find_definition,
};
use crate::text_edit::apply_text_edit;

// ============================================================================
// Lifecycle handlers
// ============================================================================

pub fn handle_initialize(msg: &serde_json::Value, responses: &mut Vec<serde_json::Value>) {
    let id = msg["id"].clone();
    let init_result = serde_json::json!({
        "capabilities": {
            "textDocumentSync": { "openClose": true, "change": 2 },
            "hoverProvider": true,
            "completionProvider": { "triggerCharacters": [".", ":"] },
            "definitionProvider": true,
            "signatureHelpProvider": { "triggerCharacters": ["(", ","] },
            "documentSymbolProvider": true,
            "referencesProvider": true,
            "renameProvider": true,
            "codeActionProvider": true,
            "workspaceSymbolProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": ["keyword", "type", "function", "variable",
                        "string", "number", "comment", "operator"],
                    "tokenModifiers": ["declaration", "readonly"]
                },
                "full": true
            }
        }
    });
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": init_result
    }));
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "method": "window/logMessage",
        "params": { "type": 3, "message": format!("XIOM Language Server v{}", env!("CARGO_PKG_VERSION")) }
    }));
}

pub fn handle_shutdown(msg: &serde_json::Value, responses: &mut Vec<serde_json::Value>) {
    let id = msg["id"].clone();
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": null
    }));
}

// ============================================================================
// Document sync handlers
// ============================================================================

pub fn handle_did_open(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let params = &msg["params"];
    if let (Some(uri), Some(text)) = (
        params["textDocument"]["uri"].as_str(),
        params["textDocument"]["text"].as_str(),
    ) {
        let uri = uri.to_string();
        {
            let mut docs = backend.documents();
            docs.insert(uri.clone(), text.to_string());
        }
        backend.mark_index_dirty();
        let diagnostics = backend.publish_diagnostics(&uri);
        responses.push(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diagnostics }
        }));
    }
}

pub fn handle_did_change(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let params = &msg["params"];
    if let Some(uri) = params["textDocument"]["uri"].as_str() {
        let uri = uri.to_string();
        if let Some(changes) = params["contentChanges"].as_array() {
            {
                let mut docs = backend.documents();
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
            backend.mark_index_dirty();
            let diagnostics = backend.publish_diagnostics(&uri);
            responses.push(serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": { "uri": uri, "diagnostics": diagnostics }
            }));
        }
    }
}

pub fn handle_did_close(msg: &serde_json::Value, backend: &Backend) {
    let params = &msg["params"];
    if let Some(uri) = params["textDocument"]["uri"].as_str() {
        let mut docs = backend.documents();
        docs.remove(uri);
        drop(docs);
        backend.mark_index_dirty();
    }
}

// ============================================================================
// Feature handlers
// ============================================================================

pub fn handle_hover(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let ai_insight = uri.as_ref().and_then(|u| get_ai_insight_for_line(u, line));

    let hover = uri.and_then(|u| {
        let docs = backend.documents();
        let text = docs.get(&u)?.clone();
        drop(docs);
        let line_str = text.lines().nth(line)?;
        // LSP character offsets are UTF-16 units; the resolver helpers index
        // BYTES. Convert once, clamped (multibyte lines used to slice mid-char).
        let byte_pos = crate::position::utf16_to_byte(line_str, character);
        let word = extract_word(line_str, byte_pos);
        if word.is_empty() { return None; }

        let bytes = line_str.as_bytes();
        let wstart = word_start_pos(line_str, byte_pos);
        let is_field_access = wstart > 0 && wstart <= bytes.len() && bytes[wstart - 1] == b'.';

        if is_field_access {
            let dot_pos = wstart - 1;
            let obj_expr = extract_obj_expr(line_str, dot_pos);
            if let Some(program) = backend.parse_cached(&u, &text) {
                if let Some(obj_type) = resolve_obj_type_text(&program, &obj_expr) {
                    let fields = find_struct_fields_in_program(&program, &obj_type);
                    for (fname, ftype) in &fields {
                        if fname == &word {
                            return Some(serde_json::json!({
                                "contents": { "kind": "markdown", "value": format!("**field** `{}`\n```xiom\n{}: {}\n```", fname, fname, ftype) }
                            }));
                        }
                    }
                    let methods = find_methods_in_program(&program, &obj_type);
                    for (mname, sig) in &methods {
                        if mname == &word {
                            return Some(serde_json::json!({
                                "contents": { "kind": "markdown", "value": format!("**method**\n```xiom\n{}\n```", sig) }
                            }));
                        }
                    }
                    let iface_methods = find_interface_methods_for_type(&program, &obj_type);
                    for (mname, sig) in &iface_methods {
                        if mname == &word {
                            return Some(serde_json::json!({
                                "contents": { "kind": "markdown", "value": format!("**method**\n```xiom\n{}\n```", sig) }
                            }));
                        }
                    }
                }
            }
            Some(serde_json::json!({
                "contents": { "kind": "markdown", "value": format!("**member** `{}`", word) }
            }))
        } else {
            if let Some(program) = backend.parse_cached(&u, &text) {
                if let Some(sig) = find_function_signature(&program, &word) {
                    return Some(serde_json::json!({
                        "contents": { "kind": "markdown", "value": format!("**function**\n```xiom\n{}\n```", sig) }
                    }));
                }
                if let Some(ty) = find_variable_type_in_program(&program, &word) {
                    return Some(serde_json::json!({
                        "contents": { "kind": "markdown", "value": format!("**variable** `{}`\n```xiom\n{}: {}\n```", word, word, ty) }
                    }));
                }
                for item in &program.items {
                    if let xiom_ast::TopDecl::Type(t) = item {
                        if t.name.name == word {
                            let fields: Vec<String> = t.fields.iter()
                                .map(|f| format!("{}: {}", f.name.name, crate::resolver::type_to_string(&f.ty)))
                                .collect();
                            let detail = if fields.is_empty() {
                                format!("type `{}`", word)
                            } else {
                                format!("type `{}` {{\n  {}\n}}", word, fields.join("\n  "))
                            };
                            return Some(serde_json::json!({
                                "contents": { "kind": "markdown", "value": format!("**type**\n```xiom\n{}\n```", detail) }
                            }));
                        }
                    }
                    if let xiom_ast::TopDecl::Enum(e) = item {
                        if e.name.name == word {
                            let variants: Vec<String> = e.variants.iter().map(|v| {
                                if v.fields.is_empty() {
                                    v.name.name.clone()
                                } else {
                                    let fds: Vec<String> = v.fields.iter()
                                        .map(|f| format!("{}: {}", f.name.name, crate::resolver::type_to_string(&f.ty)))
                                        .collect();
                                    format!("{}({})", v.name.name, fds.join(", "))
                                }
                            }).collect();
                            return Some(serde_json::json!({
                                "contents": { "kind": "markdown", "value": format!("**enum** `{}`\n```xiom\nenum {} {{\n  {}\n}}\n```", word, word, variants.join("\n  ")) }
                            }));
                        }
                    }
                }
                if let Some((enum_name, _)) = find_enum_variant_info(&program, &word) {
                    return Some(serde_json::json!({
                        "contents": { "kind": "markdown", "value": format!("**variant** of `{}`", enum_name) }
                    }));
                }
            }
            Some(serde_json::json!({
                "contents": { "kind": "markdown", "value": format!("XIOM identifier: `{}`", word) }
            }))
        }
    });

    let hover = hover.map(|mut h| {
        if let Some(ref ai_text) = ai_insight {
            if let Some(obj) = h.as_object_mut() {
                if let Some(contents) = obj.get_mut("contents").and_then(|c| c.as_object_mut()) {
                    if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
                        let enhanced = format!("{}\n\n---\nAI Insight: {}", value, ai_text);
                        contents.insert("value".into(), serde_json::json!(enhanced));
                    }
                }
            }
        }
        h
    });

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": hover }));
}

pub fn handle_completion(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let mut items = Vec::new();

    let (word_prefix, is_dot_completion, obj_name, member_prefix) = uri.as_ref().and_then(|u| {
        let docs = backend.documents();
        let text = docs.get(u)?;
        let line_str = text.lines().nth(line)?;
        let byte_pos = crate::position::utf16_to_byte(line_str, character);
        let before_cursor = &line_str[..byte_pos];
        if let Some(dot_pos) = before_cursor.rfind('.') {
            let mut obj_start = dot_pos;
            let bytes = line_str.as_bytes();
            while obj_start > 0 && (crate::resolver::is_ident_char(bytes[obj_start - 1]) || bytes[obj_start - 1] == b'.') {
                obj_start -= 1;
            }
            let obj_name = line_str[obj_start..dot_pos].to_string();
            let member_prefix = line_str[dot_pos + 1..byte_pos].to_string();
            Some((String::new(), true, obj_name, member_prefix))
        } else if before_cursor.ends_with("::") {
            let colon_pos = before_cursor.rfind("::").unwrap_or(0);
            let mut obj_start = colon_pos;
            let bytes = line_str.as_bytes();
            while obj_start > 0 && crate::resolver::is_ident_char(bytes[obj_start - 1]) {
                obj_start -= 1;
            }
            let obj_name = line_str[obj_start..colon_pos].to_string();
            let member_prefix = line_str[colon_pos + 2..byte_pos].to_string();
            Some((String::new(), true, obj_name, member_prefix))
        } else {
            Some((extract_word(line_str, byte_pos), false, String::new(), String::new()))
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
            if kw.starts_with(&word_prefix) || word_prefix.is_empty() {
                items.push(serde_json::json!({ "label": kw, "kind": 14, "insertText": kw }));
            }
        }
    }

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
                        "kind": kind, "detail": *label,
                        "insertText": snippet, "insertTextFormat": 2
                    }));
                }
            }
        }

        items.push(serde_json::json!({ "label": "self", "kind": 14, "detail": "method receiver", "insertText": "self" }));

        let std_modules = vec!["xiom", "io", "math", "string", "collections", "fs", "net", "time", "json", "test"];
        for m in &std_modules {
            if m.starts_with(&word_prefix) || word_prefix.is_empty() {
                items.push(serde_json::json!({ "label": m, "kind": 2, "detail": "module", "insertText": m }));
            }
        }
    }

    if let Some(ref u) = uri {
        let docs = backend.documents();
        if let Some(text) = docs.get(u) {
            let mut lexer = xiom_lexer::Lexer::new(text);
            let tokens = lexer.tokenize();
            let mut parser = xiom_parser::Parser::new(tokens);
            if let Ok(program) = parser.parse_program() {
                if !is_dot_completion {
                    for item in &program.items {
                        collect_symbols(item, &mut items, &word_prefix);
                    }
                    for item in &program.items {
                        if let xiom_ast::TopDecl::Enum(ed) = item {
                            for variant in &ed.variants {
                                let label = format!("{}::{}", ed.name.name, variant.name.name);
                                if word_prefix.is_empty() || label.starts_with(&word_prefix) {
                                    let mut detail = format!("variant of {}", ed.name.name);
                                    if !variant.fields.is_empty() {
                                        let fds: Vec<String> = variant.fields.iter()
                                            .map(|f| format!("{}: {}", f.name.name, crate::resolver::type_to_string(&f.ty)))
                                            .collect();
                                        detail = format!("{}({})", detail, fds.join(", "));
                                    }
                                    items.push(serde_json::json!({
                                        "label": &label, "kind": 22, "detail": detail, "insertText": &label
                                    }));
                                }
                            }
                        }
                    }
                }

                if is_dot_completion && !obj_name.is_empty() {
                    if let Some(type_name) = resolve_obj_type_text(&program, &obj_name) {
                        let fields = find_struct_fields_in_program(&program, &type_name);
                        for (field_name, field_type) in &fields {
                            if member_prefix.is_empty() || field_name.starts_with(&member_prefix) {
                                items.push(serde_json::json!({
                                    "label": field_name, "kind": 5, "detail": field_type, "insertText": field_name
                                }));
                            }
                        }
                        let methods = find_methods_in_program(&program, &type_name);
                        for (method_name, sig) in &methods {
                            if member_prefix.is_empty() || method_name.starts_with(&member_prefix) {
                                items.push(serde_json::json!({
                                    "label": method_name, "kind": 2, "detail": sig, "insertText": format!("{}(", method_name)
                                }));
                            }
                        }
                        let iface_methods = find_interface_methods_for_type(&program, &type_name);
                        for (method_name, sig) in &iface_methods {
                            if member_prefix.is_empty() || method_name.starts_with(&member_prefix) {
                                items.push(serde_json::json!({
                                    "label": method_name, "kind": 2, "detail": sig, "insertText": format!("{}(", method_name)
                                }));
                            }
                        }
                        let enum_variants = collect_enum_variants_for_type(&program, &type_name);
                        for (variant_label, variant_detail) in &enum_variants {
                            if member_prefix.is_empty() || variant_label.starts_with(&member_prefix) {
                                items.push(serde_json::json!({
                                    "label": variant_label, "kind": 22, "detail": variant_detail, "insertText": variant_label
                                }));
                            }
                        }
                    }
                    collect_module_members(&program, &obj_name, &member_prefix, &mut items);
                }
            }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": items }));
}

pub fn handle_definition(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let mut location = None;

    if let Some(ref u) = uri {
        let word = {
            let docs = backend.documents();
            if let Some(text) = docs.get(u) {
                let line_str = text.lines().nth(line).unwrap_or("");
                extract_word(line_str, crate::position::utf16_to_byte(line_str, character))
            } else { String::new() }
        };

        if !word.is_empty() {
            let text = { backend.documents().get(u).cloned() };
            if let Some(text) = text {
                if let Some(program) = backend.parse_cached(u, &text) {
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

        // Stage 5: CROSS-FILE definition. When the symbol is not declared in
        // the current document, search the other open documents' cached ASTs
        // (deterministic uri order) and return that file's declaration.
        if location.is_none() && !word.is_empty() {
            let mut others: Vec<(String, String)> = {
                let docs = backend.documents();
                docs.iter()
                    .filter(|(k, _)| uri.as_deref() != Some(k.as_str()))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            };
            others.sort_by(|a, b| a.0.cmp(&b.0));
            for (other_uri, text) in others {
                let Some(program) = backend.parse_cached(&other_uri, &text) else { continue };
                if let Some(pos) = find_definition(&program, &word) {
                    location = Some(serde_json::json!({
                        "uri": other_uri,
                        "range": {
                            "start": { "line": pos.0, "character": pos.1 },
                            "end": { "line": pos.0, "character": pos.1 + word.len() as u64 }
                        }
                    }));
                    break;
                }
            }
        }

        // Stage 5: cross-file index. A module that was never opened in the
        // editor still resolves through the lazily built project index
        // (rebuilt once per document-lifecycle event, not per request).
        if location.is_none() && !word.is_empty() {
            let roots = project_index_roots(u);
            if !roots.is_empty() {
                if let Some((target_uri, line, col)) = backend.lookup_declaration(&roots, &word) {
                    location = Some(serde_json::json!({
                        "uri": target_uri,
                        "range": {
                            "start": { "line": line, "character": col },
                            "end": { "line": line, "character": col + word.len() as u64 }
                        }
                    }));
                }
            }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": location }));
}

/// Stage 5 (cross-file index): project roots the open document belongs to,
/// mirroring the checker's root discovery (document dir, `src/` of the
/// project root, graph source roots). The stdlib is deliberately excluded:
/// it is large, and catalog symbols are served by the checker's own index.
/// The document's own directory covers sibling modules; broader parents are
/// NOT added (a file in the OS temp dir must not index all of /tmp).
fn project_index_roots(uri: &str) -> Vec<String> {
    let mut roots = Vec::new();
    if let Some(dir) = crate::uri::uri_to_parent_dir(uri) {
        roots.push(dir);
    }
    if let Some(file_path) = crate::uri::uri_to_file_path(uri) {
        if let Some(root) = xiom::find_project_root(&file_path) {
            let src_dir = root.join("src");
            if src_dir.is_dir() {
                roots.push(src_dir.to_string_lossy().to_string());
            }
        }
        if let Ok(graph) = xiom_graph::build_project_graph(&file_path) {
            for root in &graph.source_roots {
                roots.push(root.to_string_lossy().to_string());
            }
        }
    }
    roots
}

pub fn handle_signature_help(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let character = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let mut signatures = Vec::new();
    let mut active_parameter = 0;

    let docs = backend.documents();
    if let Some(text) = docs.get(uri) {
        let line_str = text.lines().nth(line).unwrap_or("");
        let before_cursor = &line_str[..crate::position::utf16_to_byte(line_str, character)];
        if let Some(paren_pos) = before_cursor.rfind('(') {
            let before_paren = &before_cursor[..paren_pos];
            let fn_name = before_paren.split_whitespace().last().unwrap_or("").trim();
            if !fn_name.is_empty() {
                let mut lexer = xiom_lexer::Lexer::new(text);
                let tokens = lexer.tokenize();
                let mut parser = xiom_parser::Parser::new(tokens);
                if let Ok(program) = parser.parse_program() {
                    if let Some(sig) = find_function_signature(&program, fn_name) {
                        signatures.push(serde_json::json!({ "label": sig, "documentation": "" }));
                    }
                }
            }

            let after_paren = &before_cursor[paren_pos + 1..];
            let mut comma_count = 0;
            let mut depth_paren = 0;
            let mut depth_brace = 0;
            let mut in_string = false;
            let mut string_char = '"';

            for c in after_paren.chars() {
                if in_string {
                    if c == string_char { in_string = false; }
                    continue;
                }
                match c {
                    '"' | '\'' => { in_string = true; string_char = c; }
                    '(' | '[' => depth_paren += 1,
                    ')' | ']' => { if depth_paren > 0 { depth_paren -= 1; } }
                    '{' => depth_brace += 1,
                    '}' => { if depth_brace > 0 { depth_brace -= 1; } }
                    ',' if depth_paren == 0 && depth_brace == 0 => { comma_count += 1; }
                    _ => {}
                }
            }
            active_parameter = comma_count;
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": { "signatures": signatures, "activeSignature": 0, "activeParameter": active_parameter }
    }));
}

pub fn handle_document_symbols(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().map(|s| s.to_string());
    let mut symbols = Vec::new();

    if let Some(ref u) = uri {
        let text = { backend.documents().get(u).cloned() };
        if let Some(text) = text {
            if let Some(program) = backend.parse_cached(u, &text) {
                for item in &program.items {
                    collect_document_symbols(item, &mut symbols);
                }
            }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": symbols }));
}

pub fn handle_references(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let pos = &msg["params"]["position"];
    let line = pos["line"].as_u64().unwrap_or(0) as usize;
    let col = pos["character"].as_u64().unwrap_or(0) as usize;

    let mut locations = Vec::new();
    if let Some(ident) = find_ident_at(backend, uri, line, col) {
        let docs = backend.documents();
        if let Some(text) = docs.get(uri) {
            for (ln, line_text) in text.lines().enumerate() {
                let mut search_start = 0;
                while let Some(pos) = line_text[search_start..].find(&ident) {
                    let abs_col = search_start + pos;
                    let before = line_text[..abs_col].chars().last().map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true);
                    let after = line_text[abs_col + ident.len()..].chars().next().map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true);
                    if before && after {
                        locations.push(serde_json::json!({
                            "uri": uri,
                            "range": {
                                "start": {"line": ln, "character": crate::position::byte_to_utf16(line_text, abs_col)},
                                "end": {"line": ln, "character": crate::position::byte_to_utf16(line_text, abs_col + ident.len())}
                            }
                        }));
                    }
                    search_start = abs_col + ident.len();
                }
            }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": locations }));
}

pub fn handle_rename(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let pos = &msg["params"]["position"];
    let line = pos["line"].as_u64().unwrap_or(0) as usize;
    let col = pos["character"].as_u64().unwrap_or(0) as usize;
    let new_name = msg["params"]["newName"].as_str().unwrap_or("");

    let mut edits = Vec::new();
    if !new_name.is_empty() {
        if let Some(ident) = find_ident_at(backend, uri, line, col) {
            let mut docs = backend.documents();
            if let Some(text) = docs.get_mut(uri) {
                let mut text_edits = Vec::new();
                for (ln, line_text) in text.lines().enumerate() {
                    let mut search_start = 0;
                    while let Some(pos) = line_text[search_start..].find(&ident) {
                        let abs_col = search_start + pos;
                        let before = line_text[..abs_col].chars().last().map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true);
                        let after = line_text[abs_col + ident.len()..].chars().next().map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true);
                        if before && after {
                            text_edits.push(serde_json::json!({
                                "range": {
                                    "start": {"line": ln, "character": crate::position::byte_to_utf16(line_text, abs_col)},
                                    "end": {"line": ln, "character": crate::position::byte_to_utf16(line_text, abs_col + ident.len())}
                                },
                                "newText": new_name
                            }));
                        }
                        search_start = abs_col + ident.len();
                    }
                }
                if !text_edits.is_empty() {
                    edits.push(serde_json::json!({
                        "textDocument": {"uri": uri, "version": null},
                        "edits": text_edits
                    }));
                }
            }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": {"changes": edits}
    }));
}

pub fn handle_semantic_tokens(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let tokens = compute_semantic_tokens(backend, uri);
    let id = msg["id"].clone();
    responses.push(serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": { "data": tokens }
    }));
}

pub fn handle_code_action(msg: &serde_json::Value, responses: &mut Vec<serde_json::Value>) {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let diagnostics = msg["params"]["context"]["diagnostics"].as_array()
        .cloned().unwrap_or_default();

    let mut actions = Vec::new();
    for diag in &diagnostics {
        let code = diag["code"].as_str().unwrap_or("");
        let msg_text = diag["message"].as_str().unwrap_or("");
        let line = diag["range"]["start"]["line"].as_u64().unwrap_or(0);
        let col = diag["range"]["start"]["character"].as_u64().unwrap_or(0);

        if code == "T001" && msg_text.contains("expected") && msg_text.contains("found") {
            actions.push(serde_json::json!({
                "title": format!("Fix type mismatch: add explicit cast for '{}'", msg_text),
                "kind": "quickfix", "diagnostics": [diag],
                "edit": { "changes": { uri: [{
                    "range": { "start": {"line": line, "character": col}, "end": {"line": line, "character": col + 1} },
                    "newText": format!("/* FIX: type mismatch -- {} */", msg_text)
                }] } }
            }));
        }

        if code.starts_with("X") {
            actions.push(serde_json::json!({
                "title": format!("[{}] {}", code, msg_text),
                "kind": "quickfix", "diagnostics": [diag],
                "edit": { "changes": { uri: [{
                    "range": { "start": {"line": line, "character": 0}, "end": {"line": line, "character": 0} },
                    "newText": format!("// FIX [{}]: {}\n", code, msg_text)
                }] } }
            }));
        }

        if code == "P001" {
            actions.push(serde_json::json!({
                "title": format!("Fix parse error: {}", msg_text),
                "kind": "quickfix", "diagnostics": [diag],
                "edit": { "changes": { uri: [{
                    "range": { "start": {"line": line, "character": col}, "end": {"line": line, "character": col + 1} },
                    "newText": format!("/* P001: {} */", msg_text)
                }] } }
            }));
        }

        if code == "E001" && msg_text.contains("moved") {
            actions.push(serde_json::json!({
                "title": format!("Fix borrow error: consider .clone() for '{}'", msg_text),
                "kind": "quickfix", "diagnostics": [diag],
                "edit": { "changes": { uri: [{
                    "range": { "start": {"line": line, "character": col}, "end": {"line": line, "character": col} },
                    "newText": "/* E001: use .clone() or restructure borrows */"
                }] } }
            }));
        }

        actions.push(serde_json::json!({
            "title": format!("Run xiom --ai to diagnose: {}", msg_text),
            "kind": "quickfix", "diagnostics": [diag],
            "command": { "title": "AI Diagnose", "command": "xiom.ai.diagnose", "arguments": [uri, line, col, msg_text] }
        }));
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": actions }));
}

pub fn handle_workspace_symbol(msg: &serde_json::Value, backend: &Backend, responses: &mut Vec<serde_json::Value>) {
    let query = msg["params"]["query"].as_str().unwrap_or("");
    let mut symbols = Vec::new();

    {
        // Stage 5: use the per-uri parse cache (workspace/symbol used to
        // re-lex+re-parse every open document on every request).
        let docs: Vec<(String, String)> = {
            let docs = backend.documents();
            docs.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };
        for (uri, text) in docs {
            let Some(program) = backend.parse_cached(&uri, &text) else { continue };
            for item in &program.items {
                collect_workspace_symbols(item, &uri, query, &mut symbols);
                if symbols.len() >= 50 { break; }
            }
            if symbols.len() >= 50 { break; }
        }
    }

    let id = msg["id"].clone();
    responses.push(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": symbols }));
}
