// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM fuzz target: full front-half pipeline (lexer -> parser -> codegen).
// The largest attack surface: monomorphization, contract lowering, sandbox
// emission. `compile_program` must always return Ok/Err for a program the
// parser accepted (the e2e suite asserts the same property on curated
// inputs; this explores the space under coverage guidance).
#![no_main]

use libfuzzer_sys::fuzz_target;
use xiom_codegen::IrEmitter;
use xiom_lexer::Lexer;
use xiom_parser::Parser;

fuzz_target!(|data: &[u8]| {
    let src = String::from_utf8_lossy(data);
    let tokens = Lexer::new(&src).tokenize();
    let mut parser = Parser::new(tokens);
    let Ok(program) = parser.parse_program() else { return };
    if !parser.take_errors().is_empty() {
        return;
    }
    let mut emitter = IrEmitter::new();
    let _ = emitter.compile_program(&program);
});
