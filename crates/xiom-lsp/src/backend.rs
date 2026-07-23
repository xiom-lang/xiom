// XIOM Language Server — Backend
// Document storage and diagnostic publishing.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use xiom_lexer::Lexer;
use xiom_parser::Parser;

use crate::diagnostics::{diagnostic_from_check_error, diagnostic_from_parse_error};
use crate::uri::uri_to_parent_dir;
use crate::uri::uri_to_file_path;

pub struct Backend {
    pub(crate) documents: Arc<Mutex<HashMap<String, String>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn publish_diagnostics(&self, uri: &str) -> Vec<serde_json::Value> {
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
            .filter(|t| matches!(t.kind, xiom_lexer::TokenKind::Error(_)))
            .collect();
        for tok in &lex_errors {
            if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
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
                for err in parser.errors() {
                    diagnostics.push(diagnostic_from_parse_error(err));
                }
                let mut checker = xiom_check::Checker::new();
                if let Some(dir) = uri_to_parent_dir(uri) {
                    checker.add_source_dir(dir);
                }
                if let Some(file_path) = uri_to_file_path(uri) {
                    if let Some(grandparent) = file_path.parent().and_then(|p| p.parent()) {
                        if std::fs::read_dir(&grandparent).map_or(false, |entries| {
                            entries.flatten().any(|e| e.path().extension().map_or(false, |ext| ext == "xi"))
                        }) { checker.add_source_dir(grandparent.to_string_lossy().to_string()); }
                    }
                    if let Some(root) = xiom::find_project_root(&file_path) {
                        let src_dir = root.join("src");
                        if src_dir.is_dir() {
                            checker.add_source_dir(src_dir.to_string_lossy().to_string());
                        }
                    }
                    if let Ok(graph) = xiom_graph::build_project_graph(&file_path) {
                        for root in &graph.source_roots {
                            checker.add_source_dir(root.to_string_lossy().to_string());
                        }
                    }
                }
                for stdlib_dir in xiom::find_stdlib_dirs() {
                    checker.add_source_dir(stdlib_dir);
                }
                checker.build_catalog_index();
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
