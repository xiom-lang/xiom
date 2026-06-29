//! AXIOM Compiler CLI — Phase 0
//! Usage: axiomc <source.ax>
//!        axiomc --emit-ir <source.ax>    (print LLVM IR)
//!        axiomc -o <output> <source.ax>  (compile via llc)

use std::env;
use std::fs;
use std::process::{self, Command};

use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_check::Checker;
use axiom_codegen::IrEmitter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("AXIOM Compiler v0.1 — Phase 0");
        eprintln!("Usage:");
        eprintln!("  axiomc <source.ax>                    print LLVM IR to stdout");
        eprintln!("  axiomc --emit-ir <source.ax>          print LLVM IR to stdout");
        eprintln!("  axiomc -o <output> <source.ax>        compile to native binary");
        eprintln!("  axiomc --target wasm <source.ax>     compile to WASM");
        process::exit(1);
    }

    // Parse flags
    let emit_ir = args.iter().any(|a| a == "--emit-ir");
    let target_wasm = args.iter().any(|a| a == "--target" && args.iter().position(|x| x == a).map_or(false, |i| args.get(i + 1).map_or(false, |v| v == "wasm")));

    let mut output_file = None;
    if let Some(pos) = args.iter().position(|a| a == "-o") {
        if let Some(out) = args.get(pos + 1) {
            output_file = Some(out.clone());
        }
    }

    // Find the source file (last positional arg)
    let source_path = args.iter().rev()
        .find(|a| !a.starts_with("--") && !a.starts_with('-'))
        .expect("no source file provided");

    if source_path == "wasm" || source_path.starts_with('-') {
        eprintln!("error: no source file provided");
        process::exit(1);
    }

    // Read source
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", source_path, e);
            process::exit(1);
        }
    };

    // ── Stage 1: Lex ──────────────────────────────────────────
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let lex_errors: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.kind, axiom_lexer::TokenKind::Error(_)))
        .collect();
    if !lex_errors.is_empty() {
        for tok in &lex_errors {
            if let axiom_lexer::TokenKind::Error(ref msg) = tok.kind {
                eprintln!("error[L001]: {} at {}:{}", msg, tok.span.line, tok.span.col);
            }
        }
        process::exit(1);
    }

    // ── Stage 2: Parse ─────────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error[P001]: {}:{}: {}", e.span.line, e.span.col, e.message);
            process::exit(1);
        }
    };

    // ── Stage 3: Type Check ────────────────────────────────────
    let mut checker = Checker::new();
    if let Err(errors) = checker.check_program(&program) {
        for err in &errors {
            eprintln!("error[T{:03}]: {}:{}: {}", 1, err.span.line, err.span.col, err.message);
        }
        process::exit(1);
    }

    // ── Stage 4: Codegen ───────────────────────────────────────
    let mut emitter = IrEmitter::new();
    let llvm_ir = match emitter.compile_program(&program) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("error[C001]: codegen: {e}");
            process::exit(1);
        }
    };

    if emit_ir || output_file.is_none() {
        // Print IR to stdout
        println!("{llvm_ir}");
        return;
    }

    // ── Stage 5: Compile to binary via llc + clang ──────────────
    let output = output_file.as_deref().unwrap_or("a.out");

    // Write IR to temp file
    let ir_path = format!("{output}.ll");
    if let Err(e) = fs::write(&ir_path, &llvm_ir) {
        eprintln!("error: cannot write IR file: {e}");
        process::exit(1);
    }

    // Run llc to produce object file
    let obj_path = format!("{output}.o");
    let llc_target = if target_wasm { "wasm32-unknown-unknown" } else { "x86_64-pc-windows-msvc" };

    let llc_status = Command::new("llc")
        .args(["-filetype=obj", &format!("-mtriple={llc_target}"), "-o", &obj_path, &ir_path])
        .status();

    match llc_status {
        Ok(s) if s.success() => {
            if target_wasm {
                // Rename .o to .wasm
                let wasm_path = format!("{output}.wasm");
                let _ = fs::rename(&obj_path, &wasm_path);
                eprintln!("compiled: {wasm_path}");
            }
        }
        Ok(s) => {
            eprintln!("error: llc failed with exit code {}", s.code().unwrap_or(-1));
            let _ = fs::remove_file(&ir_path);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("error: cannot run llc (LLVM not installed or not in PATH): {e}");
            eprintln!("note: LLVM IR written to {ir_path} — compile manually with: llc {ir_path}");
            process::exit(1);
        }
    }

    if !target_wasm {
        // Link with clang to produce executable
        let clang_status = Command::new("clang")
            .args(["-o", output, &obj_path])
            .status();

        match clang_status {
            Ok(s) if s.success() => {
                eprintln!("compiled: {output}");
                // Clean up temp files
                let _ = fs::remove_file(&ir_path);
                let _ = fs::remove_file(&obj_path);
            }
            Ok(s) => {
                eprintln!("error: clang linking failed with exit code {}", s.code().unwrap_or(-1));
                eprintln!("note: object file at {obj_path}");
                process::exit(1);
            }
            Err(e) => {
                eprintln!("error: cannot run clang (not in PATH): {e}");
                eprintln!("note: object file at {obj_path} — link manually");
                process::exit(1);
            }
        }
    }
}
