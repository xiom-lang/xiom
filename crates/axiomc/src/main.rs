//! AXIOM Compiler CLI — Phase 0
//! Usage:
//!   axiomc <source.ax>                         print LLVM IR to stdout
//!   axiomc --emit-ir <source.ax>               print LLVM IR to stdout
//!   axiomc -o <output> <source.ax>             compile to native binary
//!   axiomc --target wasm <source.ax>           compile to WASM
//!   axiomc --target wasm -o out.wasm <src.ax>  compile to WASM with name
//!   axiomc --run <source.ax>                   compile and run, print exit code

use std::env;
use std::fs;
use std::process::{self, Command};

use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_check::{Checker, BorrowChecker};
use axiom_codegen::IrEmitter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let emit_ir = args.iter().any(|a| a == "--emit-ir");
    let do_run = args.iter().any(|a| a == "--run");
    let target_wasm = parse_flag_value(&args, "--target")
        .map(|v| v == "wasm")
        .unwrap_or(false);
    let check_contracts = !args.iter().any(|a| a == "--no-contracts");
    let _explicit_contracts = args.iter().any(|a| a == "--check-contracts");

    let output_file = parse_flag_value(&args, "-o");

    // Find the source file — last arg that doesn't start with -
    let source_path = args.iter().rev()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            // Also skip flag values like "wasm"
            eprintln!("error: no source file provided");
            process::exit(1);
        });

    if source_path.starts_with('-') || source_path == "wasm" {
        eprintln!("error: no source file provided");
        process::exit(1);
    }

    // ── Stage 1: Lex ──────────────────────────────────────
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{source_path}': {e}");
            process::exit(1);
        }
    };

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

    // ── Stage 2: Parse ────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error[P001]: {l}:{c}: {m}", l = e.span.line, c = e.span.col, m = e.message);
            process::exit(1);
        }
    };

    // ── Stage 3: Type Check ───────────────────────────────
    let mut checker = Checker::new();
    if let Err(errors) = checker.check_program(&program) {
        for err in &errors {
            eprintln!("error[T001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
        }
        process::exit(1);
    }

    // ── Stage 4: Borrow Check ─────────────────────────────
    let mut borrow_checker = BorrowChecker::new();
    if let Err(errors) = borrow_checker.check_program(&program) {
        for err in &errors {
            eprintln!("error[E001]: {l}:{c}: {m}", l = err.span.line, c = err.span.col, m = err.message);
        }
        process::exit(1);
    }

    // ── Stage 5: Codegen ──────────────────────────────────
    let mut emitter = IrEmitter::new();
    emitter.set_check_contracts(check_contracts);
    let llvm_ir = match emitter.compile_program(&program) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("error[C001]: codegen: {e}");
            process::exit(1);
        }
    };

    // Just emit IR?
    if emit_ir || (output_file.is_none() && !do_run && !target_wasm) {
        println!("{llvm_ir}");
        return;
    }

    // ── Stage 6: Compile to binary via clang ──────────────
    let default_output = if target_wasm { "a.wasm" } else { "a.exe" };
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
            if target_wasm {
                cmd.args(["--target=wasm32-unknown-unknown", "-nostdlib", "-Wl,--no-entry", "-Wl,--export-all"]);
            }
            cmd.args(["-o", output, &ir_path]);

            let status = cmd.status();
            match status {
                Ok(s) if s.success() => {
                    let _ = fs::remove_file(&ir_path);
                    eprintln!("  compiled: {output}");

                    if do_run && !target_wasm {
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

                    if target_wasm {
                        // Verify WASM by checking file size
                        if let Ok(meta) = fs::metadata(output) {
                            eprintln!("  wasm size: {} bytes", meta.len());
                        }
                    }
                }
                Ok(s) => {
                    eprintln!("error: clang failed with exit code {}", s.code().unwrap_or(-1));
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
            if target_wasm {
                eprintln!("  compile manually: clang --target=wasm32 -nostdlib -Wl,--no-entry -Wl,--export-all -o {output} {ir_path}");
            } else {
                eprintln!("  compile manually: clang -o {output} {ir_path}");
            }
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("AXIOM Compiler v0.2.5 \"Hardened\" — Phase 1.5");
    eprintln!("Usage:");
    eprintln!("  axiomc <source.ax>                             print LLVM IR");
    eprintln!("  axiomc --emit-ir <source.ax>                   print LLVM IR");
    eprintln!("  axiomc -o <output> <source.ax>                 compile to native");
    eprintln!("  axiomc --run <source.ax>                       compile + run");
    eprintln!("  axiomc --target wasm <source.ax>               compile to WASM");
    eprintln!("  axiomc --target wasm -o out.wasm <source.ax>   compile to WASM");
    eprintln!("  axiomc --no-contracts <source.ax>              disable contract checks");
}

fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    let pos = args.iter().position(|a| a == flag)?;
    args.get(pos + 1).cloned()
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
