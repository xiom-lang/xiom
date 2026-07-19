// XIOM Contract Verifier — CLI entry point (Phase 5f Stage 0)
// Generates corrected SMT-LIB 2.6 with body encoding and optionally invokes Z3.
//
// Usage:
//   xiom-verify file.xi                 Emit SMT-LIB to stdout
//   xiom-verify file.xi -o out.smt      Write to file
//   xiom-verify file.xi --check           Run Z3 (requires z3 on PATH)
//   xiom-verify file.xi --z3-path /path  Use specific z3 binary

use std::env;
use std::fs;
use std::process::{self, Command};
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_check::Checker;
use xiom_verify::{SMTGenerator, Z3Runner, VerifyResult};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("XIOM Contract Verifier v0.9.0 (Phase 5f)");
        eprintln!("Usage: xiom-verify <file.xi> [-o <file>] [--check] [--z3-path <path>]");
        process::exit(1);
    }

    let file_path = &args[1];
    let source = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Error reading {file_path}: {e}");
        process::exit(1);
    });

    // Parse
    let tokens = Lexer::new(&source).tokenize();
    let program = Parser::new(tokens).parse_program().unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        process::exit(1);
    });

    // Type check
    let mut checker = Checker::new();
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        checker.add_source_dir(parent.to_string_lossy().to_string());
    }
    if let Err(errors) = checker.check_program(&program) {
        for e in &errors {
            eprintln!("  {e:?}");
        }
        process::exit(1);
    }

    // Generate SMT-LIB
    let mut generator = SMTGenerator::new();
    let smt_output = generator.generate(&program);

    // Parse CLI args
    let mut output_file = None;
    let mut do_check = false;
    let mut z3_path = "z3".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--check" => do_check = true,
            "--z3-path" => {
                if i + 1 < args.len() {
                    z3_path = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Write SMT to file or print contract summary
    if let Some(path) = &output_file {
        fs::write(path, &smt_output).unwrap_or_else(|e| {
            eprintln!("Error writing {path}: {e}");
            process::exit(1);
        });
        if !do_check {
            eprintln!("SMT-LIB written to {path}");
        }
    } else if !do_check {
        // Print contract summary instead of full SMT
        let contract_count = smt_output.lines().filter(|l| l.contains("; === Function:")).count();
        let ensures_count = smt_output.lines().filter(|l| l.contains("; ensures")).count();
        let requires_count = smt_output.lines().filter(|l| l.contains("; requires")).count();
        eprintln!("Generated {contract_count} function(s) with {requires_count} requires, {ensures_count} ensures.");
        eprintln!("Use --check to verify with Z3, or -o <file> to write SMT-LIB.");
    }

    // Check with Z3
    if do_check {
        let runner = Z3Runner::new().with_z3_path(&z3_path);
        let results = runner.verify(&smt_output);

        let mut proven = 0;
        let mut violated = 0;
        let mut inconclusive = 0;
        let mut errors = 0;

        for result in &results {
            match result {
                VerifyResult::Proven => {
                    proven += 1;
                    eprintln!("  ✅ VERIFIED");
                }
                VerifyResult::Violated { code, message, counterexample } => {
                    violated += 1;
                    eprintln!("  ❌ VIOLATED [{code}]: {message}");
                    if let Some(ce) = counterexample {
                        if !ce.values.is_empty() {
                            for (var, val) in &ce.values {
                                eprintln!("      {} = {}", var, val);
                            }
                        }
                    }
                }
                VerifyResult::Inconclusive { reason } => {
                    inconclusive += 1;
                    eprintln!("  ⚠️  UNKNOWN: {reason}");
                }
                VerifyResult::Error { message } => {
                    errors += 1;
                    eprintln!("  🔴 ERROR: {message}");
                }
            }
        }

        if errors > 0 {
            eprintln!("\nz3 invocation failed. Install z3 and ensure it is on PATH.");
            eprintln!("Download: https://github.com/Z3Prover/z3/releases");
            if output_file.is_none() {
                let smt_path = "xiom_verify_output.smt2";
                fs::write(smt_path, &smt_output).unwrap_or_else(|e| {
                    eprintln!("Error writing fallback SMT: {e}");
                });
                eprintln!("SMT-LIB written to {smt_path} for manual checking.");
            }
        }

        let summary = format!(
            "Results: {} proven, {} violated, {} unknown, {} errors",
            proven, violated, inconclusive, errors
        );
        eprintln!("\n{summary}");

        if !output_file.is_some() && errors == 0 {
            let smt_path = "xiom_verify_output.smt2";
            fs::write(smt_path, &smt_output).ok();
        }
    }
}
