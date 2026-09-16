// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM fuzz target: CTFE (compile-time function evaluation).
// Register every parsed top-level fn in a fresh engine with a tight fuel
// budget and evaluate each with placeholder Int args. The evaluator is a
// flat work-stack machine (no host recursion), so the depth/fuel limits must
// yield diagnostics -- never a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;
use xiom_ast::TopDecl;
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
    let mut engine = xiom_ctfe::CtfeEngine::new();
    engine.max_steps = 20_000;
    engine.max_depth = 32;
    for item in &program.items {
        if let TopDecl::Fn(fd) = item {
            if let Some(body) = &fd.body {
                let params: Vec<String> =
                    fd.params.iter().map(|p| p.name.name.clone()).collect();
                engine.register_function(&fd.name.name, params, &body.stmts);
            }
        }
    }
    for item in &program.items {
        if let TopDecl::Fn(fd) = item {
            let args: Vec<xiom_ctfe::CtfeValue> =
                fd.params.iter().map(|_| xiom_ctfe::CtfeValue::Int(1)).collect();
            let _ = engine.eval_function(&fd.name.name, &args, 0);
        }
    }
});
