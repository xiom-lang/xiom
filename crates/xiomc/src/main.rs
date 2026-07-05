// XIOM Programming Language
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
// -----------------------------------------------------------------------

//! XIOM Compiler CLI — Phase 0
//! Usage:
//!   xiomc <source.xi>                         print LLVM IR to stdout
//!   xiomc --emit-ir <source.xi>               print LLVM IR to stdout
//!   xiomc -o <output> <source.xi>             compile to native binary
//!   xiomc --target wasm <source.xi>           compile to WASM
//!   xiomc --target wasm -o out.wasm <src.xi>  compile to WASM with name
//!   xiomc --run <source.xi>                   compile and run, print exit code
//!   xiomc --diagnostics=json <source.xi>      JSON-structured compiler output
//!   xiomc --dump-contracts <source.xi>        emit contract index as JSON

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};
use std::time::Duration;

use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_check::{Checker, BorrowChecker};
use xiom_codegen::IrEmitter;
use xiom_verify::SMTGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(if args.iter().any(|a| a == "--help") { 0 } else { 1 });
    }

    if args.iter().any(|a| a == "--version") {
        println!("XIOM Compiler v0.20.0 \"Hardened\" -- Multi-File + Safety Fixes");
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

    let timeout_secs: u64 = parse_flag_value(&args, "--timeout")
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    // Phase 2.4: Background timeout watchdog
    if timeout_secs > 0 {
        let duration = Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            eprintln!("error: compilation timed out after {} seconds", timeout_secs);
            std::process::exit(1);
        });
    }

    let max_memory_mb: u64 = parse_flag_value(&args, "--max-memory-mb")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // Phase 2.3: Memory budget watchdog
    if max_memory_mb > 0 {
        let max_bytes = max_memory_mb * 1024 * 1024;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(2));
                if let Some(used_bytes) = get_process_memory_bytes() {
                    if used_bytes > max_bytes {
                        eprintln!(
                            "error: memory budget exceeded ({} MB used of {} MB limit). \
                             Try --max-memory-mb with a higher value or simplify the input.",
                            used_bytes / 1024 / 1024,
                            max_memory_mb
                        );
                        std::process::exit(1);
                    }
                }
            }
        });
    }

    // Stage 0: Resolve source files
    let source_paths = resolve_source_files(&args);
    if source_paths.is_empty() {
        eprintln!("error: no source file(s) provided");
        process::exit(1);
    }

    // Stage 1: Lex & Parse
    let mut all_programs: Vec<xiom_ast::Program> = Vec::new();

    for source_path in &source_paths {
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
    let mut program = merge_programs(all_programs);

fn merge_programs(programs: Vec<xiom_ast::Program>) -> xiom_ast::Program {
    let mut items: Vec<xiom_ast::TopDecl> = Vec::new();
    for p in programs {
        for item in p.items {
            match item {
                xiom_ast::TopDecl::Module(md) => {
                    // Merge with existing module of same name
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
    checker.build_catalog_index();
    let is_multi_file = source_paths.len() > 1 || checker.source_dirs.len() > 0;
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
        if !is_multi_file {
            process::exit(1);
        }
        eprintln!("note: {} type errors (continuing to codegen for multi-file compile)", errors.len());
    }

    if dump_contracts {
        let json = dump_contracts_json(&program);
        println!("{json}");
        return;
    }

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

    // Stage 4: Borrow Check
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
                eprintln!("warning[E001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
            }
        }
    }

    // Stage 4.5: Inject external module declarations
    let external_decls = checker.collect_external_decls(&program);
    if !external_decls.is_empty() {
        let existing_names: std::collections::HashSet<String> = program.items.iter().filter_map(|i| match i {
            xiom_ast::TopDecl::Type(td) => Some(td.name.name.clone()),
            xiom_ast::TopDecl::Enum(ed) => Some(ed.name.name.clone()),
            xiom_ast::TopDecl::Fn(fd) => Some(fd.name.name.clone()),
            _ => None,
        }).collect();
        for decl in external_decls {
            let name = match &decl {
                xiom_ast::TopDecl::Type(td) => td.name.name.clone(),
                xiom_ast::TopDecl::Enum(ed) => ed.name.name.clone(),
                xiom_ast::TopDecl::Fn(fd) => fd.name.name.clone(),
                _ => continue,
            };
            if !existing_names.contains(&name) {
                program.items.push(decl);
            }
        }
    }

    // Stage 5: Codegen
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

    if diagnostics_json {
        println!(r#"{{"status":"ok"}}"#);
        return;
    }

    if emit_ir || (output_file.is_none() && !do_run && target == Target::Native) {
        println!("{llvm_ir}");
        return;
    }

    // Stage 6: Compile to binary via clang
    let default_output = match target {
        Target::Wasm => "a.wasm",
        Target::Arm | Target::RisCv => "a.out",
        Target::Native => "a.exe",
    };
    let output = output_file.as_deref().unwrap_or(default_output);

    let ir_path = format!("{output}.ll");
    if let Err(e) = fs::write(&ir_path, &llvm_ir) {
        eprintln!("error: cannot write IR file: {e}");
        process::exit(1);
    }

    // Phase 1.4: Run LLVM opt -O1 to optimize IR before clang
    let opt = find_tool("opt", &[
        "C:\\Program Files\\LLVM\\bin\\opt.exe",
    ]);
    if let Some(opt_path) = &opt {
        let opt_status = Command::new(opt_path)
            .args(["-O1", "-S", "-o", &ir_path, &ir_path])
            .status();
        if let Ok(s) = opt_status {
            if !s.success() {
                eprintln!("  warning: opt -O1 failed, proceeding with unoptimized IR");
                let _ = fs::write(&ir_path, &llvm_ir);
            }
        }
    }

    // Phase 2.5: NASM assembly of runtime .asm files
    // Disabled by default — .asm files may have version-specific syntax.
    // Enable by building with: cargo build --features nasm
    // Or manually assemble per stdlib/runtime/BUILD.md
    let asm_objects: Vec<String> = Vec::new();
    #[cfg(feature = "nasm")]
    {
        let runtime_dir = find_runtime_c().and_then(|p| {
            std::path::Path::new(&p).parent().map(|d| d.to_path_buf())
        });
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
                if asm_path.exists() {
                    let obj_path = rt_dir.join(format!("{}.{}", asm_file, obj_ext));
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
            // Enable AES-NI intrinsics for crypto acceleration in xiom_runtime.c
            if target == Target::Native { cmd.arg("-maes"); }
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
                Target::Native => {
                    if cfg!(target_os = "windows") {
                        cmd.args(["-Xlinker", "/SUBSYSTEM:CONSOLE"]);
                    }
                }
            }
            if target != Target::Wasm {
                if let Some(rt) = find_runtime_c() {
                    cmd.arg(&rt);
                }
            }
            cmd.args(["-o", output, &ir_path]);
            // Link assembled .obj/.o files for hardware acceleration
            for obj in &asm_objects { cmd.arg(obj); }

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

    if sources.len() == 1 {
        if let Ok(metadata) = fs::metadata(&sources[0]) {
            if metadata.is_dir() {
                return load_package_dir(&sources[0]);
            }
        }
    }

    sources
}

/// Load all `.xi` files from a package directory.
fn load_package_dir(dir: &str) -> Vec<String> {
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

fn scan_xi_files(dir: &str) -> Vec<String> {
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

fn extract_module_path(source: &str) -> Option<String> {
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

fn print_usage() {
    eprintln!("XIOM Compiler v0.11.0 \"Self-Hosted\" -- Full Self-Hosting");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiomc [OPTIONS] <source.xi>");
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
    eprintln!("  --timeout <seconds>  Set compilation timeout (default: 60)");
    eprintln!("  --max-memory-mb <N>       Set max memory budget in MB (0 = disabled)");
    eprintln!();
    eprintln!("DEPENDENCIES:");
    eprintln!("  Required: clang (LLVM) — to compile IR to native binary");
    eprintln!("  Optional: opt (LLVM) — IR optimization pass (-O1)");
    eprintln!("  Optional: nasm — hardware-accelerated crypto/memcpy (stdlib)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiomc --run examples/demo_float.xi");
    eprintln!("  xiomc -o prog.exe source.xi");
    eprintln!("  xiomc --emit-ir examples/demo_float.xi");
    eprintln!("  xiomc --target wasm -o prog.wasm source.xi");
    eprintln!("  xiomc --verify examples/phase1_contracts.xi");
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

fn get_process_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        // Use PowerShell to query working set (avoids FFI linking issues)
        let pid = std::process::id();
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).WorkingSet64")])
            .output()
        {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = s.trim().parse::<u64>() {
                        return Some(bytes);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Linux/macOS: read /proc/self/status for VmRSS
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return Some(kb * 1024);
                }
            }
        }
        None
    }
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

#[allow(dead_code)]
fn find_nasm() -> Option<String> {
    // Check common install locations
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

#[allow(dead_code)]
fn is_newer(src: &std::path::Path, dst: &std::path::Path) -> bool {
    if let (Ok(sm), Ok(dm)) = (src.metadata(), dst.metadata()) {
        if let (Ok(st), Ok(dt)) = (sm.modified(), dm.modified()) {
            return st > dt;
        }
    }
    true // Rebuild if we can't determine timestamps
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
                UnaryOp::BitNot => "~",
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
