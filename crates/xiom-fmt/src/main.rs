// XIOM — Canonical Formatter
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::env;
use std::fs;
use std::process;

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_fmt::Formatter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut check_mode = false;
    let mut files = Vec::new();

    for arg in &args[1..] {
        if arg == "--check" {
            check_mode = true;
        } else if arg == "--help" {
            print_usage();
            return;
        } else {
            files.push(arg.clone());
        }
    }

    if files.is_empty() {
        print_usage();
        process::exit(1);
    }

    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("xiom fmt: {}: {}", file, e);
            process::exit(1);
        });

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(program) => {
                let mut formatter = Formatter::new();
                let formatted = formatter.format(&program);

                if check_mode {
                    if formatted != source {
                        eprintln!("{}: not canonically formatted", file);
                        process::exit(1);
                    }
                } else {
                    print!("{}", formatted);
                }
            }
            Err(e) => {
                eprintln!("xiom fmt: {}: parse error: {:?}", file, e);
                process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!("XIOM Format v0.10.1 -- Canonical Formatter");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom fmt [OPTIONS] <file.xi>");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help        Show this help message");
    eprintln!("  --check       Check only (exit 1 if not formatted, no output)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom fmt source.xi");
    eprintln!("  xiom fmt --check source.xi");
}
