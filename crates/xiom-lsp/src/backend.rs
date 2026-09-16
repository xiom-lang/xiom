// XIOM Language Server -- Backend
// Document storage and diagnostic publishing.
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use xiom_lexer::Lexer;
use xiom_parser::Parser;

use crate::diagnostics::{diagnostic_from_check_error, diagnostic_from_parse_error};
use crate::uri::uri_to_parent_dir;
use crate::uri::uri_to_file_path;

pub struct Backend {
    pub(crate) documents: Arc<Mutex<HashMap<String, String>>>,
    /// Stage 5 (LSP incremental tier): parsed AST per uri keyed by a content
    /// hash, so hover/completion/symbols/definition do not re-lex and
    /// re-parse unchanged text on every request. `didChange` updates the
    /// document text; the next request re-parses only that document and the
    /// stale entry is replaced.
    pub(crate) parsed: Arc<Mutex<HashMap<String, (u64, xiom_ast::Program)>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            parsed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn text_hash(text: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut h);
        h.finish()
    }

    /// Parsed program for `text`, cached per uri and invalidated by content
    /// hash. Returns None when the text does not parse (errors are not
    /// cached: partial programs are recovered by the parser itself).
    pub fn parse_cached(&self, uri: &str, text: &str) -> Option<xiom_ast::Program> {
        let hash = Self::text_hash(text);
        {
            let cache = self.parsed.lock().unwrap_or_else(|p| p.into_inner());
            if let Some((cached_hash, program)) = cache.get(uri) {
                if *cached_hash == hash {
                    return Some(program.clone());
                }
            }
        }
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(program) => {
                let mut cache = self.parsed.lock().unwrap_or_else(|p| p.into_inner());
                cache.insert(uri.to_string(), (hash, program.clone()));
                Some(program)
            }
            Err(_) => None,
        }
    }

    /// Poison-safe document store access. A panic in one handler used to
    /// poison the mutex, after which every later request panicked on the
    /// 13 `expect("document store mutex poisoned")` sites and the server
    /// stopped responding. Recovering the guard keeps the session alive
    /// (audit: mutex-poison recovery).
    pub(crate) fn documents(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        self.documents.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn publish_diagnostics(&self, uri: &str) -> Vec<serde_json::Value> {
        let text = {
            let docs = self.documents();
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
                let (line, ch) = crate::position::span_start_lsp(&text, &tok.span);
                diagnostics.push(serde_json::json!({
                    "range": {
                        "start": { "line": line, "character": ch },
                        "end": { "line": line, "character": ch + 1 }
                    },
                    // AUDIT #9 FIX: LSP severity is an INTEGER (1=Error,
                    // 2=Warning, 3=Information, 4=Hint). The string "Error"
                    // violated the spec and was dropped by conforming
                    // clients, silencing lex errors entirely.
                    "severity": 1,
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
                    diagnostics.push(diagnostic_from_parse_error(&text, err));
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
                        diagnostics.push(diagnostic_from_check_error(&text, err));
                    }
                }
            }
            Err(err) => {
                diagnostics.push(diagnostic_from_parse_error(&text, &err));
            }
        }

        diagnostics
    }
}
