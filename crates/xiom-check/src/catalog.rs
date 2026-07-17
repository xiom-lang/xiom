// XIOM — Module Catalog
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use crate::types::{CheckedType, FnSig};
use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Module export representation
// ============================================================================

#[derive(Debug, Clone)]
pub enum ModuleExport {
    Type { fields: HashMap<String, CheckedType>, is_pub: bool },
    Function { sig: FnSig, is_pub: bool },
    Const { ty: CheckedType, value: Expr, is_pub: bool },
    SubModule(HashMap<String, ModuleExport>),
}

// ============================================================================
// Module Catalog — lazy multi-file resolution
// ============================================================================

/// A cached entry for one resolved external module file.
#[derive(Debug, Clone)]
pub struct CachedModule {
    /// The dotted module name declared by the file (e.g., "benchmark.main").
    pub dotted_name: String,
    /// The parsed AST of the entire file.
    pub program: Program,
    /// Bare type-name → CheckedType for pub types/enums declared in this file.
    pub types: HashMap<String, CheckedType>,
    /// Key → FnSig for pub functions declared in this file.
    pub functions: HashMap<String, FnSig>,
    /// Type-name → field-name → CheckedType for structs declared in this file.
    pub type_fields: HashMap<String, HashMap<String, CheckedType>>,
}

/// Lazy-loading cache of external `.xi` files keyed by dotted module path.
pub struct ModuleCatalog {
    pub source_dirs: Vec<String>,
    cache: HashMap<String, CachedModule>,
    module_index: HashMap<String, String>,
}

impl ModuleCatalog {
    pub fn new(source_dirs: Vec<String>) -> Self {
        Self { source_dirs, cache: HashMap::new(), module_index: HashMap::new() }
    }

    pub fn add_source_dir(&mut self, dir: String) {
        if !self.source_dirs.contains(&dir) {
            self.source_dirs.push(dir);
        }
    }

    /// Pre-build a module_path → file_path index so all lookups are O(1).
    pub fn build_index(&mut self) {
        self.module_index.clear();
        for dir in &self.source_dirs.clone() {
            self.index_dir(Path::new(&dir));
        }
    }

    fn index_dir(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.index_dir(&path);
                } else if path.extension().map_or(false, |e| e == "xi") {
                    if let Some(dotted) = self.read_module_header(&path) {
                        self.module_index.insert(dotted, path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    fn index_lookup(&self, path_segments: &[String]) -> Option<CachedModule> {
        let dotted = path_segments.join(".");
        if let Some(file_path) = self.module_index.get(&dotted) {
            return self.parse_file(file_path, path_segments);
        }
        if let Some(last) = path_segments.last() {
            for (mod_path, file_path) in &self.module_index {
                if mod_path.ends_with(&format!(".{}", last)) || mod_path == last.as_str() {
                    return self.parse_file(file_path, path_segments);
                }
            }
        }
        None
    }

    /// Look up a module by its dotted path segments (e.g., ["benchmark", "main"]).
    /// Returns an owned clone of the CachedModule so the caller can drop the catalog
    /// borrow before mutating other checker fields.
    /// Loads and caches the file on first access.
    pub fn find_owned(&mut self, path_segments: &[String]) -> Option<CachedModule> {
        let key = path_segments.join(".");
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached.clone());
        }

        let cached = self.load_module(path_segments)?;
        self.cache.insert(key.clone(), cached.clone());
        Some(cached)
    }

    /// Returns all cached modules as owned clones (for snapshot iteration).
    pub fn all_cached(&self) -> Vec<CachedModule> {
        self.cache.values().cloned().collect()
    }

    /// Scan source_dirs for a file whose declared module matches path_segments.
    fn load_module(&self, path_segments: &[String]) -> Option<CachedModule> {
        // Strategy a: path-based lookup — <source_dir>/<p0>/<p1>/.../<pn>.xi
        for dir in &self.source_dirs {
            let file_path = format!("{}/{}.xi", dir, path_segments.join("/"));
            if Path::new(&file_path).exists() {
                if let Some(mut cached) = self.parse_file(&file_path, path_segments) {
                    cached.dotted_name = path_segments.join(".");
                    return Some(cached);
                }
            }
            // Also try single-level: <source_dir>/<dotted>.xi
            let file_path2 = format!("{}/{}.xi", dir, path_segments.join("."));
            if Path::new(&file_path2).exists() {
                if let Some(mut cached) = self.parse_file(&file_path2, path_segments) {
                    cached.dotted_name = path_segments.join(".");
                    return Some(cached);
                }
            }
            // Also try the last path segment as a flat filename in this source_dir.
            // This resolves cases where a module like `benchmark.main` is declared
            // in a file named `main.xi` (not `benchmark/main.xi` or `benchmark.main.xi`).
            // The header must match to avoid false positives against sibling dirs.
            if path_segments.len() >= 2 {
                let last = &path_segments[path_segments.len() - 1];
                let file_path3 = format!("{}/{}.xi", dir, last);
                if Path::new(&file_path3).exists() {
                    if let Some(dotted) = self.read_module_header(Path::new(&file_path3)) {
                        let declared: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                        if declared == path_segments {
                            if let Some(mut cached) = self.parse_file(&file_path3, path_segments) {
                                cached.dotted_name = path_segments.join(".");
                                return Some(cached);
                            }
                        }
                    }
                }
            }
        }

        // Index lookup: O(1) via pre-built module_index (fallback if not built: returns None)
        if let Some(cached) = self.index_lookup(path_segments) {
            return Some(cached);
        }

        // Strategy b: scan-based — walk source_dirs for any .xi file whose declared
        // module name (parsed from the file's header) matches path_segments.
        for dir in &self.source_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "xi") {
                        // Quick-parse just the module header to check identity.
                        if let Some(dotted) = self.read_module_header(&path) {
                            let declared: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                            if declared == path_segments {
                                if let Some(mut cached) = self.parse_file(&path.to_string_lossy(), path_segments) {
                                    cached.dotted_name = path_segments.join(".");
                                    return Some(cached);
                                }
                            }
                        }
                    }
                }
                // Recurse into subdirectories
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(cached) = self.load_from_dir(&path, path_segments) {
                                return Some(cached);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Recurse into a subdirectory looking for a matching module header.
    fn load_from_dir(&self, dir: &Path, path_segments: &[String]) -> Option<CachedModule> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "xi") {
                    if let Some(dotted) = self.read_module_header(&path) {
                        let declared: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                        if declared == path_segments {
                            return self.parse_file(&path.to_string_lossy(), path_segments);
                        }
                    }
                } else if path.is_dir() {
                    if let Some(cached) = self.load_from_dir(&path, path_segments) {
                        return Some(cached);
                    }
                }
            }
        }
        None
    }

    /// Quick-parse the `module a.b.c` header of a file without a full parse.
    fn read_module_header(&self, file_path: &Path) -> Option<String> {
        let source = std::fs::read_to_string(file_path).ok()?;
        let mut chars = source.chars().peekable();

        // Skip any initial whitespace / comments, and also skip file-level
        // `use` / `extern` / `const` preamble statements that may appear
        // before the `module` declaration (e.g. async.xi has use before module).
        let mut buf = String::new();
        loop {
            // Skip whitespace
            while chars.peek().map_or(false, |c| c.is_whitespace()) { chars.next(); }
            // Skip line comments
            if chars.peek() == Some(&'/') {
                chars.next();
                if chars.peek() == Some(&'/') {
                    while let Some(&c) = chars.peek() { chars.next(); if c == '\n' { break; } }
                    continue;
                } else if chars.peek() == Some(&'*') {
                    chars.next();
                    loop {
                        match chars.next() {
                            None => return None,
                            Some('*') if chars.peek() == Some(&'/') => { chars.next(); break; }
                            _ => {}
                        }
                    }
                    continue;
                } else {
                    return None;
                }
            }
            // Skip preamble statements before the module keyword
            match chars.peek() {
                Some(&'u') => {
                    // skip `use ...;`
                    while let Some(c) = chars.next() { if c == ';' { break; } }
                }
                Some(&'e') => {
                    // skip `extern ...}`
                    while let Some(c) = chars.next() { if c == '}' { break; } }
                }
                Some(&'c') => {
                    // skip `const ...;`
                    while let Some(c) = chars.next() { if c == ';' { break; } }
                }
                Some(&'p') => {
                    // skip `pub ...;`  (pub use, pub const, etc.)
                    while let Some(c) = chars.next() { if c == ';' { break; } }
                }
                Some(&'m') => break, // found `module`
                None => return None,
                _ => return None, // unexpected token before module
            }
        }

        // Read "module "
        for expected in "module ".chars() {
            match chars.next() {
                Some(c) if c == expected => {}
                _ => return None,
            }
        }

        // Read the module name: ident(.ident)*
        buf.clear();
        loop {
            match chars.peek() {
                Some(&c) if c.is_alphanumeric() || c == '_' => {
                    buf.push(c);
                    chars.next();
                }
                Some(&'.') => {
                    buf.push('.');
                    chars.next();
                }
                _ => break,
            }
        }

        if buf.is_empty() { return None; }
        Some(buf)
    }

    /// Full parse of a .xi file into a CachedModule.
    fn parse_file(&self, file_path: &str, path_segments: &[String]) -> Option<CachedModule> {
        let source = std::fs::read_to_string(file_path).ok()?;
        let tokens = Lexer::new(&source).tokenize();
        let program = Parser::new(tokens).parse_program().ok()?;

        let mut types = HashMap::new();
        let mut functions = HashMap::new();
        let mut type_fields = HashMap::new();

        // Flatten the module tree to collect pub items.
        self.collect_pub_items(&program.items, "", &mut types, &mut functions, &mut type_fields);

        Some(CachedModule {
            dotted_name: path_segments.join("."),
            program,
            types,
            functions,
            type_fields,
        })
    }

    /// Walk nested ModuleDecl tree, collecting pub type/fn/enum info.
    fn collect_pub_items(
        &self,
        items: &[TopDecl],
        prefix: &str,
        types: &mut HashMap<String, CheckedType>,
        functions: &mut HashMap<String, FnSig>,
        type_fields: &mut HashMap<String, HashMap<String, CheckedType>>,
    ) {
        for item in items {
            match item {
                TopDecl::Type(td) if td.is_pub => {
                    let mut fields = HashMap::new();
                    for field in &td.fields {
                        fields.insert(field.name.name.clone(), CheckedType::from_ast_type(&field.ty));
                    }
                    let key = if prefix.is_empty() { td.name.name.clone() } else { format!("{}.{}", prefix, td.name.name) };
                    types.insert(td.name.name.clone(), CheckedType::Named(td.name.name.clone()));
                    type_fields.insert(td.name.name.clone(), fields);
                    type_fields.insert(key.clone(), type_fields.get(&td.name.name).cloned().unwrap_or_default());
                }
                TopDecl::Enum(ed) if ed.is_pub => {
                    let key = if prefix.is_empty() { ed.name.name.clone() } else { format!("{}.{}", prefix, ed.name.name) };
                    types.insert(ed.name.name.clone(), CheckedType::Named(ed.name.name.clone()));
                    types.insert(key, CheckedType::Named(ed.name.name.clone()));
                }
                TopDecl::Fn(fd) if fd.is_pub && !fd.is_method() => {
                    let mut params: Vec<(String, CheckedType)> = Vec::new();
                    for p in &fd.params {
                        params.push((p.name.name.clone(), CheckedType::from_ast_type(&p.ty)));
                    }
                    let return_type = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                    let generics: Vec<String> = fd.generics.iter().map(|g| g.name.name.clone()).collect();
                    let sig = FnSig { params, return_type, generics, uses_implicit_this: false };
                    let bare_key = fd.name.name.clone();
                    let key = if prefix.is_empty() { bare_key.clone() } else { format!("{}.{}", prefix, bare_key) };
                    functions.insert(bare_key, sig.clone());
                    functions.insert(key, sig);
                }
                TopDecl::Module(md) => {
                    let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
                    self.collect_pub_items(&md.items, &new_prefix, types, functions, type_fields);
                }
                _ => {}
            }
        }
    }
}
