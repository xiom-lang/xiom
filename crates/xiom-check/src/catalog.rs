// XIOM -- Module Catalog
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::types::{CheckedType, FnSig};
use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use std::collections::HashMap;
use std::path::Path;

/// Fast 64-bit content hash (FNV-1a) for Level 0 incremental caching.
/// Not cryptographic -- collision resistance is unnecessary for build caches.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h = h.wrapping_mul(0x100000001b3).wrapping_add(b as u64);
    }
    h
}

/// Stage 3 Item A: collect `impl Trait[Args] for Type` method registrations
/// from a catalog file's PRE-expansion AST (see
/// [`CachedModule::impl_registrations`]). Mirrors `Checker::register_impl_inner`
/// key derivation so both paths produce identical keys.
fn collect_impl_registrations(items: &[TopDecl], out: &mut Vec<(String, String, String)>) {
    for item in items {
        match item {
            TopDecl::Impl(impl_decl) => {
                let impl_ty = if impl_decl.type_name.name != "_" {
                    impl_decl.type_name.name.clone()
                } else if let Some(first_arg) = impl_decl.trait_args.first() {
                    CheckedType::from_ast_type(first_arg).name()
                } else {
                    continue;
                };
                let arg_names: Vec<String> = impl_decl.trait_args.iter()
                    .map(|t| CheckedType::from_ast_type(t).name())
                    .collect();
                let key = if arg_names.is_empty() {
                    impl_decl.trait_name.name.clone()
                } else {
                    format!("{}[{}]", impl_decl.trait_name.name, arg_names.join(","))
                };
                for member in &impl_decl.members {
                    if let ImplItem::Fn(fd) = member {
                        out.push((key.clone(), impl_ty.clone(), fd.name.name.clone()));
                    }
                }
            }
            TopDecl::Module(md) => collect_impl_registrations(&md.items, out),
            _ => {}
        }
    }
}

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
// Module Catalog -- lazy multi-file resolution
// ============================================================================

/// A cached entry for one resolved external module file.
#[derive(Debug, Clone)]
pub struct CachedModule {
    /// The dotted module name declared by the file (e.g., "benchmark.main").
    pub dotted_name: String,
    /// The parsed AST of the entire file.
    pub program: Program,
    /// Bare type-name -> CheckedType for pub types/enums declared in this file.
    pub types: HashMap<String, CheckedType>,
    /// Key -> FnSig for pub functions declared in this file.
    pub functions: HashMap<String, FnSig>,
    /// Type-name -> field-name -> CheckedType for structs declared in this file.
    pub type_fields: HashMap<String, HashMap<String, CheckedType>>,
    /// 5c-R: Content hash of the source file (Level 0 incremental cache).
    /// Computed from the raw file bytes. If the hash matches the previous
    /// session, the entire checker/codegen/clang pipeline can be skipped.
    pub source_hash: u64,
    /// Stage 3 Item A: `impl Trait[Args] for Type` registrations collected
    /// BEFORE `expand_impl_blocks` erases the Impl decls at parse time. The
    /// checker replays these into its `impls` map so interface-qualified
    /// static calls in catalog bodies (`Num[T].one()`, `PrecisionLimits[T]
    /// .min_value()`) resolve like they do for user-program impls.
    /// Entries: (trait key "Num[Int]", impl type "Int", method name).
    pub impl_registrations: Vec<(String, String, String)>,
    /// D1: RECOVERABLE parse errors from `Parser::parse_program` (message,
    /// span). The parser returns a partial AST and keeps going, which used to
    /// silently DROP declarations (e.g. `'GBP'` multi-char literals in
    /// xiom.char made `is_currency`/`is_math_symbol` vanish). Catalog loads
    /// keep the module but record the diagnostics so the corpus gate treats
    /// them as hard errors instead of reporting downstream undefined-name
    /// cascades.
    pub parse_errors: Vec<(String, Span)>,
}

/// Lazy-loading cache of external `.xi` files keyed by dotted module path.
pub struct ModuleCatalog {
    pub source_dirs: Vec<String>,
    cache: HashMap<String, CachedModule>,
    module_index: HashMap<String, String>,
    /// R21d follow-up: canonical path per indexed module (identity check so
    /// the same file indexed under absolute+relative spellings is not a
    /// "collision").
    index_canonical: HashMap<String, String>,
    /// All candidate files per dotted module name:
    /// (source-dir index, path-match score, canonical path, display path).
    /// Winner: highest source-dir index (historic last-source-dir priority),
    /// then highest structural path match (`stdlib/xiom/net/dns.xi` beats
    /// `packages/xiom-net/src/dns.xi` for `xiom.net.dns`), then smallest
    /// canonical path (deterministic, unlike filesystem scan order).
    candidates: HashMap<String, std::collections::BTreeSet<(usize, usize, String, String)>>,
    /// Notes for ambiguous module names that were actually LOADED by this
    /// compile (surfacing every collision in the search path would flood the
    /// output with unrelated probe files). Deterministic winner:
    /// lexicographically smallest canonical path.
    pub module_collisions: Vec<String>,
    surfaced_collisions: std::collections::HashSet<String>,
}

impl ModuleCatalog {
    pub fn new(source_dirs: Vec<String>) -> Self {
        Self {
            source_dirs,
            cache: HashMap::new(),
            module_index: HashMap::new(),
            index_canonical: HashMap::new(),
            candidates: HashMap::new(),
            module_collisions: Vec::new(),
            surfaced_collisions: std::collections::HashSet::new(),
        }
    }

    pub fn add_source_dir(&mut self, dir: String) {
        if !self.source_dirs.contains(&dir) {
            self.source_dirs.push(dir);
        }
    }

    /// Pre-build a module_path -> file_path index so all lookups are O(1).
    pub fn build_index(&mut self) {
        self.module_index.clear();
        self.index_canonical.clear();
        self.candidates.clear();
        self.module_collisions.clear();
        self.surfaced_collisions.clear();
        for (dir_index, dir) in self.source_dirs.clone().iter().enumerate() {
            self.index_dir(Path::new(&dir), dir_index);
        }
    }

    /// Directories that are never XIOM source roots: build artifacts, VCS
    /// internals, and package-manager caches. Skipping them keeps the
    /// index build and the scan-based module lookup (Strategy b in
    /// `load_module`) cheap even when a source_dir is a project root or
    /// the working directory (which can contain a full `target/` tree).
    fn should_skip_dir(name: &str) -> bool {
        name.starts_with('.')
            || matches!(name, "target" | "build" | "dist" | "out" | "obj"
                | "node_modules" | ".cargo" | "cmake-build-debug" | "cmake-build-release"
                // R21d follow-up: packaged RELEASE trees contain old stdlib
                // copies (`release/xiom-v*/lib/xiom/*.xi`); indexing them
                // shadowed the live stdlib once module-index winners became
                // path-deterministic and flooded W001 collision notes. Do NOT
                // skip "debug": `stdlib/xiom/debug/` is a real module dir.
                | "release")
    }

    fn index_dir(&mut self, dir: &Path, dir_index: usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Do not index build/VCS/package-manager subtrees.
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if Self::should_skip_dir(name) {
                        continue;
                    }
                    self.index_dir(&path, dir_index);
                } else if path.extension().map_or(false, |e| e == "xi") {
                    if let Some(dotted) = self.read_module_header(&path) {
                        let display = path.to_string_lossy().to_string();
                        let canonical = std::fs::canonicalize(&path)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| display.clone());
                        // Structural match: how many trailing MODULE segments
                        // line up with trailing PATH segments.
                        let mod_segs: Vec<&str> = dotted.split('.').collect();
                        let mut path_segs: Vec<String> = Vec::new();
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            path_segs.push(stem.to_string());
                        }
                        let mut parent = path.parent();
                        while let Some(dir) = parent {
                            if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                                path_segs.push(name.to_string());
                            }
                            parent = dir.parent();
                        }
                        let mut score = 0usize;
                        for (m, p) in mod_segs.iter().rev().zip(path_segs.iter()) {
                            if m.eq_ignore_ascii_case(p) { score += 1 } else { break }
                        }
                        let cands = self.candidates.entry(dotted.clone()).or_default();
                        if !cands.iter().any(|(_, _, c, _)| c == &canonical) {
                            cands.insert((dir_index, score, canonical.clone(), display.clone()));
                        }
                        // R21d follow-up: two DIFFERENT files declare the same
                        // module path. Deterministic winner (see field docs);
                        // the ambiguity is reported when the module is loaded.
                        let best = cands.iter().max_by(|a, b| {
                            a.0.cmp(&b.0)
                                .then_with(|| a.1.cmp(&b.1))
                                .then_with(|| b.2.cmp(&a.2))
                        }).cloned();
                        if let Some((_idx, _score, best_canon, best_display)) = best {
                            if self.index_canonical.get(&dotted) != Some(&best_canon) {
                                self.index_canonical.insert(dotted.clone(), best_canon);
                                self.module_index.insert(dotted, best_display);
                            }
                        }
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
            // Short-leaf / submodule resolution:
            //  - single-segment `use types.run_all` binds the UNIQUE module
            //    whose declared path ends with `.types` (benchmark.types);
            //  - multi-segment `use xiom.slice` resolves a DIRECTORIED
            //    submodule (`xiom.string.slice` -- slice.xi declares
            //    `xiom.string.slice`, so the short path has no exact file)
            //    ONLY within the requested ROOT segment (`xiom.*`).
            // R5 (2026-09-10): the old fallback matched any module ending in
            // `.<last>` across all roots, so the worklist prefix
            // `[xiom, memory]` (from `use xiom.memory.alloc`) fuzzy-loaded
            // `benchmark.memory`, whose `use benchmark.main` dragged the whole
            // benchmark graph into every stdlib compile -- benchmark.types.
            // Address then claimed the bare `Address` name ahead of
            // xiom.net.address.Address. Root-scoping rejects that while
            // keeping xiom-rooted short submodule paths working. Matches must
            // also be unique (deterministic).
            let root = &path_segments[0];
            let want = format!(".{}", last);
            let mut hits: Vec<&String> = self.module_index.iter()
                .filter(|(mod_path, _)| {
                    if path_segments.len() == 1 {
                        mod_path.ends_with(&want) || mod_path.as_str() == last.as_str()
                    } else {
                        let same_root = mod_path.split('.').next().map_or(false, |r| r == root.as_str());
                        same_root && (mod_path.ends_with(&want) || mod_path.as_str() == last.as_str())
                    }
                })
                .map(|(_, f)| f)
                .collect();
            hits.sort();
            if hits.len() == 1 {
                return self.parse_file(hits[0], path_segments);
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
        // R49-1: cache under the module's DECLARED identity (parse_file).
        // Inserting the requested path alias instead would let one file sit
        // in the cache under two names and be injected twice.
        let cache_key = cached.dotted_name.clone();
        self.surface_collision(&cache_key);
        self.cache.insert(cache_key, cached.clone());
        Some(cached)
    }

    /// R21d follow-up: when a module with an ambiguous declaration is actually
    /// LOADED, surface a one-line note (winner + all candidate files). Only
    /// loaded modules are reported: indexing a broad search path can contain
    /// unrelated probe files that share a module name but never participate
    /// in this compile.
    fn surface_collision(&mut self, dotted: &str) {
        if self.surfaced_collisions.contains(dotted) {
            return;
        }
        let Some(cands) = self.candidates.get(dotted) else { return };
        // Distinct canonical files only: the same file indexed under an
        // absolute and a relative spelling is not a collision.
        let mut canonicals: Vec<&String> = cands.iter().map(|(_, _, c, _)| c).collect();
        canonicals.sort();
        canonicals.dedup();
        if canonicals.len() < 2 {
            return;
        }
        self.surfaced_collisions.insert(dotted.to_string());
        let winner = self.module_index.get(dotted).cloned().unwrap_or_default();
        let mut displays: Vec<String> = cands.iter().map(|(_, _, _, d)| d.clone()).collect();
        displays.sort();
        displays.dedup();
        self.module_collisions.push(format!(
            "module '{dotted}' declared by {} files [{}]; using '{winner}'",
            displays.len(),
            displays.join("', '")
        ));
    }

    /// Parse a module WITHOUT caching it (lazy resolution peek). Used by the
    /// qualified-call walk to descend into submodule segments
    /// (`os.platform.platform_name()` after `use xiom.os;`) without adding
    /// the submodule to the INJECTION set -- eager caching perturbed
    /// bare-alias keep-first resolution for unrelated programs (crypto
    /// sha256 broke when os/* submodules entered the graph).
    pub fn peek_owned(&mut self, path_segments: &[String]) -> Option<CachedModule> {
        let key = path_segments.join(".");
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached.clone());
        }
        let loaded = self.load_module(path_segments);
        if loaded.is_some() {
            self.surface_collision(&key);
        }
        loaded
    }

    /// Names of DIRECT submodules of a dotted module path (e.g. "xiom.os" ->
    /// ["env", "filesystem", "path", "platform", ...]). Derived from the
    /// module index so directory modules (os/ has os.xi + platform.xi) expose
    /// their submodules to `use`-importers -- without this, `use xiom.os;`
    /// followed by `os.platform.platform_name()` cannot resolve the
    /// "platform" segment (regression from the 4c439e6a batch).
    pub fn submodule_names(&self, dotted: &str) -> Vec<String> {
        let prefix = format!("{dotted}.");
        let mut names: Vec<String> = Vec::new();
        for key in self.module_index.keys() {
            if let Some(rest) = key.strip_prefix(&prefix) {
                if let Some(seg) = rest.split('.').next() {
                    if !names.iter().any(|n| n == seg) {
                        names.push(seg.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    /// R49-1: is `dotted` a DECLARED module name in the index? (Used to skip
    /// the path-alias rewrite for normal imports.)
    pub fn is_declared_module(&self, dotted: &str) -> bool {
        self.module_index.contains_key(dotted)
    }

    /// Every INDEXED dotted module name, sorted (deterministic order).
    /// Stage 3 Item A: drives the full-corpus body check.
    pub fn module_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.module_index.keys().cloned().collect();
        names.sort();
        names
    }

    /// Returns all cached modules as owned clones (for snapshot iteration).
    pub fn all_cached(&self) -> Vec<CachedModule> {        // BUG 23 #5 fix: deterministic ORDER -- the cache is a HashMap, so
        // iteration order is randomized per process. collect_external_decls
        // walks this list to inject decls into the program; with random order
        // the codegen's first-registered-wins symbol assignment for
        // same-leaf-name fns (e.g. `_tridiagonal`) flipped per run, causing
        // ~50% flaky "use of undefined value '@...'" compile failures.
        let mut v: Vec<CachedModule> = self.cache.values().cloned().collect();
        v.sort_by(|a, b| a.dotted_name.cmp(&b.dotted_name));
        v
    }

    /// Scan source_dirs for a file whose declared module matches path_segments.
    fn load_module(&self, path_segments: &[String]) -> Option<CachedModule> {
        // Strategy a: path-based lookup -- <source_dir>/<p0>/<p1>/.../<pn>.xi
        for dir in &self.source_dirs {
            let file_path = format!("{}/{}.xi", dir, path_segments.join("/"));
            if Path::new(&file_path).exists() {
                if let Some(cached) = self.parse_file(&file_path, path_segments) {
                    // R49-1: keep the DECLARED identity (parse_file); a moved
                    // module must not be registered under its path alias.
                    return Some(cached);
                }
            }
            // Also try single-level: <source_dir>/<dotted>.xi
            let file_path2 = format!("{}/{}.xi", dir, path_segments.join("."));
            if Path::new(&file_path2).exists() {
                if let Some(cached) = self.parse_file(&file_path2, path_segments) {
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
                            if let Some(cached) = self.parse_file(&file_path3, path_segments) {
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

        // Strategy b: scan-based -- walk source_dirs for any .xi file whose declared
        // module name (parsed from the file's header) matches path_segments.
        // ONLY used when the index was never built (e.g. catalog unit tests):
        // when `build_index` has run, the module_index already covers every
        // file this scan could find (both index the same dirs with the same
        // header check), so the scan would be pure waste -- and it is
        // PATHOLOGICAL on a source_dir that is a project root or the working
        // directory (walking the whole tree per failed leaf lookup, e.g.
        // `use xiom.core.to_int` where to_int is a fn/const, not a module --
        // observed multi-minute hangs). The index is authoritative; skip the
        // scan entirely.
        if self.module_index.is_empty() {
            for dir in &self.source_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "xi") {
                        // Quick-parse just the module header to check identity.
                        if let Some(dotted) = self.read_module_header(&path) {
                            let declared: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                            if declared == path_segments {
                                if let Some(cached) = self.parse_file(&path.to_string_lossy(), path_segments) {
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
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if Self::should_skip_dir(name) {
                                continue;
                            }
                            if let Some(cached) = self.load_from_dir(&path, path_segments) {
                                return Some(cached);
                            }
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
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if Self::should_skip_dir(name) {
                        continue;
                    }
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
        let source_hash = hash_bytes(source.as_bytes());
        let tokens = Lexer::new(&source).tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().ok()?;
        // D1: keep the recoverable diagnostics; the partial AST silently lost
        // every declaration the parser skipped.
        let parse_errors: Vec<(String, Span)> = parser.errors().iter()
            .map(|e| (e.message.clone(), e.span))
            .collect();
        // Stage 3 Item A: capture trait-impl registrations BEFORE the
        // expansion below erases the Impl decls (the checker needs the
        // trait -> implementing-type mapping for `Trait[T].method()`).
        let mut impl_registrations = Vec::new();
        collect_impl_registrations(&program.items, &mut impl_registrations);
        // 3c (2026-08-10): expand impl blocks at PARSE time so catalog-loaded
        // modules (e.g. stdlib folder modules with `impl Num[Float64] { }`)
        // expose the expanded `Type.method` freestanding fns to both the
        // checker's registration and the driver's external-decl injection.
        let program = xiom_lowering::expand_impl_blocks(&program);

        let mut types = HashMap::new();
        let mut functions = HashMap::new();
        let mut type_fields = HashMap::new();

        // Flatten the module tree to collect pub items.
        self.collect_pub_items(&program.items, "", &mut types, &mut functions, &mut type_fields);

        // R49-1 (stdlib relay p_module_path_alias): the module's IDENTITY is
        // the name it DECLARES, not the file path it was looked up by.
        // Moved modules (crypto/legacy/md5.xi declares `xiom.crypto.md5`)
        // were registered under the path alias, giving one file two
        // identities and corrupting the parent module's submodule set
        // (`use xiom.crypto.legacy.md5;` -> 33 T001s in rng_crypto).
        let declared_name = self
            .read_module_header(std::path::Path::new(file_path))
            .unwrap_or_else(|| path_segments.join("."));

        Some(CachedModule {
            dotted_name: declared_name,
            program,
            types,
            functions,
            type_fields,
            source_hash,
            impl_registrations,
            parse_errors,
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
                    types.insert(td.name.name.clone(), CheckedType::named(td.name.name.clone()));
                    type_fields.insert(td.name.name.clone(), fields);
                    type_fields.insert(key.clone(), type_fields.get(&td.name.name).cloned().unwrap_or_default());
                }
                TopDecl::Enum(ed) if ed.is_pub => {
                    let key = if prefix.is_empty() { ed.name.name.clone() } else { format!("{}.{}", prefix, ed.name.name) };
                    types.insert(ed.name.name.clone(), CheckedType::named(ed.name.name.clone()));
                    types.insert(key, CheckedType::named(ed.name.name.clone()));
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
