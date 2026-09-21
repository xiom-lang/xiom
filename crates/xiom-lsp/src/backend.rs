// XIOM Language Server -- Backend
// Document storage and diagnostic publishing.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use xiom_lexer::Lexer;
use xiom_parser::Parser;

use crate::diagnostics::{diagnostic_from_check_error, diagnostic_from_parse_error};
use crate::uri::uri_to_parent_dir;
use crate::uri::uri_to_file_path;

/// Stage 5 (LSP cross-file index): declarations collected from the project
/// source roots the open document belongs to. Built lazily on the first
/// `textDocument/definition` that misses in the open documents, cached
/// across requests, and rebuilt only after a document-lifecycle event
/// (`mark_index_dirty`) or when the root set changes.
#[derive(Default)]
pub(crate) struct FileIndex {
    /// symbol -> declaration sites (uri, line, col), sorted and deduped.
    decls: HashMap<String, Vec<(String, u64, u64)>>,
    /// Sorted root set the index was built from.
    roots: Vec<String>,
    dirty: bool,
}

/// Bound the lazy rebuild: parse at most this many files per rebuild.
const MAX_INDEX_FILES: usize = 4000;
/// Recursion cap for project trees (symlink loops, nested checkouts).
const MAX_INDEX_DEPTH: usize = 24;

impl FileIndex {
    fn is_skipped_dir(name: &str) -> bool {
        matches!(
            name,
            "target" | ".git" | "node_modules" | ".kilo" | ".vscode" | "release" | "dist"
        )
    }

    fn collect_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
        if depth > MAX_INDEX_DEPTH || out.len() >= MAX_INDEX_FILES {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if out.len() >= MAX_INDEX_FILES {
                return;
            }
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if Self::is_skipped_dir(name) {
                        continue;
                    }
                }
                Self::collect_files(&path, out, depth + 1);
            } else if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                out.push(path);
            }
        }
    }

    fn rebuild(&mut self, roots: &[String]) {
        self.decls.clear();
        self.roots = roots.to_vec();
        self.dirty = false;
        let mut files: Vec<PathBuf> = Vec::new();
        for root in roots {
            let path = Path::new(root);
            if path.is_dir() {
                Self::collect_files(path, &mut files, 0);
            }
        }
        files.sort();
        files.dedup();
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else { continue };
            let mut lexer = Lexer::new(&text);
            let mut parser = Parser::new(lexer.tokenize());
            let Ok(program) = parser.parse_program() else { continue };
            let uri = crate::uri::path_to_uri(&file);
            for (name, line, col) in crate::symbols::collect_declarations(&program) {
                self.decls
                    .entry(name)
                    .or_default()
                    .push((uri.clone(), line, col));
            }
        }
        for sites in self.decls.values_mut() {
            sites.sort();
            sites.dedup();
        }
    }
}

pub struct Backend {
    pub(crate) documents: Arc<Mutex<HashMap<String, String>>>,
    /// Stage 5 (LSP incremental tier): parsed AST per uri keyed by a content
    /// hash, so hover/completion/symbols/definition do not re-lex and
    /// re-parse unchanged text on every request. `didChange` updates the
    /// document text; the next request re-parses only that document and the
    /// stale entry is replaced.
    pub(crate) parsed: Arc<Mutex<HashMap<String, (u64, xiom_ast::Program)>>>,
    /// Stage 5 (LSP cross-file index): on-disk declarations for requests
    /// that miss in the open documents.
    pub(crate) file_index: Arc<Mutex<FileIndex>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            parsed: Arc::new(Mutex::new(HashMap::new())),
            file_index: Arc::new(Mutex::new(FileIndex::default())),
        }
    }

    /// Document lifecycle changed (open/change/close/save): the on-disk
    /// index may no longer reflect the editor state. The next definition
    /// request rebuilds it once.
    pub(crate) fn mark_index_dirty(&self) {
        let mut index = self.file_index.lock().unwrap_or_else(|p| p.into_inner());
        index.dirty = true;
    }

    /// First declaration of `symbol` in the cached cross-file index for
    /// `roots`. Rebuilds lazily when dirty or when the root set changed.
    /// Deterministic: files are traversed in sorted order and sites sorted.
    pub fn lookup_declaration(&self, roots: &[String], symbol: &str) -> Option<(String, u64, u64)> {
        let mut wanted: Vec<String> = roots.to_vec();
        wanted.sort();
        wanted.dedup();
        let mut index = self.file_index.lock().unwrap_or_else(|p| p.into_inner());
        if index.dirty || index.roots != wanted {
            index.rebuild(&wanted);
        }
        index.decls.get(symbol).and_then(|sites| sites.first().cloned())
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
