// XIOM Contract Verifier — CLI entry point
// Generates SMT-LIB 2.6 output from XIOM contracts and optionally invokes Z3.
//
// Usage:
//   xiom-verify file.xi                 Generate SMT-LIB to stdout
//   xiom-verify file.xi -o out.smt      Write to file
//   xiom-verify file.xi --check           Run Z3 on generated SMT-LIB

use std::env;
use std::fs;
use std::process::Command;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_check::Checker;
use xiom_verify::SMTGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("XIOM Contract Verifier v0.1.0");
        eprintln!("Usage: xiom-verify <file.xi> [-o <file>] [--check] [--z3-path <path>]");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let source = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Error reading {file_path}: {e}");
        std::process::exit(1);
    });

    // Parse
    let tokens = Lexer::new(&source).tokenize();
    let program = Parser::new(tokens).parse_program().unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });

    // Type check
    let mut checker = Checker::new();
    if let Err(errors) = checker.check_program(&program) {
        eprintln!("Type errors:");
        for e in &errors {
            eprintln!("  {e:?}");
        }
        std::process::exit(1);
    }

    // Generate SMT-LIB
    let mut generator = SMTGenerator::new();
    let smt_output = generator.generate(&program);

    // Output
    let mut output_file = None;
    let mut check = false;
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
            "--check" => check = true,
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

    if let Some(path) = &output_file {
        fs::write(path, &smt_output).unwrap_or_else(|e| {
            eprintln!("Error writing {path}: {e}");
            std::process::exit(1);
        });
        if !check {
            println!("Wrote SMT-LIB to {path}");
        }
    } else if !check {
        println!("{smt_output}");
    }

    // Optionally check with Z3
    if check {
        let smt_path = output_file.clone().unwrap_or_else(|| "xiom_verify_tmp.smt2".to_string());
        let wrote_temp = output_file.is_none();
        if wrote_temp {
            fs::write(&smt_path, &smt_output).unwrap_or_else(|e| {
                eprintln!("Error writing temp file: {e}");
                std::process::exit(1);
            });
        }

        let output = Command::new(&z3_path)
            .arg(&smt_path)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                print!("{stdout}");
                if stdout.contains("unsat") {
                    println!("\nVERIFIED: All contracts hold.");
                } else if stdout.contains("sat") {
                    println!("\nCOUNTEREXAMPLE: Some contracts may be violated.");
                } else {
                    println!("\nUNKNOWN: Z3 could not determine validity.");
                }
            }
            Err(e) => {
                eprintln!("Error invoking Z3 ({z3_path}): {e}");
                eprintln!("SMT-LIB output written to {smt_path}");
            }
        }

        if wrote_temp {
            let _ = fs::remove_file(&smt_path);
        }
    }
}
