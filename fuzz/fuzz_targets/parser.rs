// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM fuzz target: parser.
// Tokenize + parse arbitrary input; the parser has panic-mode recovery and a
// depth guard (MAX_EXPR_DEPTH), so every input must return Ok/Err -- never
// panic, never hang. Nested-delimiter inputs exercise the depth guard.
#![no_main]

use libfuzzer_sys::fuzz_target;
use xiom_lexer::Lexer;
use xiom_parser::Parser;

fuzz_target!(|data: &[u8]| {
    let src = String::from_utf8_lossy(data);
    let tokens = Lexer::new(&src).tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse_program();
    let _ = parser.take_errors();
});
