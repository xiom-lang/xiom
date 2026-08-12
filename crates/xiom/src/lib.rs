// XIOM Compiler Library
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
// -----------------------------------------------------------------------

pub mod ai;
pub mod graph_viz;
pub mod implicit_main;
pub mod jit;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_check::{Checker, BorrowChecker};
use xiom_codegen::IrEmitter;
use xiom_verify::SMTGenerator;

#[derive(PartialEq, Clone, Copy)]
pub enum Target {
    Native,
    /// wasm32-unknown-unknown (bare WASM, no WASI)
    Wasm,
    /// wasm32-wasi (WASI preview2, with I/O and filesystem)
    Wasi,
    Arm,
    RisCv,
}

pub struct CompileConfig {
    pub target: Target,
    pub emit_ir: bool,
    pub do_run: bool,
    pub check_only: bool,
    pub release: bool,
    pub check_contracts: bool,
    pub diagnostics_json: bool,
    pub strict_mode: bool,
    pub debug_symbols: bool,
    pub shared_lib: bool,
    pub static_lib: bool,
    pub hot_reload: bool,
    /// 7D.3: verify contracts before hot-swapping function pointers
    pub hot_reload_contracts: bool,
    /// 5e.5f: incremental compilation Ã¢â‚¬â€ cache IR, skip unchanged files
    pub incremental: bool,
    /// 5e.5f: force recompilation Ã¢â‚¬â€ ignore all caches
    pub force: bool,
    /// 7C: parallel compilation Ã¢â‚¬â€ use rayon thread pool for lex+parse
    pub parallel: bool,
    /// 7C: maximum number of parallel jobs (0 = num_cpus)
    pub jobs: usize,
    /// 7E.1: sanitizer type (none, address, undefined, leak, thread)
    pub sanitize: Option<String>,
    /// 7E.2: enable stack protector (canaries) via clang -fstack-protector
    pub stack_protector: bool,
    /// 7E.4: force runtime contract checks even in release builds
    pub runtime_contracts: bool,
    pub overflow_checks: bool,
    /// S2: Promote non-exhaustive match warnings to hard errors
    pub strict_exhaustive: bool,
    pub max_recursion_depth: u32,
    pub dump_contracts: bool,
    pub verify: bool,
    pub verify_output: Option<String>,
    pub output_file: Option<String>,
    pub link_libs: Vec<String>,
    pub link_paths: Vec<String>,
    pub c_sources: Vec<String>,
    /// M12: Scripting mode Ã¢â‚¬â€ apply implicit main wrapping if no fn main found
    pub script_mode: bool,
    /// v0.54: Binary cache Ã¢â‚¬â€ hash source with SHA-256, cache compiled binary
    /// for instant re-execution (~500ms Ã¢â€ â€™ ~5ms). Applies to --run mode.
    pub cache: bool,
    /// v0.56: ThinLTO link-time optimization (--lto flag)
    pub lto: bool,
    /// I2: Parallel codegen Ã¢â‚¬â€ rayon-based per-function IR emission (--parallel-codegen)
    pub parallel_codegen: bool,
    /// D2.1 (Phase 7): `--enable-unsafe-direct` â€” allow `#[unsafe_direct]`
    /// (trusted escape hatch) in user code (stdlib/selfhost always allowed).
    pub enable_unsafe_direct: bool,
}

impl Default for CompileConfig {
    fn default() -> Self {
        CompileConfig {
            target: Target::Native,
            emit_ir: false,
            do_run: false,
            check_only: false,
            release: false,
            check_contracts: true,
            diagnostics_json: false,
            strict_mode: false,
            debug_symbols: false,
            shared_lib: false,
            static_lib: false,
            hot_reload: false,
            hot_reload_contracts: false,
            incremental: false,
            force: false,
            parallel: false,
            jobs: 0,
            sanitize: None,
            stack_protector: false,
            runtime_contracts: false,
            overflow_checks: false,
            strict_exhaustive: false,
            max_recursion_depth: 500,
            dump_contracts: false,
            verify: false,
            verify_output: None,
            output_file: None,
            link_libs: Vec::new(),
            link_paths: Vec::new(),
            c_sources: Vec::new(),
            script_mode: false,
            cache: false,
            lto: false,
            parallel_codegen: false,
            enable_unsafe_direct: false,
        }
    }
}

// ============================================================================
// Phase 5d: Safe library API for MCP/tooling (returns Result, never exit)
// ============================================================================

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub kind: String,       // "lex_error", "parse_error", "type_error", "codegen_error"
    pub code: String,       // "L001", "P001", "T001", "C001"
    pub message: String,
    pub line: u32,
    pub col: u32,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    pub file_count: usize,
}

/// Phase 7A: Resolve all project sources using the dependency graph.
///
/// If a `xiom.toml` or `package.xi` is found, discovers all `.xi` files
/// under the configured source roots and returns them in topological
/// (dependency-first) order. Falls back to the original source list if
/// no project manifest is found or the graph cannot be built.
///
/// **Important:** The graph is only used for source directory discovery.
/// The returned source list is the original list Ã¢â‚¬â€ specific file compilation
/// should not expand to the entire project. The graph source roots are
/// returned separately so the Checker catalog can resolve `use` imports.
///
/// Returns `(expanded_sources, extra_source_dirs)`.
pub fn expand_sources_with_graph(source_paths: &[String]) -> (Vec<String>, Vec<String>) {
    if source_paths.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let first = Path::new(&source_paths[0]);
    if !first.exists() {
        return (source_paths.to_vec(), Vec::new());
    }

    // Try building the project graph for catalog/checker source dirs only.
    // We do NOT replace the source file list Ã¢â‚¬â€ explicit compilation of
    // specific files must work without pulling in the entire project.
    match xiom_graph::build_project_graph(first) {
        Ok(graph) if !graph.is_empty() => {
            // Collect extra source directories for the Checker catalog.
            // These enable the catalog to resolve cross-module `use` imports
            // even when compiling a single file.
            let extra_dirs: Vec<String> = graph
                .source_roots
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            // If the user compiled a directory (no specific .xi files),
            // use the graph's compilation order. Otherwise, keep the
            // explicit file list.
            let all_dirs: bool = source_paths.iter().all(|p| {
                std::fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false)
            });

            if all_dirs || source_paths.is_empty() {
                // Directory compilation: use topo-sorted order
                match graph.compilation_order() {
                    Ok(files) => {
                        let paths: Vec<String> = files
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        return (paths, extra_dirs);
                    }
                    Err(e) => {
                        eprintln!("xiom: warning: dependency graph: {}", e);
                    }
                }
            }

            // Explicit files: keep original list, add graph source dirs
            (source_paths.to_vec(), extra_dirs)
        }
        Ok(_) => {
            // Empty graph Ã¢â‚¬â€ fall back
            (source_paths.to_vec(), Vec::new())
        }
        Err(e) => {
            // No project found or parse error Ã¢â‚¬â€ fall back silently
            if !matches!(e, xiom_graph::GraphError::NoProjectFound(_)) {
                eprintln!("xiom: warning: {}", e);
            }
            (source_paths.to_vec(), Vec::new())
        }
    }
}

/// Production-grade library API: compile XIOM sources and return structured
/// diagnostics. Never calls `process::exit()`. Safe for use from MCP server,
/// LSP, debugger, and any long-running process.
pub fn compile_with_diagnostics(config: &CompileConfig, source_paths: &[String]) -> CompileResult {
    let mut result = CompileResult {
        success: false,
        diagnostics: Vec::new(),
        ir: None,
        contracts: None,
        warnings: None,
        file_count: 0,
    };
    let mut warnings = Vec::new();

    // Phase 7A: Expand source list using project dependency graph
    let (resolved_sources, graph_source_dirs) = expand_sources_with_graph(source_paths);
    let effective_sources: &[String] = if !resolved_sources.is_empty() {
        &resolved_sources
    } else {
        source_paths
    };

    // 5e.5f / 7B: Incremental compilation Ã¢â‚¬â€ check graph-aware cache
    if config.incremental && !config.force && effective_sources.len() >= 1 {
        let sp = &effective_sources[0];
        if let Some(cached_ir) = incremental_check(sp) {
            result.success = true;
            result.ir = Some(cached_ir);
            result.file_count = 1;
            return result;
        }
    }

    // Stage 1: Lex & Parse (Phase 7C: parallel across files via rayon)
    let file_count = effective_sources.len();
    let use_parallel = config.parallel && file_count > 1;
    if use_parallel && config.jobs > 0 {
        // Respect explicit job count
        if std::env::var("RAYON_NUM_THREADS").is_err() {
            // SAFETY: set_var is not thread-safe, but this runs before rayon's thread pool
            // is initialized (par_iter is called below). The guard ensures we only set the
            // variable once, preventing races with other threads that might read it.
            unsafe { std::env::set_var("RAYON_NUM_THREADS", config.jobs.to_string()); }
        }
    }
    let parse_results: Vec<(usize, Option<Program>, Vec<Diagnostic>)> = if use_parallel {
        use rayon::prelude::*;
        effective_sources
            .par_iter()
            .enumerate()
            .map(|(idx, source_path)| {
                let mut diags = Vec::new();
                let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "package.xi" { return (idx, None, diags); }

                let source = match fs::read_to_string(source_path) {
                    Ok(s) => s,
                    Err(e) => {
                        diags.push(Diagnostic {
                            kind: "io_error".into(), code: "I001".into(),
                            message: format!("cannot read '{}': {}", source_path, e),
                            line: 0, col: 0, file: source_path.clone(),
                            suggestion: None, help: None, note: None,
                        });
                        return (idx, None, diags);
                    }
                };

                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize();

                let mut lex_errors = false;
                for tok in &tokens {
                    if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
                        diags.push(Diagnostic {
                            kind: "lex_error".into(), code: "L001".into(),
                            message: msg.clone(),
                            line: tok.span.line, col: tok.span.col, file: source_path.clone(),
                            suggestion: None, help: None, note: None,
                        });
                        lex_errors = true;
                    }
                }
                if lex_errors { return (idx, None, diags); }

                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(p) => {
                        for e in parser.errors() {
                            let (help, note) = diagnostic_for(&e.message);
                            diags.push(Diagnostic {
                                kind: "parse_error".into(), code: "P001".into(),
                                message: e.message.clone(),
                                line: e.span.line, col: e.span.col, file: source_path.clone(),
                                suggestion: Some(suggest_fix(&e.message)),
                                help, note,
                            });
                        }
                        (idx, Some(p), diags)
                    }
                    Err(e) => {
                        let (help, note) = diagnostic_for(&e.message);
                        let suggestion = suggest_fix(&e.message);
                        diags.push(Diagnostic {
                            kind: "parse_error".into(), code: "P001".into(),
                            message: e.message,
                            line: e.span.line, col: e.span.col, file: source_path.clone(),
                            suggestion: Some(suggestion),
                            help, note,
                        });
                        (idx, None, diags)
                    }
                }
            })
            .collect()
    } else {
        // Single-file: sequential path (no rayon overhead)
        effective_sources
            .iter()
            .enumerate()
            .map(|(idx, source_path)| {
                let mut diags = Vec::new();
                let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "package.xi" { return (idx, None, diags); }

                let source = match fs::read_to_string(source_path) {
                    Ok(s) => s,
                    Err(e) => {
                        diags.push(Diagnostic {
                            kind: "io_error".into(), code: "I001".into(),
                            message: format!("cannot read '{}': {}", source_path, e),
                            line: 0, col: 0, file: source_path.clone(),
                            suggestion: None, help: None, note: None,
                        });
                        return (idx, None, diags);
                    }
                };

                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize();

                let mut lex_errors = false;
                for tok in &tokens {
                    if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
                        diags.push(Diagnostic {
                            kind: "lex_error".into(), code: "L001".into(),
                            message: msg.clone(),
                            line: tok.span.line, col: tok.span.col, file: source_path.clone(),
                            suggestion: None, help: None, note: None,
                        });
                        lex_errors = true;
                    }
                }
                if lex_errors { return (idx, None, diags); }

                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(p) => {
                        for e in parser.errors() {
                            let (help, note) = diagnostic_for(&e.message);
                            diags.push(Diagnostic {
                                kind: "parse_error".into(), code: "P001".into(),
                                message: e.message.clone(),
                                line: e.span.line, col: e.span.col, file: source_path.clone(),
                                suggestion: Some(suggest_fix(&e.message)),
                                help, note,
                            });
                        }
                        (idx, Some(p), diags)
                    }
                    Err(e) => {
                        let (help, note) = diagnostic_for(&e.message);
                        let suggestion = suggest_fix(&e.message);
                        diags.push(Diagnostic {
                            kind: "parse_error".into(), code: "P001".into(),
                            message: e.message,
                            line: e.span.line, col: e.span.col, file: source_path.clone(),
                            suggestion: Some(suggestion),
                            help, note,
                        });
                        (idx, None, diags)
                    }
                }
            })
            .collect()
    };

    // Collect diagnostics and programs in order
    let mut all_programs: Vec<Program> = Vec::new();
    for (_, _prog, diags) in &parse_results {
        for d in diags {
            result.diagnostics.push(d.clone());
        }
    }
    result.file_count = file_count;

    // Check for parse/lex errors before proceeding
    if result.diagnostics.iter().any(|d| d.kind == "parse_error" || d.kind == "lex_error") {
        return result;
    }

    // Collect programs in original order
    for (_, prog, _) in parse_results {
        if let Some(p) = prog {
            all_programs.push(p.clone());
        }
    }

    if all_programs.is_empty() {
        return result;
    }

    // Merge programs
    let program = merge_programs(all_programs);

    // Stage 3: Type Check
    let mut checker = Checker::new();
    checker.set_strict_exhaustive(config.strict_exhaustive);
    // D1: register `impl Trait[Args]` blocks from the UNEXPANDED program so
    // `Trait[Args].method()` static calls can dispatch. Must happen before
    // expand_impl_blocks erases the impl declarations.
    checker.register_impls_from_program(&program);
    // M20: Expand impl blocks into freestanding functions before type checking
    let program = program.expand_impl_blocks();
    if let Some(primary) = effective_sources.first() {
        let file_path = Path::new(primary);
        // Add the file's parent directory (e.g. examples/)
        if let Some(parent) = file_path.parent() {
            checker.add_source_dir(parent.to_string_lossy().to_string());
            // 5e.3 G-31: also walk up one level for `project/examples/demo.xi`
            // using `project/vulkan.xi`. Guard: only add the grandparent if it
            // contains at least one .xi file (avoid scanning system dirs like
            // C:\Users\...\AppData\Local for temp files).
            if let Some(grandparent) = parent.parent() {
                if std::fs::read_dir(grandparent).map_or(false, |entries| {
                    entries.flatten().any(|e| e.path().extension().map_or(false, |ext| ext == "xi"))
                }) {
                    checker.add_source_dir(grandparent.to_string_lossy().to_string());
                }
            }
        }
        // Walk-up: if a project marker exists, add src/ subdirectory
        if let Some(root) = find_project_root(file_path) {
            let src_dir = root.join("src");
            if src_dir.is_dir() {
                checker.add_source_dir(src_dir.to_string_lossy().to_string());
            }
        }
    }
    // Phase 7A: Add source root directories from the dependency graph
    for dir in &graph_source_dirs {
        checker.add_source_dir(dir.clone());
    }
    for stdlib_dir in find_stdlib_dirs() {
        checker.add_source_dir(stdlib_dir);
    }
    checker.build_catalog_index();

    if let Err(errors) = checker.check_program(&program) {
        for err in &errors {
            let suggestion = suggest_fix(&err.message);
            let (help, note) = diagnostic_for(&err.message);
            result.diagnostics.push(Diagnostic {
                kind: "type_error".into(), code: "T001".into(),
                message: err.message.clone(),
                line: err.span.line, col: err.span.col, file: "<unknown>".into(),
                suggestion: Some(suggestion),
                help, note,
            });
        }
        // Collect warnings from stderr-like messages
        for err in &errors {
            if err.message.contains("warning") {
                warnings.push(err.message.clone());
            }
        }
        return result;
    }

    // Stage 5: Codegen (IR emission)
    let mut emitter = IrEmitter::new();
    match config.target {
        Target::Wasm => emitter.set_target_triple("wasm32-unknown-unknown"),
        Target::Wasi => emitter.set_target_triple("wasm32-wasi"),
        Target::Arm => emitter.set_target_triple("aarch64-unknown-linux-gnu"),
        Target::RisCv => emitter.set_target_triple("riscv64-unknown-linux-gnu"),
        Target::Native => {}
    }
    emitter.set_check_contracts(config.check_contracts || config.runtime_contracts);
    emitter.set_overflow_checks(config.overflow_checks);
    emitter.set_parallel_codegen(config.parallel_codegen);
    emitter.set_max_recursion_depth(config.max_recursion_depth);
    emitter.set_strict_mode(config.strict_mode);
    emitter.set_hot_reload(config.hot_reload);
    emitter.set_debug_symbols(config.debug_symbols);
    emitter.set_enable_unsafe_direct(config.enable_unsafe_direct);
    if !effective_sources.is_empty() {
        emitter.set_source_file(effective_sources[0].clone());
    }

    match emitter.compile_program(&program) {
        Ok(ir) => {
            result.ir = Some(ir.clone());
            result.success = true;

            // 5e.5f / 7B: Save compiled IR to incremental cache
            if config.incremental && !config.force && effective_sources.len() >= 1 {
                incremental_save(&effective_sources[0], &ir);
            }

            // Dump contracts if requested
            if config.dump_contracts {
                result.contracts = Some(dump_contracts_json(&program));
            }
        }
        Err(e) => {
            result.diagnostics.push(Diagnostic {
                kind: "codegen_error".into(), code: "C001".into(),
                message: e,
                line: 0, col: 0, file: "<unknown>".into(),
                suggestion: None, help: None, note: None,
            });
        }
    }

    if !warnings.is_empty() {
        result.warnings = Some(warnings);
    }

    result
}

pub fn compile(config: &CompileConfig, source_paths: &[String]) -> Result<(), Vec<String>> {
    // v0.54: Binary cache Ã¢â‚¬â€ check for cached binary before compilation.
    // When --run --cache is used, skip the full compile pipeline if the
    // source hasn't changed since the last compilation.
    if config.cache && config.do_run && source_paths.len() == 1 {
        let source_path = &source_paths[0];
        if let Ok(source) = fs::read_to_string(source_path) {
            let lookup_source = if config.script_mode {
                crate::implicit_main::wrap_implicit_main(&source)
            } else {
                source
            };
            if let Some(cached) = crate::jit::script_cache_get(&lookup_source) {
                let run_status = Command::new(&cached).status();
                match run_status {
                    Ok(s) => {
                        eprintln!("  cached run exit code: {}", s.code().unwrap_or(-1));
                        return Ok(());
                    }
                    Err(_) => { /* stale cache entry or binary removed Ã¢â‚¬â€ proceed */ }
                }
            }
        }
    }

    // M12: Auto-discover stdlib from binary path so the C runtime is always found.
    // This ensures xiom run, playground, MCP, and direct CLI all work without XIOM_STDLIB env var.
    if std::env::var("XIOM_STDLIB").is_err() {
        if let Ok(exe) = std::env::current_exe() {
            let mut search = exe.parent();
            for _ in 0..8 {
                if let Some(dir) = search {
                    let candidate = dir.join("stdlib");
                    if candidate.is_dir() {
                        // SAFETY: set_var is called during stdlib discovery, before
                        // any compilation threads are spawned. No concurrent access.
                        unsafe { std::env::set_var("XIOM_STDLIB", candidate.to_string_lossy().to_string()); }
                        break;
                    }
                    let rt = dir.join("runtime");
                    if rt.is_dir() && rt.join("xiom_runtime.c").exists() {
                        // SAFETY: set_var is called during single-threaded initialization
                        // before rayon's thread pool or any parallel work starts.
                        unsafe { std::env::set_var("XIOM_STDLIB", dir.to_string_lossy().to_string()); }
                        break;
                    }
                    search = dir.parent();
                } else { break; }
            }
        }
    }
    // Phase 7A: Expand source list using project dependency graph
    let (resolved_sources, graph_source_dirs) = expand_sources_with_graph(source_paths);
    let effective_sources: &[String] = if !resolved_sources.is_empty() {
        &resolved_sources
    } else {
        source_paths
    };

    // Stage 1: Lex & Parse
    let mut all_programs: Vec<xiom_ast::Program> = Vec::new();

    for source_path in effective_sources {
        let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "package.xi" { continue; }

        let source = fs::read_to_string(source_path)
            .map_err(|e| { eprintln!("error: cannot read '{source_path}': {e}"); vec![format!("cannot read '{source_path}': {e}")] })?;

        // M12: Scripting mode Ã¢â‚¬â€ apply implicit main wrapping.
        // In script_mode (xiom run), always wrap.
        // In --check mode, wrap only if source has no fn main (script-like).
        let source = if config.script_mode {
            crate::implicit_main::wrap_implicit_main(&source)
        } else if config.check_only && !source.contains("fn main") {
            crate::implicit_main::wrap_implicit_main(&source)
        } else {
            source
        };

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();

        let lex_errors: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.kind, xiom_lexer::TokenKind::Error(_)))
            .collect();
        if !lex_errors.is_empty() {
            for tok in &lex_errors {
                if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
                    render_error("L001", &tok.span, msg, Some(&source), None, None);
                }
            }
            return Err(vec!["compilation failed".to_string()]);
        }

        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(p) => {
                // Surface RECOVERED parse errors (panic-mode recovery returns
                // Ok with a partial AST). Without this, a malformed declaration
                // is silently DROPPED Ã¢â‚¬â€ e.g. a trailing-comma fn disappeared
                // with no diagnostic and callers got 'undefined variable'.
                if !parser.errors().is_empty() {
                    for e in parser.errors() {
                        let (help, note) = diagnostic_for(&e.message);
                        render_error("P001", &e.span, &e.message, Some(&source), help.as_deref(), note.as_deref());
                    }
                    return Err(vec!["compilation failed".to_string()]);
                }
                all_programs.push(p);
            }
            Err(e) => {
                let (help, note) = diagnostic_for(&e.message);
                render_error("P001", &e.span, &e.message, Some(&source), help.as_deref(), note.as_deref());
                return Err(vec!["compilation failed".to_string()]);
            }
        }
    }

    // Merge all parsed programs into one
    let mut program = merge_programs(all_programs);
    // D1: register `impl Trait[Args]` blocks before expansion erases them.
    let mut checker = Checker::new();
    checker.register_impls_from_program(&program);
    // M20: Expand impl blocks
    program = program.expand_impl_blocks();

    // Stage 3: Type Check
    checker.set_strict_exhaustive(config.strict_exhaustive);
    if let Some(primary) = effective_sources.first() {
        let file_path = Path::new(primary);
        if let Some(parent) = file_path.parent() {
            checker.add_source_dir(parent.to_string_lossy().to_string());
            if let Some(grandparent) = parent.parent() {
                if std::fs::read_dir(grandparent).map_or(false, |entries| {
                    entries.flatten().any(|e| e.path().extension().map_or(false, |ext| ext == "xi"))
                }) { checker.add_source_dir(grandparent.to_string_lossy().to_string()); }
            }
        }
        if let Some(root) = find_project_root(file_path) {
            let src_dir = root.join("src");
            if src_dir.is_dir() {
                checker.add_source_dir(src_dir.to_string_lossy().to_string());
            }
        }
    }
    // Phase 7A: Add source root directories from the dependency graph
    for dir in &graph_source_dirs {
        checker.add_source_dir(dir.clone());
    }
    let examples_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().expect("CARGO_MANIFEST_DIR has parent")
        .parent().expect("project root has parent")
        .join("examples");
    if examples_root.is_dir() {
        checker.add_source_dir(examples_root.to_string_lossy().to_string());
    }
    for stdlib_dir in find_stdlib_dirs() {
        checker.add_source_dir(stdlib_dir);
    }
    checker.build_catalog_index();
    let is_multi_file = effective_sources.len() > 1 || checker.source_dirs.len() > 0;
    if let Err(errors) = checker.check_program(&program) {
        if config.diagnostics_json {
            let parts: Vec<String> = errors.iter().map(|err| {
                let suggestion = suggest_fix(&err.message);
                format!(
                    r#"{{"kind":"type_error","code":"T001","message":"{}","location":{{"file":"{}","line":{},"col":{}}},"suggestion":"{}"}}"#,
                    escape_json(&err.message), escape_json("<unknown>"), err.span.line, err.span.col,
                    escape_json(&suggestion)
                )
            }).collect();
            println!("[{}]", parts.join(","));
        } else {
            for err in &errors {
                let (help_msg, note_msg) = diagnostic_for(&err.message);
                eprintln!("error[T001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
                if let Some(note) = note_msg {
                    eprintln!("  = note: {note}");
                }
                if let Some(help) = help_msg {
                    eprintln!("  = help: {help}");
                }
            }
        }
        if !is_multi_file {
            return Err(vec!["compilation failed".to_string()]);
        }
        eprintln!("note: {} type errors Ã¢â‚¬â€ aborting codegen", errors.len());
        return Err(vec!["compilation failed".to_string()]);
    }

    if config.dump_contracts {
        let json = dump_contracts_json(&program);
        println!("{json}");
        return Ok(());
    }

    if config.verify {
        let mut generator = SMTGenerator::new();
        let smt = generator.generate(&program);
        if let Some(path) = &config.verify_output {
            fs::write(path, &smt).expect("failed to write SMT output");
            eprintln!("SMT-LIB written to {}", path);
        } else {
            println!("{}", smt);
        }
        if let Ok(_) = std::process::Command::new("z3").arg("-version").output() {
            eprintln!("Z3 found Ã¢â‚¬â€ use 'z3 file.smt2' to verify");
        }
        return Ok(());
    }

    let primary_source = effective_sources.first().map(|s| s.as_str()).unwrap_or("<unknown>");

    if config.check_only {
        if config.diagnostics_json {
            println!(r#"{{"status":"check_passed","type_errors":0,"borrow_warnings":0,"time_ms":0}}"#);
        } else {
            eprintln!("  Type check PASSED (no errors)");
        }
        return Ok(());
    }

    // Stage 4: Borrow Check
    let mut borrow_checker = BorrowChecker::new();
    if let Err(errors) = borrow_checker.check_program(&program) {
        if config.strict_mode {
            // P2-2: In strict mode, borrow errors are hard errors (not warnings).
            // This enforces ownership rules at compile time.
            if config.diagnostics_json {
                let parts: Vec<String> = errors.iter().map(|err| {
                    format!(
                        r#"{{"kind":"borrow_error","code":"E001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                        escape_json(&err.message), escape_json(primary_source), err.span.line, err.span.col
                    )
                }).collect();
                println!("[{}]", parts.join(","));
            } else {
                for err in &errors {
                    eprintln!("error[E001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
                }
            }
            eprintln!("note: {} borrow errors Ã¢â‚¬â€ aborting compilation (--strict mode)", errors.len());
            eprintln!("  = help: Fix ownership violations or remove --strict to treat as warnings.");
            return Err(vec!["compilation failed".to_string()]);
        } else {
            if config.diagnostics_json {
                let parts: Vec<String> = errors.iter().map(|err| {
                    format!(
                        r#"{{"kind":"borrow_warning","code":"E001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                        escape_json(&err.message), escape_json(primary_source), err.span.line, err.span.col
                    )
                }).collect();
                println!("[{}]", parts.join(","));
            } else {
                for err in &errors {
                    eprintln!("warning[E001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
                }
            }
        }
    }

    // Stage 4.5: Inject external module declarations
    let external_decls = checker.collect_external_decls(&program);
    if !external_decls.is_empty() {
        fn fn_dedup_key(fd: &xiom_ast::FnDecl) -> String {
            if fd.is_method() {
                format!("{}.{}", fd.receiver.as_ref().expect("method has receiver").name, fd.name.name)
            } else {
                fd.name.name.clone()
            }
        }
        let existing_names: std::collections::HashSet<String> = program.items.iter().filter_map(|i| match i {
            xiom_ast::TopDecl::Type(td) => Some(td.name.name.clone()),
            xiom_ast::TopDecl::Enum(ed) => Some(ed.name.name.clone()),
            xiom_ast::TopDecl::Fn(fd) => Some(fn_dedup_key(fd)),
            _ => None,
        }).collect();
        for decl in external_decls {
            let name = match &decl {
                xiom_ast::TopDecl::Type(td) => td.name.name.clone(),
                xiom_ast::TopDecl::Enum(ed) => ed.name.name.clone(),
                xiom_ast::TopDecl::Interface(id) => id.name.name.clone(),
                xiom_ast::TopDecl::Fn(fd) => fn_dedup_key(fd),
                xiom_ast::TopDecl::Extern(_) => { program.items.push(decl); continue; }
                xiom_ast::TopDecl::Const(cd) => cd.name.name.clone(),
                _ => continue,
            };
            if !existing_names.contains(&name) {
                program.items.push(decl);
            }
        }
    }

    // Stage 5: Codegen
    let mut emitter = IrEmitter::new();
    emitter.set_check_contracts(config.check_contracts || config.runtime_contracts);
    emitter.set_overflow_checks(config.overflow_checks);
    emitter.set_parallel_codegen(config.parallel_codegen);
    emitter.set_max_recursion_depth(config.max_recursion_depth);
    emitter.set_strict_mode(config.strict_mode);
    emitter.set_hot_reload(config.hot_reload);
    emitter.set_debug_symbols(config.debug_symbols);
    emitter.set_enable_unsafe_direct(config.enable_unsafe_direct);
    if !effective_sources.is_empty() {
        emitter.set_source_file(effective_sources[0].clone());
    }
    emitter.set_target_triple(match config.target {
        Target::Wasm => "wasm32-unknown-unknown",
        Target::Wasi => "wasm32-wasi",
        Target::Arm => "aarch64-unknown-linux-gnu",
        Target::RisCv => "riscv64gc-unknown-linux-gnu",
        Target::Native => {
            if cfg!(target_os = "windows") { "x86_64-pc-windows-msvc" }
            else if cfg!(target_os = "linux") { "x86_64-unknown-linux-gnu" }
            else if cfg!(target_os = "macos") { "x86_64-apple-darwin" }
            else { "x86_64-unknown-linux-gnu" }
        },
    });
    let llvm_ir = match emitter.compile_program(&program) {
        Ok(ir) => ir,
        Err(e) => {
            if config.diagnostics_json {
                println!(r#"{{"kind":"codegen_error","code":"C001","message":"{}","location":{{"file":"{}","line":0,"col":0}}}}"#,
                    escape_json(&e), escape_json(primary_source));
            } else {
                eprintln!("error[C001]: codegen: {e}");
            }
            return Err(vec!["compilation failed".to_string()]);
        }
    };

    if config.diagnostics_json {
        println!(r#"{{"status":"ok"}}"#);
        return Ok(());
    }

    if config.emit_ir || (config.output_file.is_none() && !config.do_run && config.target == Target::Native) {
        println!("{llvm_ir}");
        return Ok(());
    }

    // M12: Fix codegen bug Ã¢â‚¬â€ inttoptr-to-i8* registers stored as i8 instead of i8*
    // The codegen may emit: %X = inttoptr i64 %Y to i8*; store i8 %X, i8** %A
    // which is a type mismatch. Fix: store i8* %X, i8** %A.
    let llvm_ir = fix_inttoptr_store_mismatch(&llvm_ir);

    // Stage 6: Compile to binary via clang
    let default_output = match config.target {
        Target::Wasm | Target::Wasi => "a.wasm",
        Target::Arm | Target::RisCv => "a.out",
        Target::Native => {
            if cfg!(windows) { "a.exe" } else { "a.out" }
        }
    };
    let output = config.output_file.as_deref().unwrap_or(default_output);

    let ir_path = format!("{output}.ll");
    // M10: For shared library JIT, add dllexport to main() so it's callable.
    let llvm_ir = if config.shared_lib && cfg!(windows) {
        add_dllexport_to_main(&llvm_ir)
    } else {
        llvm_ir
    };

    if let Err(e) = fs::write(&ir_path, &llvm_ir) {
        eprintln!("error: cannot write IR file: {e}");
        return Err(vec!["compilation failed".to_string()]);
    }

    // 7D.4: Generate export manifest for hot reload host
    if config.hot_reload {
        generate_export_manifest(&program, output);
    }

    let opt = find_tool("opt", &[
        "C:\\Program Files\\LLVM\\bin\\opt.exe",
        "/usr/bin/opt",
        "/usr/local/bin/opt",
    ]);

    if let Some(opt_path) = &opt {
        let verify_status = Command::new(opt_path)
            .args(["-verify", &ir_path])
            .output();
        match verify_status {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  warning: LLVM IR verification failed: {}", stderr.lines().next().unwrap_or("unknown error"));
                eprintln!("  note: proceeding with compilation; check the generated IR at {}", ir_path);
            }
            Err(_) => {}
        }
    }

    if let Some(opt_path) = &opt {
        // D1 (2026-08-08): default optimization raised from -O1 to -O2.
        // clang/LLVM miscompiles (stack overflow / illegal-instruction traps)
        // the IR shape produced for native Int128 loops containing inlined
        // Vec operations at -O0/-O1; -O2's mem2reg+SSA produces correct code.
        // Verified: i128 loop + Vec.push crashes at -O0/-O1, works at -O2.
        let opt_level = if config.release { "-O3" } else { "-O2" };
        let opt_status = Command::new(opt_path)
            .args([opt_level, "-S", "-o", &ir_path, &ir_path])
            .status();
        if let Ok(s) = opt_status {
            if !s.success() {
                eprintln!("  warning: opt -O2 failed, proceeding with unoptimized IR");
                let _ = fs::write(&ir_path, &llvm_ir);
            }
        }
    }

    #[allow(unused_mut)]
    let mut asm_objects: Vec<String> = Vec::new();
    #[cfg(feature = "nasm")]
    {
        let runtime_dir = find_runtime_c().and_then(|p| {
            std::path::Path::new(&p).parent().map(|d| d.to_path_buf())
        });
        let build_dir = std::path::PathBuf::from("build");
        let nasm = find_nasm();
        if let (Some(nasm_path), Some(rt_dir)) = (&nasm, &runtime_dir) {
            let asm_files = ["crypto_x86_64.asm", "mem_x86_64.asm", "context_switch.asm"];
            let obj_ext = if cfg!(target_os = "windows") { "obj" } else { "o" };
            let nasm_fmt = if cfg!(target_os = "windows") { "win64" }
                           else if cfg!(target_os = "macos") { "macho64" }
                           else { "elf64" };
            let nasm_path = nasm_path.clone();
            for asm_file in &asm_files {
                let asm_path = rt_dir.join(asm_file);
                let obj_name = format!("{}.{}", asm_file, obj_ext);
                let obj_path = rt_dir.join(&obj_name);
                let build_obj = build_dir.join(&obj_name);
                if build_obj.exists() {
                    asm_objects.push(build_obj.to_string_lossy().to_string());
                } else if asm_path.exists() {
                    if !obj_path.exists() || is_newer(&asm_path, &obj_path) {
                        let status = Command::new(&nasm_path)
                            .args(["-f", nasm_fmt, &asm_path.to_string_lossy(), "-o", &obj_path.to_string_lossy()])
                            .status();
                        if status.map_or(false, |s| s.success()) {
                            asm_objects.push(obj_path.to_string_lossy().to_string());
                        } else {
                            let _ = std::fs::remove_file(&obj_path);
                        }
                    } else {
                        asm_objects.push(obj_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let clang = find_tool("clang", &[
        "C:\\Program Files\\LLVM\\bin\\clang.exe",
        "/usr/bin/clang",
        "/usr/local/bin/clang",
    ]);

    match clang {
        Some(clang_path) => {
            let mut cmd = Command::new(&clang_path);
            // v0.58: full ISA enablement for the NATIVE x86_64 runtime. SSE/SSE2
            // are x86-64 baseline; AES-NI + AVX + AVX2 + AVX-512 (F/BW/DQ/VL) are
            // enabled unconditionally so stdlib runtime C can use the whole SIMD/
            // crypto instruction set without per-function target attributes
            // (simd_runtime.c already uses __attribute__((target(...))) where it
            // needs them; the global flags remove that requirement for new code).
            // PRODUCTION RULE: wide-ISA execution MUST be gated at runtime via the
            // CPUID dispatch in simd_runtime.c (xiom_simd_has_avx2/avx512/...) —
            // the -O2 vectorizer can also emit wide instructions in hot loops, so
            // only dispatch-gated code paths may rely on AVX-512 presence.
            if config.target == Target::Native && cfg!(target_arch = "x86_64") {
                cmd.arg("-maes");
                cmd.arg("-mavx");
                cmd.arg("-mavx2");
                // BUG 20 fix (2026-08-12): AVX-512 flags are HOST-CPUID-gated.
                // The -O2 vectorizer emits AVX-512 (zmm) in ORDINARY float loops,
                // which traps 0xC000001D on CPUs without AVX-512 (Zen 2 CI
                // machine) — the runtime's CPUID dispatch gates only the
                // INTENTIONAL SIMD calls, never the vectorizer, so the FLAGS
                // themselves must match the host. Builds on AVX-512 machines
                // still get the full set; the simd_runtime dispatch covers
                // foreign machines at execution time.
                if std::arch::is_x86_feature_detected!("avx512f") {
                    cmd.arg("-mavx512f");
                    cmd.arg("-mavx512bw");
                    cmd.arg("-mavx512dq");
                    cmd.arg("-mavx512vl");
                }
            }
            if asm_objects.is_empty() { cmd.arg("-DXIOM_NO_ASM"); }
            if config.debug_symbols { cmd.arg("-g"); }
            // v0.56: Apply optimization level to clang (same as opt passes).
            // D1: default -O2 (see opt-level comment above Ã¢â‚¬â€ -O0/-O1
            // miscompile native-Int128 loop + Vec IR shapes).
            if config.release { cmd.arg("-O3"); } else { cmd.arg("-O2"); }
            // v0.56: ThinLTO for 20-40% smaller/faster binaries
            if config.lto {
                cmd.arg("-flto=thin");
                cmd.arg("-fuse-ld=lld");
            }
            // Suppress MSVC deprecation warnings (fopen, etc.) in the runtime C code.
            cmd.arg("-D_CRT_SECURE_NO_WARNINGS");
            // Suppress deprecated-declaration warnings (e.g. GetVersionExA) on
            // Windows, matching the JIT runtime build (xiom-jit/src/lib.rs).
            if cfg!(windows) {
                cmd.arg("-Wno-deprecated-declarations");
            }
            // POSIX (Linux/WSL) native links need the math library for the
            // stdlib's extern math fns (exp/ln/sqrt/etc. — math.xi FFI).
            if !cfg!(windows) && config.target == Target::Native {
                cmd.arg("-lm");
            }
            // 7E.1: Sanitizer flags
            if let Some(ref sanitizer) = config.sanitize {
                cmd.arg(&format!("-fsanitize={}", sanitizer));
                // Address sanitizer needs -g for line numbers
                if sanitizer == "address" { cmd.arg("-g"); cmd.arg("-fno-omit-frame-pointer"); }
            }
            // 7E.2: Stack protector (stack canaries)
            if config.stack_protector {
                cmd.arg("-fstack-protector");
            }
            if config.shared_lib { cmd.arg("-shared"); }
            if config.static_lib { cmd.arg("-c"); }
            match config.target {
                Target::Wasm => {
                    cmd.args(["--target=wasm32-unknown-unknown", "-nostdlib", "-Wl,--no-entry", "-Wl,--export-all"]);
                }
                Target::Wasi => {
                    cmd.args(["--target=wasm32-wasi", "-nostdlib", "-Wl,--no-entry", "-Wl,--export-all"]);
                }
                Target::Arm => {
                    cmd.args(["--target=aarch64-unknown-linux-gnu"]);
                }
                Target::RisCv => {
                    cmd.args(["--target=riscv64gc-unknown-linux-gnu"]);
                }
                Target::Native => {
                    if cfg!(target_os = "windows") {
                        cmd.args(["-Xlinker", "/SUBSYSTEM:CONSOLE", "-Xlinker", "/STACK:8388608,8388608", "-Xlinker", "/Brepro"]);
                    }
                }
            }
            if config.target != Target::Wasm && config.target != Target::Wasi {
                let runtime_c_files = find_runtime_c_files();
                // M21: Deduplicate C sources by canonical path to prevent duplicate
                // symbols when --c-source overlaps with auto-discovered runtime files.
                let mut seen_c_sources: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut add_c_source = |cmd: &mut Command, path: &str| {
                    let canonical = std::path::Path::new(path).canonicalize()
                        .unwrap_or_else(|_| std::path::PathBuf::from(path));
                    let key = canonical.to_string_lossy().to_lowercase();
                    if seen_c_sources.insert(key) {
                        cmd.arg(path);
                    }
                };
                if runtime_c_files.is_empty() {
                    if let Some(rt) = find_runtime_c() {
                        add_c_source(&mut cmd, &rt);
                    }
                } else {
                    for rt in &runtime_c_files {
                        let abs_rt = if std::path::Path::new(rt).is_absolute() {
                            rt.clone()
                        } else {
                            std::env::current_dir().unwrap_or_default().join(rt).to_string_lossy().to_string()
                        };
                        add_c_source(&mut cmd, &abs_rt);
                    }
                }
                for cs in &config.c_sources {
                    add_c_source(&mut cmd, cs);
                }
            }
            let cwd0 = std::env::current_dir().unwrap_or_default();
            let abs_output = if std::path::Path::new(output).is_absolute() { output.to_string() } else { cwd0.join(output).to_string_lossy().to_string() };

            let unique_tmp = std::env::temp_dir().join(format!(
                "xiomlink_{}_{}",
                std::process::id(),
                output.replace(['\\', '/', ':', '.'], "_")
            ));
            let _ = std::fs::create_dir_all(&unique_tmp);
            const STAGED_IR_NAME: &str = "xiominput.ll";
            if let Err(e) = fs::copy(&ir_path, unique_tmp.join(STAGED_IR_NAME)) {
                eprintln!("error: cannot stage IR file into temp dir: {e}");
                return Err(vec!["compilation failed".to_string()]);
            }
            cmd.args(["-o", &abs_output, STAGED_IR_NAME]);
            if config.target == Target::Native {
                for obj in &asm_objects { cmd.arg(obj); }
            }
            if config.target != Target::Wasm && config.target != Target::Wasi {
                for lp in &config.link_paths {
                    cmd.arg(&format!("-L{lp}"));
                }
                for lib in &config.link_libs {
                    cmd.arg(&format!("-l{lib}"));
                }
            }
            cmd.current_dir(&unique_tmp);
            let clang_output = cmd.output();
            let _ = std::fs::remove_dir_all(&unique_tmp);
            match clang_output {
                Ok(out) if out.status.success() => {
                    let _ = fs::remove_file(&ir_path);
                    eprintln!("  compiled: {output}");

                    // v0.54: Binary cache Ã¢â‚¬â€ store compiled binary keyed by SHA-256 of source.
                    // Subsequent runs with --run --cache skip the entire compile pipeline.
                    if config.cache && source_paths.len() == 1 {
                        let source_path = &source_paths[0];
                        if let Ok(source) = fs::read_to_string(source_path) {
                            let cache_source = if config.script_mode {
                                crate::implicit_main::wrap_implicit_main(&source)
                            } else {
                                source
                            };
                            crate::jit::script_cache_put(&cache_source, &PathBuf::from(&abs_output));
                        }
                    }

                    if config.do_run && config.target == Target::Native {
                        let exe = if output.contains('\\') || output.contains('/') {
                            output.to_string()
                        } else {
                            // Platform-appropriate relative path prefix
                            if cfg!(windows) { format!(".\\{output}") }
                            else { format!("./{output}") }
                        };
                        let run_status = Command::new(&exe).status();
                        match run_status {
                            Ok(s) => eprintln!("  exit code: {}", s.code().unwrap_or(-1)),
                            Err(e) => {
                                eprintln!("error: cannot run '{exe}': {e}");
            std::process::exit(1);
                            }
                        }
                    }

                    if config.target == Target::Wasm || config.target == Target::Wasi {
                        if let Ok(meta) = fs::metadata(output) {
                            eprintln!("  wasm size: {} bytes", meta.len());
                        }
                    }
                    Ok(())
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("error: clang failed with exit code {}", out.status.code().unwrap_or(-1));
                    // Only show the "missing stdio.h" hint when clang actually
                    // reports a fatal error about it (not when it appears in
                    // diagnostic notes alongside other errors like invalid IR).
                    if stderr.contains("fatal error:") && stderr.contains("stdio.h") {
                        eprintln!("  -> Missing C standard library headers (stdio.h).");
                        eprintln!("  -> Install Visual Studio 2022 Build Tools with 'Desktop development with C++':");
                        eprintln!("      winget install Microsoft.VisualStudio.2022.BuildTools");
                        eprintln!("    Or run: .\\install_deps.ps1");
                    }
                    eprintln!("  stderr: {}", stderr.trim());
                    return Err(vec!["compilation failed".to_string()]);
                }
                Err(e) => {
                    eprintln!("error: cannot run clang: {e}");
                    eprintln!("note: LLVM IR written to {ir_path}");
                    return Err(vec!["compilation failed".to_string()]);
                }
            }
        }
        None => {
            eprintln!("note: clang not found Ã¢â‚¬â€ LLVM IR written to {ir_path}");
            match config.target {
                Target::Wasm => {
                    eprintln!("  compile manually: clang --target=wasm32-unknown-unknown -nostdlib -Wl,--no-entry -Wl,--export-all -o {output} {ir_path}");
                }
                Target::Wasi => {
                    eprintln!("  compile manually: clang --target=wasm32-wasi -nostdlib -Wl,--no-entry -Wl,--export-all -o {output} {ir_path}");
                }
                Target::Arm => {
                    eprintln!("  compile manually: clang --target=aarch64-unknown-linux-gnu -o {output} {ir_path}");
                }
                Target::RisCv => {
                    eprintln!("  compile manually: clang --target=riscv64gc-unknown-linux-gnu -o {output} {ir_path}");
                }
                Target::Native => {
                    eprintln!("  compile manually: clang -o {output} {ir_path}");
                }
            }
            return Err(vec!["compilation failed".to_string()]);
        }
    }
}

pub fn merge_programs(programs: Vec<xiom_ast::Program>) -> xiom_ast::Program {
    let mut items: Vec<xiom_ast::TopDecl> = Vec::new();
    for p in programs {
        for item in p.items {
            match item {
                xiom_ast::TopDecl::Module(md) => {
                    let md_name = md.name.name.clone();
                    if let Some(existing) = items.iter_mut().find_map(|i| {
                        if let xiom_ast::TopDecl::Module(emd) = i {
                            if emd.name.name == md_name { Some(emd) } else { None }
                        } else { None }
                    }) {
                        existing.items.extend(md.items);
                    } else {
                        items.push(xiom_ast::TopDecl::Module(md));
                    }
                }
                other => items.push(other),
            }
        }
    }
    xiom_ast::Program::new(items, xiom_ast::Span::new(0, 0))
}

/// 7D.4: Generate a module export manifest for the hot reload host.
/// Lists all `pub fn` names and their djb2 hash table indices.
/// Format: `fn_name:hash_index` (one per line).
pub fn generate_export_manifest(program: &xiom_ast::Program, output_base: &str) {
    let manifest_path = format!("{output_base}.exports");
    let mut exports = Vec::new();

    fn collect_pub_fns(items: &[xiom_ast::TopDecl], exports: &mut Vec<String>) {
        for item in items {
            match item {
                xiom_ast::TopDecl::Fn(fd) if fd.is_pub => {
                    let name = if let Some(ref recv) = fd.receiver {
                        format!("{}.{}", recv.name, fd.name.name)
                    } else {
                        fd.name.name.clone()
                    };
                    // Compute djb2 hash (same as runtime)
                    let hash: u64 = name.bytes().fold(5381u64, |h, b| {
                        ((h << 5).wrapping_add(h)).wrapping_add(b as u64)
                    });
                    let idx = hash % 1024;
                    exports.push(format!("{name}:{idx}"));
                }
                xiom_ast::TopDecl::Module(md) => {
                    collect_pub_fns(&md.items, exports);
                }
                _ => {}
            }
        }
    }

    collect_pub_fns(&program.items, &mut exports);

    if !exports.is_empty() {
        let content = exports.join("\n") + "\n";
        let _ = fs::write(&manifest_path, &content);
    }
}

pub fn resolve_source_files(args: &[String]) -> Vec<String> {
    let mut sources = Vec::new();
    let mut skip_next = false;

    for arg in args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "-o" | "--target" | "--verify-output" | "--link" | "--link-path" | "--c-source"
            | "--timeout" | "--max-memory-mb" | "--max-depth" | "--jobs" | "--sanitize" | "--ai-model" | "--ai-timeout") {
            skip_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        if matches!(arg.as_str(), "wasm" | "arm" | "riscv") {
            continue;
        }
        sources.push(arg.clone());
    }

    if sources.is_empty() {
        return Vec::new();
    }

    if sources.len() == 1 {
        if let Ok(metadata) = fs::metadata(&sources[0]) {
            if metadata.is_dir() {
                return load_package_dir(&sources[0]);
            }
        }
    }

    sources
}

pub fn load_package_dir(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let package_path = format!("{}/package.xi", dir);

    if fs::metadata(&package_path).is_ok() {
        if let Ok(modules) = parse_package_manifest(&package_path) {
            let xi_files = scan_xi_files(dir);
            let module_file_map = build_module_file_map(&xi_files);
            for module_path in &modules {
                if let Some(file) = module_file_map.get(module_path) {
                    files.push(file.clone());
                }
            }
            if !files.is_empty() {
                return files;
            }
        }
    }

    files = scan_xi_files(dir);
    files
}

pub fn scan_xi_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name != "package.xi" {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    files
}

pub fn build_module_file_map(files: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            let module_path = extract_module_path(&content);
            if let Some(path) = module_path {
                map.insert(path, file.clone());
            }
        }
    }
    map
}

pub fn extract_module_path(source: &str) -> Option<String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].is_eof() {
            return None;
        }
        if let xiom_lexer::TokenKind::Module = &tokens[i].kind {
            i += 1;
            let mut path_parts = Vec::new();
            if i < tokens.len() {
                if let xiom_lexer::TokenKind::Ident(name) = &tokens[i].kind {
                    path_parts.push(name.clone());
                    i += 1;
                }
                while i < tokens.len() {
                    if let xiom_lexer::TokenKind::Dot = &tokens[i].kind {
                        i += 1;
                        if i < tokens.len() {
                            if let xiom_lexer::TokenKind::Ident(name) = &tokens[i].kind {
                                path_parts.push(name.clone());
                                i += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            if !path_parts.is_empty() {
                return Some(path_parts.join("."));
            }
            return None;
        }
        i += 1;
    }
    None
}

fn parse_package_manifest(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("cannot read package manifest: {e}"))?;
    let mut modules = Vec::new();
    let mut in_modules = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("modules:") || trimmed.starts_with("\"modules\":") {
            in_modules = true;
        }

        if in_modules {
            let bracket_start = trimmed.find('[');
            let bracket_end = trimmed.find(']');
            let extract = if let (Some(s), Some(e)) = (bracket_start, bracket_end) {
                &trimmed[s..=e]
            } else if bracket_start.is_some() {
                &trimmed[bracket_start.expect("guarded by is_some above")..]
            } else if bracket_end.is_some() {
                return Ok(modules);
            } else {
                continue;
            };

            if let Some(start) = extract.find('[') {
                let inner = &extract[start..];
                let inner = inner.trim_start_matches('[');
                let inner = if let Some(end) = inner.rfind(']') {
                    &inner[..=end]
                } else {
                    inner
                };
                let inner = inner.trim_end_matches(']');

                for part in inner.split(',') {
                    let part = part.trim().trim_matches('"').trim();
                    if !part.is_empty() {
                        modules.push(part.to_string());
                    }
                }
            }

            if trimmed.contains(']') {
                break;
            }
        }
    }

    Ok(modules)
}

pub fn find_stdlib_dirs() -> Vec<String> {
    let mut roots: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(env_dir) = std::env::var("XIOM_STDLIB") {
        if !env_dir.trim().is_empty() {
            roots.push(std::path::PathBuf::from(env_dir));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        let mut hops = 0;
        while let Some(dir) = cur {
            let candidate = dir.join("stdlib");
            if candidate.is_dir() {
                roots.push(candidate);
                break;
            }
            hops += 1;
            if hops > 8 {
                break;
            }
            cur = dir.parent();
        }
    }

    roots.push(std::path::PathBuf::from("stdlib"));

    if let Some(repo_stdlib) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|repo| repo.join("stdlib"))
    {
        roots.push(repo_stdlib);
    }

    let mut dirs: Vec<String> = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let root_str = root.to_string_lossy().to_string();
        if !dirs.contains(&root_str) {
            dirs.push(root_str);
        }
        let xiom_sub = root.join("xiom");
        if xiom_sub.is_dir() {
            let sub_str = xiom_sub.to_string_lossy().to_string();
            if !dirs.contains(&sub_str) {
                dirs.push(sub_str);
            }
        }
    }
    dirs
}

/// 5e.3 G-30/G-31: walk up from a source file's parent directory looking for
/// project root markers (xiom.toml, package.xi, xiom.lock, .git, src/). When found, the
/// project root and its src/ subdirectory are added as source_dirs so the
/// catalog can resolve cross-directory `use xiom.*` imports.
pub fn find_project_root(file_path: &Path) -> Option<PathBuf> {
    let start = if file_path.is_dir() { file_path.to_path_buf() } else {
        file_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    };
    let mut cur = Some(start.as_path());
    let mut hops = 0;
    while let Some(dir) = cur {
        // Project markers (in priority order)
        if dir.join("xiom.toml").is_file()  { return Some(dir.to_path_buf()); }
        if dir.join("package.xi").is_file() { return Some(dir.to_path_buf()); }
        if dir.join("xiom.lock").is_file()    { return Some(dir.to_path_buf()); }
        if dir.join(".git").is_dir()          { return Some(dir.to_path_buf()); }
        if dir.join("src").is_dir()           { return Some(dir.to_path_buf()); }
        hops += 1;
        if hops > 8 { break; }
        cur = dir.parent();
    }
    None
}

pub fn find_runtime_c() -> Option<String> {
    // Walk UP from the xiom binary (up to 8 levels) looking for the standard
    // layouts. This is the reliable path for dev (`target/debug/xiom.exe` →
    // `<repo>/stdlib/runtime/xiom_runtime.c`) AND for the playground server
    // whose cwd is NOT the repo root. The old exe.parent().parent() guess
    // (`target/debug` → `target`) never reached the project root.
    if let Ok(exe) = std::env::current_exe() {
        let mut search = exe.parent();
        for _ in 0..8 {
            if let Some(dir) = search {
                let cand1 = dir.join("stdlib").join("runtime").join("xiom_runtime.c");
                if cand1.is_file() { return Some(cand1.to_string_lossy().to_string()); }
                let cand2 = dir.join("runtime").join("xiom_runtime.c");
                if cand2.is_file() { return Some(cand2.to_string_lossy().to_string()); }
                search = dir.parent();
            } else { break; }
        }
    }
    let candidates: Vec<String> = {
        let mut paths = vec![
            "stdlib\\runtime\\xiom_runtime.c".to_string(),
            "stdlib/runtime/xiom_runtime.c".to_string(),
        ];
        if let Ok(sd) = std::env::var("XIOM_STDLIB") {
            paths.push(format!("{}/../runtime/xiom_runtime.c", sd));
            paths.push(format!("{}\\..\\runtime\\xiom_runtime.c", sd));
        }
        paths
    };
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

/// M10: Add dllexport to the @main function definition for shared library JIT.
fn add_dllexport_to_main(ir: &str) -> String {
    // Replace `define i64 @main(` with `define dllexport i64 @main(`
    ir.replace("define i64 @main(", "define dllexport i64 @main(")
        .replace("define void @main(", "define dllexport void @main(")
}

/// M12: Fix IR type mismatch: inttoptr i64 %X to i8* followed by store i8 %X, i8** %Y
/// Inttoptr produces i8* but codegen may emit store i8 (expecting non-pointer).
/// Only fix when the store target is i8** (pointer-to-pointer), not i8* (raw byte pointer).
fn fix_inttoptr_store_mismatch(ir: &str) -> String {
    use std::collections::HashSet;
    // Collect all registers defined by inttoptr to i8*
    let mut i8p_regs: HashSet<String> = HashSet::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.contains("inttoptr i64") && trimmed.contains("to i8*") {
            if let Some(reg) = trimmed.split('=').next() {
                let reg = reg.trim().to_string();
                if reg.starts_with('%') {
                    i8p_regs.insert(reg);
                }
            }
        }
    }

    if i8p_regs.is_empty() { return ir.to_string(); }

    // Fix lines: `store i8 %reg, i8** %ptr` Ã¢â€ â€™ `store i8* %reg, i8** %ptr`
    // Only when the target is i8** (pointer-to-pointer, from alloca i8*)
    let mut result = String::with_capacity(ir.len());
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("store i8 ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 4 {
                let src_reg = parts[2].trim_end_matches(',');
                // Check target type: must be i8** (pointer to pointer), not i8*
                // Format: store i8 %reg, i8** %ptr  => parts: [store, i8, %reg,, i8**, %ptr]
                let target_is_ptr_to_ptr = parts.len() >= 4
                    && parts[3] == "i8**";
                if i8p_regs.contains(src_reg) && target_is_ptr_to_ptr {
                    let fixed = line.replacen("store i8 ", "store i8* ", 1);
                    result.push_str(&fixed);
                    result.push('\n');
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result.trim_end_matches('\n').to_string()
}

pub fn find_runtime_c_files() -> Vec<String> {
    // XIOM_RUNTIME_DIR override Ã¢â‚¬â€ production deployments set this explicitly
    if let Ok(rt_dir) = std::env::var("XIOM_RUNTIME_DIR") {
        let dir = std::path::Path::new(&rt_dir);
        if dir.is_dir() {
            let mut c_files: Vec<String> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("c") {
                        c_files.push(p.to_string_lossy().to_string());
                    }
                }
            }
            if !c_files.is_empty() { c_files.sort(); return c_files; }
        }
    }

    let mut dir_candidates: Vec<String> = vec![
        "stdlib\\runtime".to_string(),
        "stdlib/runtime".to_string(),
        "runtime".to_string(),
    ];

    // Walk UP from the xiom binary (up to 8 levels) — reliable for dev
    // (target/debug/xiom.exe → <repo>/stdlib/runtime) and for servers whose
    // cwd is not the repo root (playground).
    if let Ok(exe) = std::env::current_exe() {
        let mut search = exe.parent();
        for _ in 0..8 {
            if let Some(dir) = search {
                let cand = dir.join("stdlib").join("runtime");
                if cand.is_dir() {
                    dir_candidates.push(cand.to_string_lossy().to_string());
                }
                let cand2 = dir.join("runtime");
                if cand2.is_dir() {
                    dir_candidates.push(cand2.to_string_lossy().to_string());
                }
                search = dir.parent();
            } else { break; }
        }
    }

    // Search relative to the xiom binary location (production installs)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // bin/xiom.exe -> ../runtime/ (standard release layout)
            dir_candidates.push(format!("{}/../runtime", exe_dir.display()));
            dir_candidates.push(format!("{}\\..\\runtime", exe_dir.display()));
            // bin/xiom.exe -> ../stdlib/runtime/ (stdlib layout)
            dir_candidates.push(format!("{}/../stdlib/runtime", exe_dir.display()));
            dir_candidates.push(format!("{}\\..\\stdlib\\runtime", exe_dir.display()));
            // Same directory as binary
            dir_candidates.push(format!("{}/runtime", exe_dir.display()));
            dir_candidates.push(format!("{}\\runtime", exe_dir.display()));
            // Grandparent-based (for `target/debug/xiom.exe` -> `../../runtime/`)
            if let Some(grandparent) = exe_dir.parent() {
                dir_candidates.push(format!("{}/runtime", grandparent.display()));
                dir_candidates.push(format!("{}\\runtime", grandparent.display()));
                dir_candidates.push(format!("{}/stdlib/runtime", grandparent.display()));
                dir_candidates.push(format!("{}\\stdlib\\runtime", grandparent.display()));
            }
        }
    }

    // CARGO_MANIFEST_DIR-based paths (dev/test environments)
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = std::path::Path::new(&manifest);
        for depth in 2..5 {
            let mut p = base.to_path_buf();
            for _ in 0..depth { p = p.join(".."); }
            dir_candidates.push(format!("{}/stdlib/runtime", p.display()));
            dir_candidates.push(format!("{}/runtime", p.display()));
            dir_candidates.push(format!("{}\\stdlib\\runtime", p.display()));
            dir_candidates.push(format!("{}\\runtime", p.display()));
        }
    }

    for dir in &dir_candidates {
        let dir_path = std::path::Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }
        let mut c_files: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|e| e.to_str()) == Some("c")
                {
                    c_files.push(path.to_string_lossy().to_string());
                }
            }
        }
        if !c_files.is_empty() {
            c_files.sort();
            return c_files;
        }
    }
    Vec::new()
}

pub fn find_tool(name: &str, extra_paths: &[&str]) -> Option<String> {
    for path in extra_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    if Command::new(name).arg("--version").output().is_ok() {
        return Some(name.to_string());
    }
    None
}

pub fn find_nasm() -> Option<String> {
    let candidates: Vec<&str> = if cfg!(target_os = "windows") {
        vec![
            "C:\\Program Files\\NASM\\nasm.exe",
            "C:\\Users\\lefte\\AppData\\Local\\bin\\NASM\\nasm.exe",
        ]
    } else {
        vec![
            "/usr/local/bin/nasm",
            "/usr/bin/nasm",
            "/opt/homebrew/bin/nasm",
        ]
    };
    find_tool("nasm", &candidates)
}

pub fn is_newer(src: &std::path::Path, dst: &std::path::Path) -> bool {
    if let (Ok(sm), Ok(dm)) = (src.metadata(), dst.metadata()) {
        if let (Ok(st), Ok(dt)) = (sm.modified(), dm.modified()) {
            return st > dt;
        }
    }
    true
}

/// 5c-R: Display the error code reference for `--explain <code>`.
/// Reads from `docs/error_codes/{code}.md` relative to the project root.
pub fn explain_error(code: &str) {
    let root = std::env::current_dir().unwrap_or_default();
    let path = root.join("docs").join("error_codes").join(format!("{code}.md"));
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            println!("{content}");
            println!("Ã¢â€â‚¬Ã¢â€â‚¬");
            println!("For the full error-code index: docs/error_codes/README.md");
        }
        Err(_) => {
            eprintln!("Unknown error code: {code}");
            eprintln!("Available codes are listed in docs/error_codes/README.md");
            eprintln!("Run: xiom --explain X0010  (for type mismatch)");
            std::process::exit(1);
        }
    }
}

pub fn render_error(code: &str, span: &xiom_ast::Span, message: &str, source_text: Option<&str>, help: Option<&str>, note: Option<&str>) {
    eprintln!("error[{code}]: {l}:{c}: {m}", l = span.line, c = span.col, m = message);
    if let Some(text) = source_text {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = span.line.saturating_sub(1) as usize;
        if line_idx < lines.len() {
            let source_line = lines[line_idx];
            eprintln!("  |");
            eprintln!("{ln:>3} | {source_line}", ln = span.line);
            if span.col > 0 {
                let padding = span.col.saturating_sub(1) as usize;
                let caret = " ".repeat(padding.min(100));
                eprintln!("  | {caret}^");
            }
        }
    }
    if let Some(n) = note {
        eprintln!("  = note: {n}");
    }
    if let Some(h) = help {
        eprintln!("  = help: {h}");
    }
}

pub fn diagnostic_for(msg: &str) -> (Option<String>, Option<String>) {
    if msg.contains("undefined variable") || msg.contains("not found in this scope") {
        (Some("Check the spelling. If from another module, add a `use` declaration.".to_string()),
         Some("The compiler cannot resolve this name. Without it, the expression has no type.".to_string()))
    } else if msg.contains("type mismatch") {
        (Some("Expected and actual types differ. Consider adding a type annotation or conversion.".to_string()),
         Some("Type mismatches prevent the compiler from guaranteeing memory safety.".to_string()))
    } else if msg.contains("cannot call") && msg.contains("on this expression") {
        (Some("The value's type does not support this method. Check the type definition for available methods.".to_string()),
         Some("Method calls require the receiver type to have the method registered.".to_string()))
    } else if msg.contains("has no field") {
        (Some("The struct does not have this field. Check the field name and struct definition.".to_string()),
         Some("Field access on a non-existent field would read undefined memory.".to_string()))
    } else if msg.contains("cannot find") || msg.contains("unresolved") {
        (Some("The identifier is not in scope. Add a `use` import or define it.".to_string()),
         Some("Unresolved names prevent the compiler from generating correct code.".to_string()))
    } else if msg.contains("numeric") || msg.contains("must be numeric") {
        (Some("The operation requires numeric operands (Int, Float64). Check operand types.".to_string()),
         Some("Non-numeric types (Str, Bool, structs) cannot participate in arithmetic.".to_string()))
    } else if msg.contains("annotated") {
        (Some("The declared type does not match the expression. Remove annotation or fix expression.".to_string()),
         Some("Type annotations must match the inferred type for memory safety.".to_string()))
    } else {
        (Some("Review the error and check syntax/types at the indicated location.".to_string()), None)
    }
}

pub fn suggest_fix(msg: &str) -> String {
    diagnostic_for(msg).0.unwrap_or_else(|| "Review the error and check syntax/types.".to_string())
}

pub fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn contract_expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(id) => id.name.clone(),
        Expr::Int(n, _) => n.to_string(),
        Expr::Float(f, _) => {
            if *f == f.floor() && f.is_finite() {
                format!("{}.0", f)
            } else {
                f.to_string()
            }
        }
        Expr::Bool(b, _) => b.to_string(),
        Expr::Str(s, _) => format!("\"{}\"", s),
        Expr::Binary(left, op, right, _) => {
            format!("{} {} {}",
                contract_expr_to_string(left),
                op,
                contract_expr_to_string(right))
        }
        Expr::Unary(op, inner, _) => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
                UnaryOp::Ref => "&",
                UnaryOp::MutRef => "&mut ",
                UnaryOp::BitNot => "~",
                UnaryOp::Deref => "*",
            };
            format!("{}{}", op_str, contract_expr_to_string(inner))
        }
        Expr::Field(obj, field, _) => {
            format!("{}.{}", contract_expr_to_string(obj), field.name)
        }
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            let a: Vec<String> = args.iter().map(|a| contract_expr_to_string(a)).collect();
            format!("{}({})", contract_expr_to_string(func), a.join(", "))
        }
        Expr::Paren(inner, _) => {
            format!("({})", contract_expr_to_string(inner))
        }
        Expr::Index(obj, index, _) => {
            format!("{}[{}]", contract_expr_to_string(obj), contract_expr_to_string(index))
        }
        Expr::AtPre(inner, _) => {
            format!("{}@pre", contract_expr_to_string(inner))
        }
        Expr::Some(inner, _) => {
            format!("Some({})", contract_expr_to_string(inner))
        }
        Expr::None(_) => "None".to_string(),
        Expr::Ok(inner, _) => {
            format!("Ok({})", contract_expr_to_string(inner))
        }
        Expr::Err(inner, _) => {
            format!("Err({})", contract_expr_to_string(inner))
        }
        Expr::Is(expr, _pattern, _) => {
            format!("{} is _", contract_expr_to_string(expr))
        }
        Expr::Imply(left, right, _) => {
            format!("{} => {}", contract_expr_to_string(left), contract_expr_to_string(right))
        }
        Expr::Struct(ident, fields, _spread, _) => {
            let f: Vec<String> = fields.iter()
                .map(|(k, v)| format!("{}: {}", k.name, contract_expr_to_string(v)))
                .collect();
            format!("{}{{{}}}", ident.name, f.join(", "))
        }
        Expr::Array(items, _) => {
            let f: Vec<String> = items.iter().map(|i| contract_expr_to_string(i)).collect();
            format!("[{}]", f.join(", "))
        }
        Expr::Ref(inner, _) => {
            format!("&{}", contract_expr_to_string(inner))
        }
        Expr::MutRef(inner, _) => {
            format!("&mut {}", contract_expr_to_string(inner))
        }
        Expr::Try(inner, _) => {
            format!("{}?", contract_expr_to_string(inner))
        }
        expr => format!("{:?}", expr),
    }
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Named(ident, params) => {
            let base = ident.name.clone();
            if params.is_empty() {
                base
            } else {
                let p: Vec<String> = params.iter().map(|t| type_to_string(t)).collect();
                format!("{}[{}]", base, p.join(", "))
            }
        }
        Type::Ref(inner) => format!("&{}", type_to_string(inner)),
        Type::MutRef(inner) => format!("&mut {}", type_to_string(inner)),
        Type::Option(inner) => format!("Option[{}]", type_to_string(inner)),
        Type::Result(ok, err) => format!("Result[{}, {}]", type_to_string(ok), type_to_string(err)),
        Type::Vec(inner) => format!("Vec[{}]", type_to_string(inner)),
        Type::Slice(inner) => format!("Slice[{}]", type_to_string(inner)),
        Type::Map(k, v) => format!("Map[{}, {}]", type_to_string(k), type_to_string(v)),
        Type::Set(inner) => format!("Set[{}]", type_to_string(inner)),
        Type::Tuple(tys) => {
            let p: Vec<String> = tys.iter().map(|t| type_to_string(t)).collect();
            format!("({})", p.join(", "))
        }
        Type::Ptr(inner) => format!("*{}", type_to_string(inner)),
        Type::Array(size, inner) => {
            format!("[{}]{}", contract_expr_to_string(size), type_to_string(inner))
        }
        Type::Fn(params, ret) => {
            let p: Vec<String> = params.iter().map(|t| type_to_string(t)).collect();
            format!("fn({}) -> {}", p.join(", "), type_to_string(ret))
        }
        Type::ImplTrait(traits) => {
            let names: Vec<String> = traits.iter().map(|t| t.name.clone()).collect();
            format!("impl {}", names.join(" + "))
        }
        Type::AnonStruct(fields) => {
            let parts: Vec<String> = fields.iter()
                .map(|f| format!("{}: {}", f.name.name, type_to_string(&f.ty)))
                .collect();
            format!("{{ {} }}", parts.join("; "))
        }
        Type::Never => "!".to_string(),
    }
}

fn fn_signature_string(fd: &FnDecl) -> String {
    let mut sig = String::new();
    if fd.is_pub {
        sig.push_str("pub ");
    }
    if fd.is_async {
        sig.push_str("async ");
    }
    sig.push_str("fn ");
    if let Some(recv) = &fd.receiver {
        sig.push_str(&recv.name);
        sig.push('.');
    }
    sig.push_str(&fd.name.name);

    if !fd.generics.is_empty() {
        sig.push('[');
        let g: Vec<String> = fd.generics.iter().map(|gp| {
            let mut s = gp.name.name.clone();
            if !gp.bounds.is_empty() {
                s.push_str(": ");
                s.push_str(&gp.bounds.iter().map(|b| b.name.clone()).collect::<Vec<_>>().join(" + "));
            }
            s
        }).collect();
        sig.push_str(&g.join(", "));
        sig.push(']');
    }

    sig.push('(');
    let p: Vec<String> = fd.params.iter()
        .map(|p| format!("{}: {}", p.name.name, type_to_string(&p.ty)))
        .collect();
    sig.push_str(&p.join(", "));
    sig.push(')');

    if let Some(ret) = &fd.return_type {
        sig.push_str(" -> ");
        sig.push_str(&type_to_string(ret));
    }

    sig
}

/// Phase 5d: Public API Ã¢â‚¬â€ dump all contract signatures as JSON.
pub fn dump_contracts_json(program: &Program) -> String {
    let mut items: Vec<String> = Vec::new();

    for decl in &program.items {
        match decl {
            TopDecl::Fn(fd) => {
                if fd.contracts.is_empty() {
                    continue;
                }
                let mut requires: Vec<String> = Vec::new();
                let mut ensures: Vec<String> = Vec::new();
                for c in &fd.contracts {
                    match c {
                        ContractClause::Requires(expr, _) => {
                            requires.push(contract_expr_to_string(expr));
                        }
                        ContractClause::Ensures(expr, _) => {
                            ensures.push(contract_expr_to_string(expr));
                        }
                    }
                }
                let req_json: Vec<String> = requires.iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                let ens_json: Vec<String> = ensures.iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                items.push(format!(
                    r#"{{"function":"{}","type_params":[],"requires":[{}],"ensures":[{}],"signature":"{}"}}"#,
                    escape_json(&fd.name.name),
                    req_json.join(","),
                    ens_json.join(","),
                    escape_json(&fn_signature_string(fd))
                ));
            }
            TopDecl::Type(td) => {
                if td.invariants.is_empty() {
                    continue;
                }
                let invs: Vec<String> = td.invariants.iter()
                    .map(|e| format!("\"{}\"", escape_json(&contract_expr_to_string(e))))
                    .collect();
                items.push(format!(
                    r#"{{"type":"{}","invariants":[{}]}}"#,
                    escape_json(&td.name.name),
                    invs.join(",")
                ));
            }
            TopDecl::Module(md) => {
                items.extend(dump_module_contracts(md));
            }
            TopDecl::Extern(_) => {}
            _ => {}
        }
    }

    format!("[{}]", items.join(","))
}

fn dump_module_contracts(md: &ModuleDecl) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();

    for decl in &md.items {
        match decl {
            TopDecl::Fn(fd) => {
                if fd.contracts.is_empty() {
                    continue;
                }
                let mut requires: Vec<String> = Vec::new();
                let mut ensures: Vec<String> = Vec::new();
                for c in &fd.contracts {
                    match c {
                        ContractClause::Requires(expr, _) => {
                            requires.push(contract_expr_to_string(expr));
                        }
                        ContractClause::Ensures(expr, _) => {
                            ensures.push(contract_expr_to_string(expr));
                        }
                    }
                }
                let req_json: Vec<String> = requires.iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                let ens_json: Vec<String> = ensures.iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                items.push(format!(
                    r#"{{"function":"{}","type_params":[],"requires":[{}],"ensures":[{}],"signature":"{}"}}"#,
                    escape_json(&fd.name.name),
                    req_json.join(","),
                    ens_json.join(","),
                    escape_json(&fn_signature_string(fd))
                ));
            }
            TopDecl::Type(td) => {
                if td.invariants.is_empty() {
                    continue;
                }
                let invs: Vec<String> = td.invariants.iter()
                    .map(|e| format!("\"{}\"", escape_json(&contract_expr_to_string(e))))
                    .collect();
                items.push(format!(
                    r#"{{"type":"{}","invariants":[{}]}}"#,
                    escape_json(&td.name.name),
                    invs.join(",")
                ));
            }
            TopDecl::Module(nested) => {
                items.extend(dump_module_contracts(nested));
            }
            TopDecl::Extern(_) => {}
            _ => {}
        }
    }

    items
}

// ============================================================================
// ============================================================================
// Phase 7B: Industrial Incremental Compilation Ã¢â‚¬â€ Graph-aware Cache
// ============================================================================

/// Get or create the project-level cache database.
/// Cache directory: `<project_root>/.xi_cache/`
pub fn get_project_cache(source_file: &Path) -> Option<xiom_graph::CacheDb> {
    let root = find_project_root(source_file)?;
    Some(xiom_graph::CacheDb::for_project(&root))
}

/// Phase 7B: Check if cached IR is available for a module via the graph-aware cache.
/// Returns Some(cached_ir) if all of the following are true:
/// 1. A project root is found (xiom.toml, package.xi, etc.)
/// 2. The module's source hash matches the cached fingerprint
/// 3. All dependencies are still valid
/// 4. L4 (IR) tier is cached
///
/// Falls back to legacy single-file check for non-project files.
pub fn incremental_check(source_path: &str) -> Option<String> {
    let path = Path::new(source_path);
    let cache = get_project_cache(path)?;

    // Try to find this file in the dependency graph
    let module_path = match xiom_graph::build_project_graph(path) {
        Ok(graph) => {
            // Find the module by file path
            graph.nodes.iter()
                .find(|n| n.file_path == path)
                .map(|n| n.module_path.clone())
        }
        Err(_) => None,
    };

    if let Some(mp) = module_path {
        // Graph-aware cache check
        let entry = cache.get(&mp)?;
        let current_hash = match xiom_graph::hash::file_sha256(path) {
            Ok(h) => h,
            Err(_) => return None,
        };
        if entry.fingerprint.source_hash != current_hash {
            return None;
        }
        if !entry.tiers.l4_ir {
            return None;
        }
        cache.load_tier(&entry.cache_key, "ll")
    } else {
        // Legacy fallback: simple content-hash check
        let current_hash = match xiom_graph::hash::file_sha256(path) {
            Ok(h) => h,
            Err(_) => return None,
        };
        let cache_key = &current_hash[..current_hash.len().min(16)];
        cache.load_tier(cache_key, "ll")
    }
}

/// Phase 7B: Save compiled IR to the graph-aware cache.
/// Stores the L4 (IR) tier and updates the cache entry with fingerprint metadata.
pub fn incremental_save(source_path: &str, ir: &str) {
    let path = Path::new(source_path);
    let cache = match get_project_cache(path) {
        Some(c) => c,
        None => return,
    };

    let current_hash = match xiom_graph::hash::file_sha256(path) {
        Ok(h) => h,
        Err(_) => return,
    };
    let cache_key = xiom_graph::hash::short_hash(&current_hash).to_string();

    // Store the IR
    cache.store_tier(&cache_key, "ll", ir);

    // Try to build a graph entry for this file
    if let Ok(graph) = xiom_graph::build_project_graph(path) {
        if let Some(node) = graph.nodes.iter().find(|n| n.file_path == path) {
            let dependents: Vec<String> = graph
                .reverse_edges
                .get(graph.path_to_idx.get(&node.module_path).copied().unwrap_or(0))
                .map(|v| v.iter().map(|&i| graph.nodes[i].module_path.clone()).collect())
                .unwrap_or_default();

            let mut tiers = xiom_graph::CacheTiers::default();
            tiers.l4_ir = true;

            let entry = xiom_graph::cache::make_cache_entry(node, dependents, tiers);
            cache.insert(entry);
        }
    } else {
        // Store a minimal entry for non-project files
        let entry = xiom_graph::CacheEntry {
            module_path: path.to_string_lossy().to_string(),
            file_path: path.to_string_lossy().to_string(),
            fingerprint: xiom_graph::Fingerprint {
                source_hash: current_hash,
                signature_hash: String::new(),
            },
            cache_key: cache_key.clone(),
            tiers: {
                let mut t = xiom_graph::CacheTiers::default();
                t.l4_ir = true;
                t
            },
            dependencies: vec![],
            dependents: vec![],
            last_compiled: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        cache.insert(entry);
    }
}

// ============================================================================
// Tests Ã¢â‚¬â€ compile_with_diagnostics
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn default_config() -> CompileConfig {
        CompileConfig::default()
    }

    fn write_temp_file(name: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("xiom_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_compile_simple_program() {
        let path = write_temp_file("simple.xi", "fn main() -> Int { return 42; }");
        let config = default_config();
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(result.success, "simple program should compile: {:?}", result.diagnostics);
        assert!(result.diagnostics.is_empty(), "no diagnostics expected: {:?}", result.diagnostics);
    }

    #[test]
    fn test_compile_with_parse_error() {
        let path = write_temp_file("bad.xi", "fn main() -> Int { return 42; ");
        let config = default_config();
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(!result.success, "parse error should fail compilation");
        assert!(!result.diagnostics.is_empty(), "should have parse errors");
        assert!(result.diagnostics.iter().any(|d| d.message.contains("expected") || d.message.contains("Parse")),
            "error should mention parse issue: {:?}", result.diagnostics);
    }

    #[test]
    fn test_compile_with_type_error() {
        let path = write_temp_file("type_err.xi", "fn main() -> Int { return \"not an int\"; }");
        let config = default_config();
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(!result.success, "type error should fail compilation");
        assert!(!result.diagnostics.is_empty(), "should have type errors");
    }

    #[test]
    fn test_compile_emit_ir() {
        let path = write_temp_file("ir_test.xi", "fn main() -> Int { return 42; }");
        let mut config = default_config();
        config.emit_ir = true;
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(result.success);
        assert!(result.ir.is_some(), "emit_ir should produce IR output");
        let ir = result.ir.unwrap();
        assert!(ir.contains("define i64 @main"), "IR should contain main: {}", &ir[..200.min(ir.len())]);
    }

    #[test]
    fn test_compile_check_only() {
        let path = write_temp_file("check_test.xi", "fn main() -> Int { return 42; }");
        let mut config = default_config();
        config.check_only = true;
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(result.success);
    }

    #[test]
    fn test_compile_diagnostics_json() {
        let path = write_temp_file("diag_test.xi", "fn main() -> Int { return \"oops\"; }");
        let mut config = default_config();
        config.diagnostics_json = true;
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(!result.success);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_compile_empty_file() {
        let path = write_temp_file("empty.xi", "");
        let config = default_config();
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        // Empty file should compile (no code = no errors)
        assert!(result.success || result.diagnostics.is_empty());
    }

    #[test]
    fn test_compile_nonexistent_file() {
        let path = std::path::PathBuf::from("nonexistent_file_12345.xi");
        let config = default_config();
        let result = compile_with_diagnostics(&config, &[path.to_string_lossy().to_string()]);
        assert!(!result.success, "nonexistent file should fail");
        assert!(!result.diagnostics.is_empty());
    }
}
