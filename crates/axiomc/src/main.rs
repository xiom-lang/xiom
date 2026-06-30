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
use std::process::{self, Command};

use axiom_ast::*;
use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_check::{Checker, BorrowChecker};
use axiom_codegen::IrEmitter;
use axiom_verify::SMTGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
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

    // Find the source file — last arg that doesn't start with -
    let source_path = args.iter().rev()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("error: no source file provided");
            process::exit(1);
        });

    if source_path.starts_with('-') || matches!(source_path.as_str(), "wasm" | "arm" | "riscv") {
        eprintln!("error: no source file provided");
        process::exit(1);
    }

    // ── Stage 1: Lex ──────────────────────────────────────
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            if diagnostics_json {
                println!(r#"{{"kind":"io_error","code":"F001","message":"cannot read '{source_path}': {e}","location":{{"file":"{source_path}","line":0,"col":0}}}}"#);
            } else {
                eprintln!("error: cannot read '{source_path}': {e}");
            }
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let lex_errors: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.kind, axiom_lexer::TokenKind::Error(_)))
        .collect();
    if !lex_errors.is_empty() {
        if diagnostics_json {
            let mut parts: Vec<String> = Vec::new();
            for tok in &lex_errors {
                if let axiom_lexer::TokenKind::Error(msg) = &tok.kind {
                    parts.push(format!(
                        r#"{{"kind":"lex_error","code":"L001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                        escape_json(msg), escape_json(source_path), tok.span.line, tok.span.col
                    ));
                }
            }
            println!("[{}]", parts.join(","));
        } else {
            for tok in &lex_errors {
                if let axiom_lexer::TokenKind::Error(msg) = &tok.kind {
                    eprintln!("error[L001]: {msg} at {l}:{c}", l = tok.span.line, c = tok.span.col);
                }
            }
        }
        process::exit(1);
    }

    // ── Stage 2: Parse ────────────────────────────────────
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            if diagnostics_json {
                println!(r#"{{"kind":"parse_error","code":"P001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                    escape_json(&e.message), escape_json(source_path), e.span.line, e.span.col);
            } else {
                eprintln!("error[P001]: {l}:{c}: {m}", l = e.span.line, c = e.span.col, m = e.message);
            }
            process::exit(1);
        }
    };

    // ── Stage 3: Type Check ───────────────────────────────
    let mut checker = Checker::new();
    if let Err(errors) = checker.check_program(&program) {
        if diagnostics_json {
            let parts: Vec<String> = errors.iter().map(|err| {
                format!(
                    r#"{{"kind":"type_error","code":"T001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                    escape_json(&err.message), escape_json(source_path), err.span.line, err.span.col
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

    // ── Stage 4: Borrow Check ─────────────────────────────
    let mut borrow_checker = BorrowChecker::new();
    if let Err(errors) = borrow_checker.check_program(&program) {
        if diagnostics_json {
            let parts: Vec<String> = errors.iter().map(|err| {
                format!(
                    r#"{{"kind":"borrow_error","code":"E001","message":"{}","location":{{"file":"{}","line":{},"col":{}}}}}"#,
                    escape_json(&err.message), escape_json(source_path), err.span.line, err.span.col
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
                    escape_json(&e), escape_json(source_path));
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

            let status = cmd.status();
            match status {
                Ok(s) if s.success() => {
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

fn print_usage() {
    eprintln!("AXIOM Compiler v0.10.0 \"Sovereign\" — Self-Hosted");
    eprintln!("Usage:");
    eprintln!("  axiomc <source.ax>                             print LLVM IR");
    eprintln!("  axiomc --emit-ir <source.ax>                   print LLVM IR");
    eprintln!("  axiomc -o <output> <source.ax>                 compile to native");
    eprintln!("  axiomc --run <source.ax>                       compile + run");
    eprintln!("  axiomc --target wasm <source.ax>               compile to WASM");
    eprintln!("  axiomc --target arm <source.ax>                compile to ARM (aarch64)");
    eprintln!("  axiomc --target riscv <source.ax>              compile to RISC-V (riscv64gc)");
    eprintln!("  axiomc --target wasm -o out.wasm <src.ax>      compile to WASM with name");
    eprintln!("  axiomc --no-contracts <source.ax>              disable contract checks");
    eprintln!("  axiomc --diagnostics=json <source.ax>          JSON-structured errors");
    eprintln!("  axiomc --dump-contracts <source.ax>            emit contract index JSON");
    eprintln!("  axiomc --verify <source.ax>                    SMT-LIB contract verification");
    eprintln!("  axiomc --verify-output <file> <source.ax>      SMT-LIB output to file");
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
    let candidates = [
        "stdlib\\runtime\\axiom_runtime.c",
        "stdlib/runtime/axiom_runtime.c",
    ];
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
