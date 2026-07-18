// XIOM Compiler Library
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
// -----------------------------------------------------------------------

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_check::{Checker, BorrowChecker};
use xiom_codegen::IrEmitter;
use xiom_verify::SMTGenerator;

#[derive(PartialEq, Clone, Copy)]
pub enum Target {
    Native,
    Wasm,
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
    pub max_recursion_depth: u32,
    pub dump_contracts: bool,
    pub verify: bool,
    pub verify_output: Option<String>,
    pub output_file: Option<String>,
    pub link_libs: Vec<String>,
    pub link_paths: Vec<String>,
    pub c_sources: Vec<String>,
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
            max_recursion_depth: 500,
            dump_contracts: false,
            verify: false,
            verify_output: None,
            output_file: None,
            link_libs: Vec::new(),
            link_paths: Vec::new(),
            c_sources: Vec::new(),
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

    // Stage 1: Lex & Parse
    let mut all_programs: Vec<Program> = Vec::new();
    for source_path in source_paths {
        let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "package.xi" { continue; }

        let source = match fs::read_to_string(source_path) {
            Ok(s) => s,
            Err(e) => {
                result.diagnostics.push(Diagnostic {
                    kind: "io_error".into(), code: "I001".into(),
                    message: format!("cannot read '{}': {}", source_path, e),
                    line: 0, col: 0, file: source_path.clone(),
                    suggestion: None, help: None, note: None,
                });
                return result;
            }
        };
        result.file_count += 1;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();

        // Collect lex errors
        let mut lex_errors = false;
        for tok in &tokens {
            if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
                result.diagnostics.push(Diagnostic {
                    kind: "lex_error".into(), code: "L001".into(),
                    message: msg.clone(),
                    line: tok.span.line, col: tok.span.col, file: source_path.clone(),
                    suggestion: None, help: None, note: None,
                });
                lex_errors = true;
            }
        }
        if lex_errors { continue; }

        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(p) => {
                // Surface RECOVERED parse errors: parse_program returns Ok with
                // a partial AST after panic-mode recovery. Silently accepting
                // it drops declarations (e.g. a trailing-comma fn vanished
                // with no diagnostic). Report every recovered error.
                for e in parser.errors() {
                    let (help, note) = diagnostic_for(&e.message);
                    result.diagnostics.push(Diagnostic {
                        kind: "parse_error".into(), code: "P001".into(),
                        message: e.message.clone(),
                        line: e.span.line, col: e.span.col, file: source_path.clone(),
                        suggestion: Some(suggest_fix(&e.message)),
                        help, note,
                    });
                }
                all_programs.push(p);
            }
            Err(e) => {
                let (help, note) = diagnostic_for(&e.message);
                let suggestion = suggest_fix(&e.message);
                result.diagnostics.push(Diagnostic {
                    kind: "parse_error".into(), code: "P001".into(),
                    message: e.message,
                    line: e.span.line, col: e.span.col, file: source_path.clone(),
                    suggestion: Some(suggestion),
                    help, note,
                });
            }
        }
    }

    if result.diagnostics.iter().any(|d| d.kind == "parse_error" || d.kind == "lex_error") {
        return result;
    }

    if all_programs.is_empty() {
        return result;
    }

    // Merge programs
    let program = merge_programs(all_programs);

    // Stage 3: Type Check
    let mut checker = Checker::new();
    if let Some(primary) = source_paths.first() {
        if let Some(parent) = Path::new(primary).parent() {
            checker.add_source_dir(parent.to_string_lossy().to_string());
        }
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
        Target::Arm => emitter.set_target_triple("aarch64-unknown-linux-gnu"),
        Target::RisCv => emitter.set_target_triple("riscv64-unknown-linux-gnu"),
        Target::Native => {}
    }
    emitter.set_check_contracts(config.check_contracts);
    emitter.set_max_recursion_depth(config.max_recursion_depth);
    emitter.set_strict_mode(config.strict_mode);

    match emitter.compile_program(&program) {
        Ok(ir) => {
            result.ir = Some(ir.clone());
            result.success = true;

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

pub fn compile(config: &CompileConfig, source_paths: &[String]) {
    // Stage 1: Lex & Parse
    let mut all_programs: Vec<xiom_ast::Program> = Vec::new();

    for source_path in source_paths {
        let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "package.xi" { continue; }

        let source = fs::read_to_string(source_path)
            .map_err(|e| format!("cannot read '{source_path}': {e}"))
            .unwrap_or_else(|e| { eprintln!("error: {e}"); process::exit(1); });

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
            process::exit(1);
        }

        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(p) => {
                // Surface RECOVERED parse errors (panic-mode recovery returns
                // Ok with a partial AST). Without this, a malformed declaration
                // is silently DROPPED — e.g. a trailing-comma fn disappeared
                // with no diagnostic and callers got 'undefined variable'.
                if !parser.errors().is_empty() {
                    for e in parser.errors() {
                        let (help, note) = diagnostic_for(&e.message);
                        render_error("P001", &e.span, &e.message, Some(&source), help.as_deref(), note.as_deref());
                    }
                    process::exit(1);
                }
                all_programs.push(p);
            }
            Err(e) => {
                let (help, note) = diagnostic_for(&e.message);
                render_error("P001", &e.span, &e.message, Some(&source), help.as_deref(), note.as_deref());
                process::exit(1);
            }
        }
    }

    // Merge all parsed programs into one
    let mut program = merge_programs(all_programs);

    // Stage 3: Type Check
    let mut checker = Checker::new();
    if let Some(primary) = source_paths.first() {
        if let Some(parent) = Path::new(primary).parent() {
            checker.add_source_dir(parent.to_string_lossy().to_string());
        }
    }
    let examples_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("examples");
    if examples_root.is_dir() {
        checker.add_source_dir(examples_root.to_string_lossy().to_string());
    }
    for stdlib_dir in find_stdlib_dirs() {
        checker.add_source_dir(stdlib_dir);
    }
    checker.build_catalog_index();
    let is_multi_file = source_paths.len() > 1 || checker.source_dirs.len() > 0;
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
            process::exit(1);
        }
        eprintln!("note: {} type errors — aborting codegen", errors.len());
        process::exit(1);
    }

    if config.dump_contracts {
        let json = dump_contracts_json(&program);
        println!("{json}");
        return;
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
            eprintln!("Z3 found — use 'z3 file.smt2' to verify");
        }
        return;
    }

    let primary_source = source_paths.first().map(|s| s.as_str()).unwrap_or("<unknown>");

    if config.check_only {
        if config.diagnostics_json {
            println!(r#"{{"status":"check_passed","type_errors":0,"borrow_warnings":0,"time_ms":0}}"#);
        } else {
            eprintln!("  Type check PASSED (no errors)");
        }
        return;
    }

    // Stage 4: Borrow Check
    let mut borrow_checker = BorrowChecker::new();
    if let Err(errors) = borrow_checker.check_program(&program) {
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
                eprintln!("warning[E001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
            }
        }
    }

    // Stage 4.5: Inject external module declarations
    let external_decls = checker.collect_external_decls(&program);
    if !external_decls.is_empty() {
        fn fn_dedup_key(fd: &xiom_ast::FnDecl) -> String {
            if fd.is_method() {
                format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
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
    emitter.set_check_contracts(config.check_contracts);
    emitter.set_max_recursion_depth(config.max_recursion_depth);
    emitter.set_strict_mode(config.strict_mode);
    emitter.set_target_triple(match config.target {
        Target::Wasm => "wasm32-unknown-unknown",
        Target::Arm => "aarch64-unknown-linux-gnu",
        Target::RisCv => "riscv64gc-unknown-linux-gnu",
        Target::Native => "x86_64-pc-windows-msvc",
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
            process::exit(1);
        }
    };

    if config.diagnostics_json {
        println!(r#"{{"status":"ok"}}"#);
        return;
    }

    if config.emit_ir || (config.output_file.is_none() && !config.do_run && config.target == Target::Native) {
        println!("{llvm_ir}");
        return;
    }

    // Stage 6: Compile to binary via clang
    let default_output = match config.target {
        Target::Wasm => "a.wasm",
        Target::Arm | Target::RisCv => "a.out",
        Target::Native => "a.exe",
    };
    let output = config.output_file.as_deref().unwrap_or(default_output);

    let ir_path = format!("{output}.ll");
    if let Err(e) = fs::write(&ir_path, &llvm_ir) {
        eprintln!("error: cannot write IR file: {e}");
        process::exit(1);
    }

    let opt = find_tool("opt", &[
        "C:\\Program Files\\LLVM\\bin\\opt.exe",
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
        let opt_level = if config.release { "-O3" } else { "-O1" };
        let opt_status = Command::new(opt_path)
            .args([opt_level, "-S", "-o", &ir_path, &ir_path])
            .status();
        if let Ok(s) = opt_status {
            if !s.success() {
                eprintln!("  warning: opt -O1 failed, proceeding with unoptimized IR");
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
    ]);

    match clang {
        Some(clang_path) => {
            let mut cmd = Command::new(&clang_path);
            if config.target == Target::Native { cmd.arg("-maes"); }
            if asm_objects.is_empty() { cmd.arg("-DXIOM_NO_ASM"); }
            if config.debug_symbols { cmd.arg("-g"); }
            if config.shared_lib { cmd.arg("-shared"); }
            if config.static_lib { cmd.arg("-c"); }
            match config.target {
                Target::Wasm => {
                    cmd.args(["--target=wasm32-unknown-unknown", "-nostdlib", "-Wl,--no-entry", "-Wl,--export-all"]);
                }
                Target::Arm => {
                    cmd.args(["--target=aarch64-unknown-linux-gnu"]);
                }
                Target::RisCv => {
                    cmd.args(["--target=riscv64gc-unknown-linux-gnu"]);
                }
                Target::Native => {
                    if cfg!(target_os = "windows") {
                        cmd.args(["-Xlinker", "/SUBSYSTEM:CONSOLE", "-Xlinker", "/STACK:2097152,2097152", "-Xlinker", "/Brepro"]);
                    }
                }
            }
            if config.target != Target::Wasm {
                let runtime_c_files = find_runtime_c_files();
                if runtime_c_files.is_empty() {
                    if let Some(rt) = find_runtime_c() {
                        cmd.arg(&rt);
                    }
                } else {
                    for rt in &runtime_c_files {
                        let abs_rt = if std::path::Path::new(rt).is_absolute() {
                            rt.clone()
                        } else {
                            std::env::current_dir().unwrap_or_default().join(rt).to_string_lossy().to_string()
                        };
                        cmd.arg(abs_rt);
                    }
                }
                for cs in &config.c_sources {
                    cmd.arg(cs);
                }
            }
            let cwd0 = std::env::current_dir().unwrap_or_default();
            let abs_output = if std::path::Path::new(output).is_absolute() { output.to_string() } else { cwd0.join(output).to_string_lossy().to_string() };

            let unique_tmp = std::env::temp_dir().join(format!(
                "xiomc_link_{}_{}",
                std::process::id(),
                output.replace(['\\', '/', ':', '.'], "_")
            ));
            let _ = std::fs::create_dir_all(&unique_tmp);
            const STAGED_IR_NAME: &str = "xiomc_input.ll";
            if let Err(e) = fs::copy(&ir_path, unique_tmp.join(STAGED_IR_NAME)) {
                eprintln!("error: cannot stage IR file into temp dir: {e}");
                process::exit(1);
            }
            cmd.args(["-o", &abs_output, STAGED_IR_NAME]);
            if config.target == Target::Native {
                for obj in &asm_objects { cmd.arg(obj); }
            }
            if config.target != Target::Wasm {
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

                    if config.do_run && config.target == Target::Native {
                        let exe = if output.contains('\\') || output.contains('/') {
                            output.to_string()
                        } else {
                            format!(".\\{output}")
                        };
                        let run_status = Command::new(&exe).status();
                        match run_status {
                            Ok(s) => eprintln!("  exit code: {}", s.code().unwrap_or(-1)),
                            Err(e) => {
                                eprintln!("error: cannot run '{exe}': {e}");
                                process::exit(1);
                            }
                        }
                    }

                    if config.target == Target::Wasm {
                        if let Ok(meta) = fs::metadata(output) {
                            eprintln!("  wasm size: {} bytes", meta.len());
                        }
                    }
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("error: clang failed with exit code {}", out.status.code().unwrap_or(-1));
                    if stderr.contains("stdio.h") || stderr.contains("fatal error") {
                        eprintln!("  -> Missing C standard library headers (stdio.h).");
                        eprintln!("  -> Install Visual Studio 2022 Build Tools with 'Desktop development with C++':");
                        eprintln!("      winget install Microsoft.VisualStudio.2022.BuildTools");
                        eprintln!("    Or run: .\\install_deps.ps1");
                    }
                    eprintln!("  stderr: {}", stderr.trim());
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: cannot run clang: {e}");
                    eprintln!("note: LLVM IR written to {ir_path}");
                    process::exit(1);
                }
            }
        }
        None => {
            eprintln!("note: clang not found — LLVM IR written to {ir_path}");
            match config.target {
                Target::Wasm => {
                    eprintln!("  compile manually: clang --target=wasm32-unknown-unknown -nostdlib -Wl,--no-entry -Wl,--export-all -o {output} {ir_path}");
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
            process::exit(1);
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

pub fn resolve_source_files(args: &[String]) -> Vec<String> {
    let mut sources = Vec::new();
    let mut skip_next = false;

    for arg in args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "-o" | "--target" | "--verify-output" | "--link" | "--link-path" | "--c-source") {
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
                &trimmed[bracket_start.unwrap()..]
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

pub fn find_runtime_c() -> Option<String> {
    let candidates: Vec<String> = {
        let mut paths = vec![
            "stdlib\\runtime\\xiom_runtime.c".to_string(),
            "stdlib/runtime/xiom_runtime.c".to_string(),
        ];
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                if let Some(parent) = exe_dir.parent() {
                    paths.push(format!("{}/runtime/xiom_runtime.c", parent.display()));
                    paths.push(format!("{}\\runtime\\xiom_runtime.c", parent.display()));
                }
            }
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

pub fn find_runtime_c_files() -> Vec<String> {
    let mut dir_candidates: Vec<String> = vec![
        "stdlib\\runtime".to_string(),
        "stdlib/runtime".to_string(),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if let Some(parent) = exe_dir.parent() {
                dir_candidates.push(format!("{}/runtime", parent.display()));
                dir_candidates.push(format!("{}\\runtime", parent.display()));
            }
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
            println!("──");
            println!("For the full error-code index: docs/error_codes/README.md");
        }
        Err(_) => {
            eprintln!("Unknown error code: {code}");
            eprintln!("Available codes are listed in docs/error_codes/README.md");
            eprintln!("Run: xiomc --explain X0010  (for type mismatch)");
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
        Expr::Call(func, args, _) => {
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

/// Phase 5d: Public API — dump all contract signatures as JSON.
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
