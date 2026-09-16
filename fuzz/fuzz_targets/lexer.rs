// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM fuzz target: lexer.
// Arbitrary bytes (fed as lossy UTF-8, matching what sources can contain)
// must never panic the lexer. Audit finding: the toy LCG harness only
// explored 500 pseudo-random ASCII strings; this explores the full byte
// space under libFuzzer with coverage guidance.
#![no_main]

use libfuzzer_sys::fuzz_target;
use xiom_lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    let src = String::from_utf8_lossy(data);
    let mut lexer = Lexer::new(&src);
    let tokens = lexer.tokenize();
    // A byte-wise lexer can never emit more tokens than it has bytes plus an
    // EOF marker. Violations mean a token loop failed to advance.
    assert!(
        tokens.len() <= src.len() + 1,
        "lexer emitted {} tokens for {} bytes",
        tokens.len(),
        src.len()
    );
});
