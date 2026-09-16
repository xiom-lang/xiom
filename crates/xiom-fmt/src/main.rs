// XIOM -- Canonical Formatter
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut check_mode = false;
    let mut in_place = false;
    let mut files = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "--check" => check_mode = true,
            "--in-place" | "-i" => in_place = true,
            "--help" => {
                print_usage();
                return;
            }
            "--version" => {
                println!("xiom-fmt v{}",
                    env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => files.push(arg.clone()),
        }
    }

    if files.is_empty() {
        print_usage();
        process::exit(1);
    }

    let mut exit_code = 0;

    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("xiom fmt: {}: {}", file, e);
            process::exit(1);
        });

        match xiom_fmt::format_source_text(&source) {
            Ok(formatted) => {
                if check_mode {
                    if formatted != source {
                        eprintln!("{}: not canonically formatted", file);
                        exit_code = 1;
                    }
                } else if in_place {
                    if formatted != source {
                        fs::write(file, &formatted).unwrap_or_else(|e| {
                            eprintln!("xiom fmt: {}: write error: {}", file, e);
                            process::exit(1);
                        });
                    }
                } else {
                    print!("{formatted}");
                }
            }
            Err(e) => {
                eprintln!("xiom fmt: {}: {}", file, e);
                process::exit(1);
            }
        }
    }

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn print_usage() {
    eprintln!("XIOM Format v0.47.6 -- Canonical Formatter");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom fmt [OPTIONS] <file.xi> [file2.xi ...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help           Show this help message");
    eprintln!("  --check          Check only (exit 1 if not formatted, no output)");
    eprintln!("  --in-place, -i   Write formatted output back to the file");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom fmt source.xi              Print formatted to stdout");
    eprintln!("  xiom fmt --check source.xi      Verify formatting only");
    eprintln!("  xiom fmt -i source.xi           Format file in-place");
}
