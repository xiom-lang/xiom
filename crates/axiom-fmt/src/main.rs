// AXIOM — Canonical Formatter
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::env;
use std::fs;
use std::process;

use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_fmt::Formatter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut check_mode = false;
    let mut files = Vec::new();

    for arg in &args[1..] {
        if arg == "--check" {
            check_mode = true;
        } else {
            files.push(arg.clone());
        }
    }

    if files.is_empty() {
        eprintln!("axiom fmt: missing file operand");
        eprintln!("Usage: axiom fmt [--check] <file.ax>");
        process::exit(1);
    }

    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("axiom fmt: {}: {}", file, e);
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
                eprintln!("axiom fmt: {}: parse error: {:?}", file, e);
                process::exit(1);
            }
        }
    }
}
