// AXIOM Programming Language
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
// -----------------------------------------------------------------------

//! AXIOM Compiler CLI — Phase 0
//! Usage:
//!   axiomc <source.ax>                         print LLVM IR to stdout
//!   axiomc --emit-ir <source.ax>               print LLVM IR to stdout
//!   axiomc -o <output> <source.ax>             compile to native binary
//!   axiomc --target wasm <source.ax>           compile to WASM
//!   axiomc --target wasm -o out.wasm <src.ax>  compile to WASM with name
//!   axiomc --run <source.ax>                   compile and run, print exit code
//!   axiomc --diagnostics=json <source.ax>      JSON-structured compiler output
//!   axiomc --dump-contracts <source.ax>        emit contract index as JSON

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use axiom_ast::*;
use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_check::{Checker, BorrowChecker};
use axiom_codegen::IrEmitter;
use axiom_verify::SMTGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(if args.iter().any(|a| a == "--help") { 0 } else { 1 });
    }

    if args.iter().any(|a| a == "--version") {
        println!("AXIOM Compiler v0.20.0 \"Hardened\" -- Multi-File + Safety Fixes");
        return;
    }

    let emit_ir = args.iter().any(|a| a == "--emit-ir");
    let do_run = args.iter().any(|a| a == "--run");
    let target = parse_target(&args);
    let check_contracts = !args.iter().any(|a| a == "--no-contracts");
    let _explicit_contracts = args.iter().any(|a| a == "--check-contracts");
    let diagnostics_json = args.iter().any(|a| a == "--diagnostics=json");
    let dump_contracts = args.iter().any(|a| a == "--dump-contracts");
    let verify = args.iter().any(|a| a == "--verify") || args.iter().any(|a| a == "--verify-output");
    let verify_output = parse_flag_value(&args, "--verify-output");

    let output_file = parse_flag_value(&args, "-o");

    // ── Stage 0: Resolve source files ─────────────────────
    let source_paths = resolve_source_files(&args);
    if source_paths.is_empty() {
        eprintln!("error: no source file(s) provided");
        process::exit(1);
    }

    // ── Stage 1: Lex & Parse ──────────────────────────────
    let mut all_programs: Vec<axiom_ast::Program> = Vec::new();
    let mut source_label = "<unknown>".to_string();

    for source_path in &source_paths {
        let file_name = Path::new(source_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "package.ax" { continue; }
        
        let source = fs::read_to_string(source_path)
            .map_err(|e| format!("cannot read '{source_path}': {e}"))
            .unwrap_or_else(|e| { eprintln!("error: {e}"); process::exit(1); });
        
        source_label = source_path.clone();
        
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        
        let lex_errors: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.kind, axiom_lexer::TokenKind::Error(_)))
            .collect();
        if !lex_errors.is_empty() {
            for tok in &lex_errors {
                if let axiom_lexer::TokenKind::Error(msg) = &tok.kind {
                    eprintln!("error[L001]: {msg} at {l}:{c}", l = tok.span.line, c = tok.span.col);
                }
            }
            process::exit(1);
        }
        
        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(p) => all_programs.push(p),
            Err(e) => {
                eprintln!("error[P001]: {l}:{c}: {m}", l = e.span.line, c = e.span.col, m = e.message);
                process::exit(1);
            }
        }
    }
    
    // Merge all parsed programs into one
    let program = merge_programs(all_programs);

fn merge_programs(programs: Vec<axiom_ast::Program>) -> axiom_ast::Program {
    let mut items: Vec<axiom_ast::TopDecl> = Vec::new();
    for p in programs {
        for item in p.items {
            match item {
                axiom_ast::TopDecl::Module(md) => {
                    // Merge with existing module of same name
                    let md_name = md.name.name.clone();
                    if let Some(existing) = items.iter_mut().find_map(|i| {
                        if let axiom_ast::TopDecl::Module(emd) = i {
                            if emd.name.name == md_name { Some(emd) } else { None }
                        } else { None }
                    }) {
                        existing.items.extend(md.items);
                    } else {
                        items.push(axiom_ast::TopDecl::Module(md));
                    }
                }
                other => items.push(other),
            }
        }
    }
    axiom_ast::Program::new(items, axiom_ast::Span::new(0, 0))
}

    // ── Stage 3: Type Check ───────────────────────────────
    let mut checker = Checker::new();
    if let Err(errors) = checker.check_program(&program) {
        if diagnostics_json {
            let parts: Vec<String> = errors.iter().map(|err| {
                format!(
                    r#"{{"kind":"type_error","code":"T001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                    escape_json(&err.message), escape_json("<unknown>"), err.span.line, err.span.col
                )
            }).collect();
            println!("[{}]", parts.join(","));
        } else {
            for err in &errors {
                eprintln!("error[T001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
            }
        }
        process::exit(1);
    }

    // ── Stage 3.5: Dump Contracts (if requested) ──────────
    if dump_contracts {
        let json = dump_contracts_json(&program);
        println!("{json}");
        return;
    }

    // ── Stage 3.6: SMT Verification (if requested) ────────
    if verify {
        let mut generator = SMTGenerator::new();
        let smt = generator.generate(&program);
        if let Some(path) = &verify_output {
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

    // ── Stage 4: Borrow Check ─────────────────────────────
    let mut borrow_checker = BorrowChecker::new();
    if let Err(errors) = borrow_checker.check_program(&program) {
        if diagnostics_json {
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
        process::exit(1);
    }

    // ── Stage 5: Codegen ──────────────────────────────────
    let mut emitter = IrEmitter::new();
    emitter.set_check_contracts(check_contracts);
    emitter.set_target_triple(match target {
        Target::Wasm => "wasm32-unknown-unknown",
        Target::Arm => "aarch64-unknown-linux-gnu",
        Target::RisCv => "riscv64gc-unknown-linux-gnu",
        Target::Native => "x86_64-pc-windows-msvc",
    });
    let llvm_ir = match emitter.compile_program(&program) {
        Ok(ir) => ir,
        Err(e) => {
            if diagnostics_json {
                println!(r#"{{"kind":"codegen_error","code":"C001","message":"{}","location":{{"file":"{}","line":0,"col":0}}}}"#,
                    escape_json(&e), escape_json(primary_source));
            } else {
                eprintln!("error[C001]: codegen: {e}");
            }
            process::exit(1);
        }
    };

    // If diagnostics=json, output success JSON instead of compiling further
    if diagnostics_json {
        println!(r#"{{"status":"ok"}}"#);
        return;
    }

    // Just emit IR?
    if emit_ir || (output_file.is_none() && !do_run && target == Target::Native) {
        println!("{llvm_ir}");
        return;
    }

    // ── Stage 6: Compile to binary via clang ──────────────
    let default_output = match target {
        Target::Wasm => "a.wasm",
        Target::Arm | Target::RisCv => "a.out",
        Target::Native => "a.exe",
    };
    let output = output_file.as_deref().unwrap_or(default_output);

    // Write IR to temp .ll file
    let ir_path = format!("{output}.ll");
    if let Err(e) = fs::write(&ir_path, &llvm_ir) {
        eprintln!("error: cannot write IR file: {e}");
        process::exit(1);
    }

    let clang = find_tool("clang", &[
        "C:\\Program Files\\LLVM\\bin\\clang.exe",
    ]);

    match clang {
        Some(clang_path) => {
            let mut cmd = Command::new(&clang_path);
            match target {
                Target::Wasm => {
                    cmd.args(["--target=wasm32-unknown-unknown", "-nostdlib", "-Wl,--no-entry", "-Wl,--export-all"]);
                }
                Target::Arm => {
                    cmd.args(["--target=aarch64-unknown-linux-gnu"]);
                }
                Target::RisCv => {
                    cmd.args(["--target=riscv64gc-unknown-linux-gnu"]);
                }
                Target::Native => {}
            }
            // Include runtime C library for non-WASM targets (resolves extern functions)
            if target != Target::Wasm {
                if let Some(rt) = find_runtime_c() {
                    cmd.arg(&rt);
                }
            }
            cmd.args(["-o", output, &ir_path]);

            let clang_output = cmd.output();
            match clang_output {
                Ok(out) if out.status.success() => {
                    let _ = fs::remove_file(&ir_path);
                    eprintln!("  compiled: {output}");

                    if do_run && target == Target::Native {
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

                    if target == Target::Wasm {
                        if let Ok(meta) = fs::metadata(output) {
                            eprintln!("  wasm size: {} bytes", meta.len());
                        }
                    }
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("error: clang failed with exit code {}", out.status.code().unwrap_or(-1));
                    // Detect common failures and give actionable advice
                    if stderr.contains("stdio.h") || stderr.contains("fatal error") {
                        eprintln!("  → Missing C standard library headers (stdio.h).");
                        eprintln!("  → Install Visual Studio 2022 Build Tools with 'Desktop development with C++':");
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
            match target {
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

/// Resolve source files from CLI arguments.
/// - If argument is a directory, load all `.ax` files (excluding `package.ax`)
/// - If multiple `.ax` file arguments, return them all
/// - Single file is returned as-is
fn resolve_source_files(args: &[String]) -> Vec<String> {
    let mut sources = Vec::new();
    let mut skip_next = false;

    for arg in args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "-o" | "--target" | "--verify-output") {
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

    // If a single directory is given, load the package
    if sources.len() == 1 {
        if let Ok(metadata) = fs::metadata(&sources[0]) {
            if metadata.is_dir() {
                return load_package_dir(&sources[0]);
            }
        }
    }

    sources
}

/// Load all `.ax` files from a package directory.
/// Reads `package.ax` if present for the module list, otherwise scans the directory.
fn load_package_dir(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let package_path = format!("{}/package.ax", dir);

    // Try package.ax manifest first
    if fs::metadata(&package_path).is_ok() {
        if let Ok(modules) = parse_package_manifest(&package_path) {
            // Resolve each module to a file by scanning the directory
            let ax_files = scan_ax_files(dir);
            let module_file_map = build_module_file_map(&ax_files);
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

    // Fallback: scan all .ax files in directory
    files = scan_ax_files(dir);
    files
}

/// Scan a directory for `.ax` files (excluding `package.ax`).
fn scan_ax_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ax") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name != "package.ax" {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    files
}

/// Build a map from module path (from `module` declaration) to file path.
/// Reads each file's first lines to find the `module` declaration.
fn build_module_file_map(files: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
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

/// Extract the module path from the first `module` declaration found in source.
fn extract_module_path(source: &str) -> Option<String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut i = 0;
    // Skip leading comments/whitespace (lexer already removed them)
    while i < tokens.len() {
        if tokens[i].is_eof() {
            return None;
        }
        if let axiom_lexer::TokenKind::Module = &tokens[i].kind {
            i += 1;
            // Parse the module path
            let mut path_parts = Vec::new();
            if i < tokens.len() {
                if let axiom_lexer::TokenKind::Ident(name) = &tokens[i].kind {
                    path_parts.push(name.clone());
                    i += 1;
                }
                while i < tokens.len() {
                    if let axiom_lexer::TokenKind::Dot = &tokens[i].kind {
                        i += 1;
                        if i < tokens.len() {
                            if let axiom_lexer::TokenKind::Ident(name) = &tokens[i].kind {
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

/// Simple parser for `package.ax` manifest format.
/// Extracts the `modules: [...]` array.
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

/// Build a concatenated source string from multiple `.ax` files,
/// wrapping each file's content inside nested module blocks derived
/// from its `module` declaration.
fn build_concatenated_source(files: &[String]) -> Result<String, String> {
    let mut result = String::from("module benchmark {\n");

    for file_path in files {
        let file_name = Path::new(file_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "package.ax" { continue; }

        let source = fs::read_to_string(file_path).map_err(|e| format!("cannot read '{file_path}': {e}"))?;
        let module_path = extract_module_path(&source);

        if let Some(_path) = module_path {
            let body = strip_module_decl(&source);

            // Emit body with single-level indent (inside `module benchmark {`)
            for line in body.lines() {
                result.push_str("  ");
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    result.push_str("}\n");
    Ok(result)
}

/// Strip the module declaration line and any leading content (e.g. copyright header)
/// from a source file, returning only the code body after the `module` line.
fn strip_module_decl(source: &str) -> String {
    let mut result = String::new();
    let mut found_module = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !found_module {
            // Skip blank lines and comment lines before the module declaration
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }
            if trimmed.starts_with("module ") {
                found_module = true;
                continue;
            }
            // If we hit non-comment, non-module content before module declaration, include it
            found_module = true;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

fn print_usage() {
    eprintln!("AXIOM Compiler v0.11.0 \"Self-Hosted\" -- Full Self-Hosting");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  axiomc [OPTIONS] <source.ax>");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help              Show this help message");
    eprintln!("  --version           Print version");
    eprintln!("  -o <output>         Output binary path (default: a.exe)");
    eprintln!("  --run               Compile and run, print exit code");
    eprintln!("  --emit-ir           Print LLVM IR to stdout (no compilation)");
    eprintln!("  --target <target>   Target: native (default), wasm, arm, riscv");
    eprintln!("  --no-contracts      Disable contract runtime checks");
    eprintln!("  --diagnostics=json  Output diagnostics as JSON");
    eprintln!("  --dump-contracts    Print contract index as JSON");
    eprintln!("  --verify            Generate SMT-LIB contract verification output");
    eprintln!("  --verify-output <f> Write SMT-LIB to file");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  axiomc --run examples/demo_float.ax");
    eprintln!("  axiomc -o prog.exe source.ax");
    eprintln!("  axiomc --emit-ir examples/demo_float.ax");
    eprintln!("  axiomc --target wasm -o prog.wasm source.ax");
    eprintln!("  axiomc --verify examples/phase1_contracts.ax");
}

#[derive(PartialEq)]
enum Target {
    Native,
    Wasm,
    Arm,
    RisCv,
}

fn parse_target(args: &[String]) -> Target {
    match parse_flag_value(args, "--target").as_deref() {
        Some("wasm") => Target::Wasm,
        Some("arm") => Target::Arm,
        Some("riscv") => Target::RisCv,
        _ => Target::Native,
    }
}

fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    let pos = args.iter().position(|a| a == flag)?;
    args.get(pos + 1).cloned()
}

fn find_runtime_c() -> Option<String> {
    // Search paths — ordered by priority:
    // 1. Relative to CWD (development: running from repo root)
    // 2. Relative to the axiomc.exe binary (installed: %LOCALAPPDATA%\axiom\bin\)
    // 3. Absolute Windows SDK paths
    let candidates: Vec<String> = {
        let mut paths = vec![
            "stdlib\\runtime\\axiom_runtime.c".to_string(),
            "stdlib/runtime/axiom_runtime.c".to_string(),
        ];
        // Installed path: bin/../runtime/axiom_runtime.c
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                // Look in ../runtime/ relative to bin/
                if let Some(parent) = exe_dir.parent() {
                    paths.push(format!("{}/runtime/axiom_runtime.c", parent.display()));
                    paths.push(format!("{}\\runtime\\axiom_runtime.c", parent.display()));
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

fn find_tool(name: &str, extra_paths: &[&str]) -> Option<String> {
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

fn escape_json(s: &str) -> String {
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
        Expr::Struct(ident, fields, _) => {
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

fn dump_contracts_json(program: &Program) -> String {
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
                // Recursively collect contracts from modules
                items.extend(dump_module_contracts(md));
            }
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
            _ => {}
        }
    }

    items
}
