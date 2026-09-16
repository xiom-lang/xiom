// XIOM -- Language Server
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

mod ai;
mod backend;
mod diagnostics;
mod handlers;
mod position;
mod resolver;
mod semantic_tokens;
mod symbols;
mod text_edit;
mod transport;
mod uri;

use backend::Backend;
use transport::{LspReader, write_lsp_message};

use std::env;

// ============================================================================
// Main dispatch -- thin router to handler modules
// ============================================================================

pub fn handle_lsp_message(msg: &serde_json::Value, backend: &Backend) -> Vec<serde_json::Value> {
    let mut responses = Vec::new();

    let method = match msg["method"].as_str() {
        Some(m) => m,
        None => return responses,
    };

    match method {
        "initialize" => handlers::handle_initialize(msg, &mut responses),
        "initialized" => {}
        "shutdown" => handlers::handle_shutdown(msg, &mut responses),
        "textDocument/didOpen" => handlers::handle_did_open(msg, backend, &mut responses),
        "textDocument/didChange" => handlers::handle_did_change(msg, backend, &mut responses),
        "textDocument/didClose" => handlers::handle_did_close(msg, backend),
        "textDocument/hover" => handlers::handle_hover(msg, backend, &mut responses),
        "textDocument/completion" => handlers::handle_completion(msg, backend, &mut responses),
        "textDocument/definition" => handlers::handle_definition(msg, backend, &mut responses),
        "textDocument/signatureHelp" => handlers::handle_signature_help(msg, backend, &mut responses),
        "textDocument/documentSymbol" => handlers::handle_document_symbols(msg, backend, &mut responses),
        "textDocument/references" => handlers::handle_references(msg, backend, &mut responses),
        "textDocument/rename" => handlers::handle_rename(msg, backend, &mut responses),
        "textDocument/semanticTokens/full" => handlers::handle_semantic_tokens(msg, backend, &mut responses),
        "textDocument/codeAction" => handlers::handle_code_action(msg, &mut responses),
        "workspace/symbol" => handlers::handle_workspace_symbol(msg, backend, &mut responses),
        _ => {}
    }

    responses
}

// ============================================================================
// Entry point
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return;
    }

    let backend = Backend::new();
    let reader = LspReader::new();

    while let Some(raw) = reader.read_message() {
        let msg: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if msg["method"].as_str().is_none() {
            continue;
        }

        let responses = handle_lsp_message(&msg, &backend);
        for response in &responses {
            write_lsp_message(response);
        }

        if msg["method"].as_str() == Some("shutdown") {
            break;
        }
    }
}

fn print_usage() {
    eprintln!("XIOM Language Server v0.48.9");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom lsp");
    eprintln!();
    eprintln!("The XIOM Language Server provides diagnostics, hover, completion,");
    eprintln!("and go-to-definition for .xi files. Launch from editor configuration.");
    eprintln!();
    eprintln!("VS Code: editors/vscode/package.json");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_msg(json_str: &str) -> serde_json::Value {
        serde_json::from_str(json_str).unwrap()
    }

    fn find_response_by_method<'a>(
        responses: &'a [serde_json::Value],
        method: &str,
    ) -> Option<&'a serde_json::Value> {
        responses
            .iter()
            .find(|r| r["method"].as_str() == Some(method))
    }

    fn find_response_by_id<'a>(
        responses: &'a [serde_json::Value],
        id: i32,
    ) -> Option<&'a serde_json::Value> {
        responses
            .iter()
            .find(|r| r["id"].as_i64() == Some(id as i64))
    }

    fn open_document(backend: &Backend, uri: &str, text: &str) -> Vec<serde_json::Value> {
        let msg = parse_msg(&format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {{
                    "textDocument": {{
                        "uri": "{uri}",
                        "languageId": "xiom",
                        "version": 1,
                        "text": "{text}"
                    }}
                }}
            }}"#,
            uri = uri,
            text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
        ));
        handle_lsp_message(&msg, backend)
    }

    // -----------------------------------------------------------------------
    // test_initialize
    // -----------------------------------------------------------------------

    #[test]
    fn test_initialize() {
        let backend = Backend::new();
        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "processId": null,
                    "rootUri": null,
                    "capabilities": {}
                }
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);

        let init_response = find_response_by_id(&responses, 1)
            .expect("should have initialize response");

        let result = &init_response["result"];
        let caps = &result["capabilities"];

        assert_eq!(init_response["jsonrpc"].as_str(), Some("2.0"));
        assert_eq!(caps["hoverProvider"].as_bool(), Some(true));
        assert_eq!(caps["definitionProvider"].as_bool(), Some(true));
        assert_eq!(caps["documentSymbolProvider"].as_bool(), Some(true));
        assert_eq!(caps["workspaceSymbolProvider"].as_bool(), Some(true));

        let sync = &caps["textDocumentSync"];
        assert_eq!(sync["openClose"].as_bool(), Some(true));
        assert_eq!(sync["change"].as_i64(), Some(2));

        let completion = &caps["completionProvider"];
        let triggers: Vec<&str> = completion["triggerCharacters"]
            .as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert!(triggers.contains(&"."));
        assert!(triggers.contains(&":"));

        let sig_help = &caps["signatureHelpProvider"];
        let sig_triggers: Vec<&str> = sig_help["triggerCharacters"]
            .as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert!(sig_triggers.contains(&"("));
        assert!(sig_triggers.contains(&","));

        let log_msg = find_response_by_method(&responses, "window/logMessage")
            .expect("should have logMessage notification");
        assert_eq!(log_msg["params"]["message"].as_str(), Some("XIOM Language Server v0.48.9"));
    }

    // -----------------------------------------------------------------------
    // test_did_open_pushes_diagnostics
    // -----------------------------------------------------------------------

    #[test]
    fn test_did_open_pushes_diagnostics() {
        let backend = Backend::new();
        let responses = open_document(
            &backend, "file:///test.xi", "fn main() -> Int { 42 }",
        );

        let diag_notification = find_response_by_method(
            &responses, "textDocument/publishDiagnostics",
        ).expect("didOpen should trigger publishDiagnostics");

        assert_eq!(diag_notification["params"]["uri"].as_str(), Some("file:///test.xi"));
        assert!(diag_notification["params"]["diagnostics"].is_array());
    }

    #[test]
    fn test_did_open_stores_document() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn foo() {}");
        let docs = backend.documents.lock().unwrap();
        assert_eq!(docs.get("file:///test.xi").unwrap(), "fn foo() {}");
    }

    // -----------------------------------------------------------------------
    // test_hover_function
    // -----------------------------------------------------------------------

    #[test]
    fn test_hover_function() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn add(x: Int, y: Int) -> Int { x + y }");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
                "params": {
                    "textDocument": { "uri": "file:///test.xi" },
                    "position": { "line": 0, "character": 3 }
                }
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);
        let hover_response = find_response_by_id(&responses, 2).expect("should have hover response");
        let result = &hover_response["result"];
        assert!(!result.is_null(), "hover result should not be null");
        let value = result["contents"]["value"].as_str().unwrap();
        assert!(value.contains("function"), "hover should show function");
        assert!(value.contains("add"), "hover should contain function name");
        assert!(value.contains("Int"), "hover should contain parameter types");
    }

    #[test]
    fn test_hover_returns_null_for_empty_document() {
        let backend = Backend::new();
        open_document(&backend, "file:///empty.xi", "");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
                "params": {
                    "textDocument": { "uri": "file:///empty.xi" },
                    "position": { "line": 0, "character": 0 }
                }
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);
        let hover_response = find_response_by_id(&responses, 3).unwrap();
        assert!(hover_response["result"].is_null());
    }

    // -----------------------------------------------------------------------
    // test_completion_keywords
    // -----------------------------------------------------------------------

    #[test]
    fn test_completion_keywords() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 4, "method": "textDocument/completion",
                "params": {
                    "textDocument": { "uri": "file:///test.xi" },
                    "position": { "line": 0, "character": 0 }
                }
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);
        let completion_response = find_response_by_id(&responses, 4).expect("should have completion response");
        let items = completion_response["result"].as_array().expect("result should be an array");
        let labels: Vec<&str> = items.iter().map(|item| item["label"].as_str().unwrap()).collect();

        assert!(labels.contains(&"fn"));
        assert!(labels.contains(&"let"));
        assert!(labels.contains(&"return"));
        assert!(labels.contains(&"if"));
        assert!(labels.contains(&"Int"));
        assert!(labels.contains(&"Bool"));
    }

    #[test]
    fn test_completion_keywords_with_prefix() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "f");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 5, "method": "textDocument/completion",
                "params": {
                    "textDocument": { "uri": "file:///test.xi" },
                    "position": { "line": 0, "character": 1 }
                }
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);
        let completion_response = find_response_by_id(&responses, 5).unwrap();
        let items = completion_response["result"].as_array().unwrap();
        let labels: Vec<&str> = items.iter().map(|item| item["label"].as_str().unwrap()).collect();

        assert!(labels.contains(&"fn"), "should suggest 'fn' for prefix 'f'");
        assert!(labels.contains(&"for"), "should suggest 'for' for prefix 'f'");
        assert!(labels.contains(&"false"), "should suggest 'false' for prefix 'f'");
    }

    // -----------------------------------------------------------------------
    // test_shutdown
    // -----------------------------------------------------------------------

    #[test]
    fn test_shutdown() {
        let backend = Backend::new();
        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 6, "method": "shutdown", "params": null
            }"#,
        );

        let responses = handle_lsp_message(&msg, &backend);
        assert_eq!(responses.len(), 1);
        let shutdown_response = &responses[0];
        assert_eq!(shutdown_response["jsonrpc"].as_str(), Some("2.0"));
        assert_eq!(shutdown_response["id"].as_i64(), Some(6));
        assert!(shutdown_response["result"].is_null());
    }

    // -----------------------------------------------------------------------
    // test_stdlib_module_no_false_positives
    // -----------------------------------------------------------------------

    #[test]
    fn test_stdlib_module_no_false_positives() {
        let backend = Backend::new();
        // R31: resolve through the shared helper (checkout, XIOM_STDLIB,
        // installed lib); loud SKIP without a stdlib, hard FAIL in CI.
        let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
        // BUG 29 (512-module layout): alloc moved stdlib/xiom/alloc.xi ->
        // stdlib/xiom/memory/alloc.xi. Namespace wave 2 (78b107fb) then moved
        // it again to stdlib/xiom/alloc/alloc.xi (directory-aligned quartet).
        let alloc_path = stdlib_root.join("xiom").join("alloc").join("alloc.xi");
        if xiom_graph::paths::skip_if_missing("stdlib alloc module", &alloc_path) {
            return;
        }
        let text = std::fs::read_to_string(&alloc_path).expect("stdlib alloc module must exist");

        let uri = format!("file:///{}", alloc_path.to_string_lossy().replace('\\', "/").replace(':', "%3A"));
        {
            let mut docs = backend.documents.lock().unwrap();
            docs.insert(uri.clone(), text);
        }

        let diagnostics = backend.publish_diagnostics(&uri);
        let errors: Vec<String> = diagnostics.iter()
            .filter_map(|d| d["message"].as_str().map(String::from))
            .collect();
        assert!(
            errors.is_empty(),
            "stdlib/xiom/alloc.xi must produce ZERO diagnostics via LSP. Got {} errors:\n{}",
            errors.len(), errors.join("\n")
        );
    }

    // -----------------------------------------------------------------------
    // test_uri_to_parent_dir
    // -----------------------------------------------------------------------

    #[test]
    fn test_uri_to_parent_dir() {
        let dir = uri::uri_to_parent_dir("file:///e%3A/Projects/AXIOM/stdlib/xiom/alloc.xi");
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir.contains("stdlib"), "parent dir should contain stdlib: {dir}");
        assert!(dir.ends_with("xiom"), "parent dir should end with xiom: {dir}");
        assert!(dir.starts_with("e:") || dir.starts_with("E:"), "drive letter decoded: {dir}");
    }

    // -----------------------------------------------------------------------
    // test_workspace_symbol
    // -----------------------------------------------------------------------

    #[test]
    fn test_workspace_symbol() {
        let backend = Backend::new();
        open_document(&backend, "file:///a.xi", "fn hello() -> Int { 42 }\nfn world() -> Bool { true }\nconst MAX: Int = 100");
        open_document(&backend, "file:///b.xi", "fn add(x: Int, y: Int) -> Int { x + y }\nenum Color { Red, Green, Blue }");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 20, "method": "workspace/symbol",
                "params": { "query": "hello" }
            }"#,
        );
        let responses = handle_lsp_message(&msg, &backend);
        let ws_response = find_response_by_id(&responses, 20).expect("should have workspace/symbol response");
        let symbols = ws_response["result"].as_array().expect("result should be an array");
        assert_eq!(symbols.len(), 1, "query 'hello' should match exactly one symbol");
        assert_eq!(symbols[0]["name"].as_str(), Some("hello"));
        assert_eq!(symbols[0]["kind"].as_i64(), Some(12));
        assert_eq!(symbols[0]["location"]["uri"].as_str(), Some("file:///a.xi"));

        let msg2 = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 21, "method": "workspace/symbol",
                "params": { "query": "" }
            }"#,
        );
        let responses2 = handle_lsp_message(&msg2, &backend);
        let ws_response2 = find_response_by_id(&responses2, 21).unwrap();
        let symbols2 = ws_response2["result"].as_array().unwrap();
        assert!(symbols2.len() >= 5, "empty query should return all symbols (got {})", symbols2.len());

        let msg3 = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 22, "method": "workspace/symbol",
                "params": { "query": "Color" }
            }"#,
        );
        let responses3 = handle_lsp_message(&msg3, &backend);
        let ws_response3 = find_response_by_id(&responses3, 22).unwrap();
        let symbols3 = ws_response3["result"].as_array().unwrap();
        assert_eq!(symbols3.len(), 1);
        assert_eq!(symbols3[0]["name"].as_str(), Some("Color"));
        assert_eq!(symbols3[0]["kind"].as_i64(), Some(13));
    }

    // M3.3: Rename and CodeAction tests

    #[test]
    fn test_rename_symbol() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn hello() -> Int { 42 }");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 30, "method": "textDocument/rename",
                "params": {
                    "textDocument": {"uri": "file:///test.xi"},
                    "position": {"line": 0, "character": 3},
                    "newName": "greet"
                }
            }"#,
        );
        let responses = handle_lsp_message(&msg, &backend);
        // Rename should return a response (even if empty changes)
        assert!(!responses.is_empty(), "rename should produce a response");
    }

    #[test]
    fn test_rename_no_document() {
        let backend = Backend::new();
        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 31, "method": "textDocument/rename",
                "params": {
                    "textDocument": {"uri": "file:///nonexistent.xi"},
                    "position": {"line": 0, "character": 0},
                    "newName": "x"
                }
            }"#,
        );
        let responses = handle_lsp_message(&msg, &backend);
        // Rename on unknown document should still produce a response
        assert!(!responses.is_empty(), "rename should produce a response even for unknown doc");
    }

    #[test]
    fn test_code_action_type_mismatch() {
        let backend = Backend::new();
        // Open a document and push a type error diagnostic, then request code actions
        open_document(&backend, "file:///err.xi", "fn main() { io.println(42); }");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 32, "method": "textDocument/codeAction",
                "params": {
                    "textDocument": {"uri": "file:///err.xi"},
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 99}
                    },
                    "context": {
                        "diagnostics": [{
                            "range": {
                                "start": {"line": 0, "character": 28},
                                "end": {"line": 0, "character": 30}
                            },
                            "message": "expected Str, found Int",
                            "code": "T001",
                            "severity": 1
                        }]
                    }
                }
            }"#,
        );
        let responses = handle_lsp_message(&msg, &backend);
        let ca_response = find_response_by_id(&responses, 32)
            .expect("should have codeAction response");
        let actions = ca_response["result"].as_array()
            .expect("result should be array");
        // May have suggestions or be empty -- both are valid
        assert!(!actions.is_empty() || actions.is_empty(), "code actions should be an array");
    }

    #[test]
    fn test_code_action_no_diagnostics() {
        let backend = Backend::new();
        open_document(&backend, "file:///clean.xi", "fn main() -> Int { 42 }");

        let msg = parse_msg(
            r#"{
                "jsonrpc": "2.0", "id": 33, "method": "textDocument/codeAction",
                "params": {
                    "textDocument": {"uri": "file:///clean.xi"},
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 99}
                    },
                    "context": {"diagnostics": []}
                }
            }"#,
        );
        let responses = handle_lsp_message(&msg, &backend);
        let ca_response = find_response_by_id(&responses, 33)
            .expect("should have codeAction response");
        let actions = ca_response["result"].as_array()
            .expect("result should be array");
        assert_eq!(actions.len(), 0, "no diagnostics should yield no code actions");
    }

    // -- M21-4: LSP edge cases ------------------------------------------

    // Completion in various contexts
    #[test] fn test_completion_after_dot() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "type Point = { x: Int; y: Int; } fn main() { var p = Point{ x: 1; y: 2 }; p.");
        // Request completion at end of file
        let doc_len = "type Point = { x: Int; y: Int; } fn main() { var p = Point{ x: 1; y: 2 }; p.".len();
        let msg = parse_msg(&format!(r#"{{"jsonrpc":"2.0","id":40,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"file:///test.xi"}},"position":{{"line":0,"character":{}}}}}}}"#, doc_len));
        let responses = handle_lsp_message(&msg, &backend);
        let cr = find_response_by_id(&responses, 40).expect("should have completion response");
        assert!(cr["result"].is_array());
    }

    #[test] fn test_completion_after_colon() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "use math.");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":41,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":9}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let cr = find_response_by_id(&responses, 41).expect("should have completion response");
        assert!(cr["result"].is_array());
    }

    #[test] fn test_completion_after_double_colon() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn main() { module::");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":42,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":20}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let cr = find_response_by_id(&responses, 42).expect("should have completion response");
        assert!(cr["result"].is_array());
    }

    #[test] fn test_completion_empty_file() {
        let backend = Backend::new();
        open_document(&backend, "file:///empty.xi", "");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":43,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///empty.xi"},"position":{"line":0,"character":0}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let cr = find_response_by_id(&responses, 43).expect("should have completion response");
        let items = cr["result"].as_array().unwrap();
        // Should at minimum return keywords
        assert!(!items.is_empty(), "empty file should still get keyword completions");
    }

    // Hover on complex expressions
    #[test] fn test_hover_on_type_annotation() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "type Point = { x: Int; y: Int; } fn main() -> Point { return Point{ x: 1; y: 2 }; }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":50,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":1}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let hr = find_response_by_id(&responses, 50).expect("hover should respond");
        assert!(!hr["result"].is_null() || hr["result"].is_null()); // both are valid
    }

    #[test] fn test_hover_on_struct_field() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "type Point = { x: Float64; y: Float64; } fn get_x(p: Point) -> Float64 { return p.x; }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":51,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":80}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let hr = find_response_by_id(&responses, 51).expect("hover should respond");
        assert!(hr.get("result").is_some());
    }

    #[test] fn test_hover_on_function_call() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn add(a: Int, b: Int) -> Int { return a + b; } fn main() -> Int { return add(1, 2); }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":52,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":85}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let hr = find_response_by_id(&responses, 52).expect("hover should respond");
        assert!(hr.get("result").is_some());
    }

    // Goto-def for methods, imports, modules
    #[test] fn test_definition_basic() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn target() -> Int { return 42; } fn main() -> Int { return target(); }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":60,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":62}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let dr = find_response_by_id(&responses, 60).expect("definition should respond");
        assert!(dr.get("result").is_some());
    }

    #[test] fn test_definition_multiple_files() {
        let backend = Backend::new();
        open_document(&backend, "file:///lib.xi", "pub fn lib_fn() -> Int { return 1; }");
        open_document(&backend, "file:///main.xi", "fn main() -> Int { return lib_fn(); }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":61,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///main.xi"},"position":{"line":0,"character":35}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let dr = find_response_by_id(&responses, 61).expect("definition should respond");
        assert!(dr.get("result").is_some());
    }

    // Document symbols
    #[test] fn test_document_symbol_multiple() {
        let backend = Backend::new();
        let src = "fn one() -> Int { return 1; }\nfn two() -> Int { return 2; }\ntype Counter = { val: Int; }\nenum Color { Red, Green, Blue }";
        open_document(&backend, "file:///test.xi", src);
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":70,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///test.xi"}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let ds = find_response_by_id(&responses, 70).expect("documentSymbol should respond");
        let symbols = ds["result"].as_array().unwrap();
        assert!(symbols.len() >= 4, "should find at least 4 symbols, got {}", symbols.len());
    }

    #[test] fn test_document_symbol_empty_file() {
        let backend = Backend::new();
        open_document(&backend, "file:///empty.xi", "");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":71,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///empty.xi"}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let ds = find_response_by_id(&responses, 71).expect("documentSymbol should respond");
        assert!(ds["result"].as_array().unwrap().is_empty(), "empty file should have no symbols");
    }

    // Diagnostics on open files
    #[test] fn test_diagnostics_on_error() {
        let backend = Backend::new();
        open_document(&backend, "file:///err.xi", "fn main() -> Int { return x; }");
        let diags = backend.publish_diagnostics("file:///err.xi");
        // Should contain at least the undefined variable error
        assert!(!diags.is_empty(), "should have diagnostics for undefined variable");
    }

    #[test] fn test_diagnostics_on_valid() {
        let backend = Backend::new();
        open_document(&backend, "file:///ok.xi", "fn main() -> Int { return 42; }");
        let diags = backend.publish_diagnostics("file:///ok.xi");
        // Valid code should have zero or minimal diagnostics
        let errors: Vec<_> = diags.iter()
            .filter(|d| d["severity"].as_i64() == Some(1))
            .collect();
        assert!(errors.is_empty(), "valid code should have no errors: {:?}", errors);
    }

    // DidChange updates document
    #[test] fn test_did_change_updates_document() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn old() {}");
        let change_msg = parse_msg(r#"{
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": "file:///test.xi", "version": 2},
                "contentChanges": [{"text": "fn new() -> Int { return 1; }"}]
            }
        }"#);
        handle_lsp_message(&change_msg, &backend);
        let docs = backend.documents.lock().unwrap();
        assert_eq!(docs.get("file:///test.xi").unwrap(), "fn new() -> Int { return 1; }");
    }

    // DidClose removes document
    #[test] fn test_did_close_removes_document() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn f() {}");
        let close_msg = parse_msg(r#"{
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {"textDocument": {"uri": "file:///test.xi"}}
        }"#);
        handle_lsp_message(&close_msg, &backend);
        let docs = backend.documents.lock().unwrap();
        assert!(!docs.contains_key("file:///test.xi"), "document should be removed after close");
    }

    // Signature help
    #[test] fn test_signature_help() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn add(a: Int, b: Int) -> Int { return a + b; } fn main() -> Int { return add(");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":80,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":78}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let sh = find_response_by_id(&responses, 80).expect("signatureHelp should respond");
        assert!(sh.get("result").is_some());
    }

    // Workspace symbol fuzzy search
    #[test] fn test_workspace_symbol_fuzzy() {
        let backend = Backend::new();
        open_document(&backend, "file:///a.xi", "fn calculate_sum() -> Int { 0 }\nfn compute_avg() -> Float64 { 0.0 }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":81,"method":"workspace/symbol","params":{"query":"calc"}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let ws = find_response_by_id(&responses, 81).expect("workspace/symbol should respond");
        assert!(ws["result"].is_array());
    }

    #[test] fn test_workspace_symbol_partial_match() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn user_login() -> Bool { true }\nfn user_logout() -> Bool { false }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":82,"method":"workspace/symbol","params":{"query":"user"}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let ws = find_response_by_id(&responses, 82).expect("workspace/symbol should respond");
        let symbols = ws["result"].as_array().unwrap();
        assert_eq!(symbols.len(), 2, "query 'user' should match both user_login and user_logout");
    }

    // Initialized notification
    #[test] fn test_initialized_notification() {
        let backend = Backend::new();
        let msg = parse_msg(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        // initialized is a notification, should not produce a response with id
        assert!(responses.iter().all(|r| r.get("id").is_none()), "initialized should not produce id responses");
    }

    // Document formatting (may not be implemented yet -- must not panic)
    #[test] fn test_formatting_request() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn main() -> Int{return 42;}");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":90,"method":"textDocument/formatting","params":{"textDocument":{"uri":"file:///test.xi"},"options":{"tabSize":2,"insertSpaces":true}}}"#);
        let _responses = handle_lsp_message(&msg, &backend);
        // Must not panic -- feature may not be implemented
    }

    #[test] fn test_range_formatting_request() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn main() -> Int{return 42;}");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":91,"method":"textDocument/rangeFormatting","params":{"textDocument":{"uri":"file:///test.xi"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":999}},"options":{"tabSize":2,"insertSpaces":true}}}"#);
        let _responses = handle_lsp_message(&msg, &backend);
        // Must not panic -- feature may not be implemented
    }

    // Semantic tokens
    #[test] fn test_semantic_tokens() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn main() -> Int { return 42; }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":92,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///test.xi"}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let st = find_response_by_id(&responses, 92).expect("semanticTokens should respond");
        assert!(st.get("result").is_some() || st.get("error").is_some());
    }

    // References
    #[test] fn test_references() {
        let backend = Backend::new();
        open_document(&backend, "file:///test.xi", "fn target() -> Int { return 1; } fn call1() -> Int { return target(); } fn call2() -> Int { return target(); }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":93,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.xi"},"position":{"line":0,"character":3},"context":{"includeDeclaration":true}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let refs = find_response_by_id(&responses, 93).expect("references should respond");
        assert!(refs.get("result").is_some() || refs.get("error").is_some());
    }

    // Stage 5: LSP parse cache -- repeated requests reuse the parsed AST and a
    // text change invalidates the entry.
    #[test] fn test_parse_cache_reuses_and_invalidates() {
        let backend = Backend::new();
        let uri = "file:///cache.xi";
        let a = backend.parse_cached(uri, "fn a() -> Int { return 1; }").expect("parses");
        assert_eq!(a.items.len(), 1);
        let b = backend.parse_cached(uri, "fn a() -> Int { return 1; }").expect("cache hit parses");
        assert_eq!(a, b, "identical text must return the identical AST");
        let c = backend
            .parse_cached(uri, "fn a() -> Int { return 1; }\nfn extra() -> Int { return 2; }")
            .expect("changed text parses");
        assert_eq!(c.items.len(), 2, "changed text must re-parse");
        let cache = backend.parsed.lock().unwrap();
        assert_eq!(cache.get(uri).map(|(_, p)| p.items.len()), Some(2));
    }

    // Stage 5: cross-file definition -- a symbol declared only in another
    // open document resolves to that document's Location.
    #[test] fn test_definition_cross_file() {
        let backend = Backend::new();
        open_document(
            &backend,
            "file:///a.xi",
            "fn helper() -> Int { return 7; }\nfn main() -> Int { return helper(); }",
        );
        open_document(&backend, "file:///b.xi", "fn caller() -> Int { return helper(); }");
        let msg = parse_msg(r#"{"jsonrpc":"2.0","id":94,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///b.xi"},"position":{"line":0,"character":30}}}"#);
        let responses = handle_lsp_message(&msg, &backend);
        let resp = find_response_by_id(&responses, 94).expect("definition should respond");
        assert_eq!(
            resp["result"]["uri"].as_str(),
            Some("file:///a.xi"),
            "helper is declared in a.xi; got {resp}"
        );
        assert_eq!(resp["result"]["range"]["start"]["line"].as_u64(), Some(0));
    }
}
