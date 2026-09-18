// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XIOM WASM -- Browser-based compiler for the playground
//! Compiles XIOM source -> LLVM IR and diagnostics entirely in the browser.
//! No file system, no process spawning, no external dependencies.

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WasmDiagnostic {
    pub kind: String,
    pub code: String,
    pub message: String,
    pub line: u64,
    pub col: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub ir: Option<String>,
    pub diagnostics: Vec<WasmDiagnostic>,
}

/// Compile XIOM source code to LLVM IR.
/// Returns JSON-serialized CompileResult.
#[wasm_bindgen]
pub fn compile_xiom(source: &str) -> String {
    let mut diagnostics = Vec::new();

    // Stage 1: Lex
    let mut lexer = xiom_lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    // Collect lex errors
    for tok in &tokens {
        if let xiom_lexer::TokenKind::Error(msg) = &tok.kind {
            diagnostics.push(WasmDiagnostic {
                kind: "lex_error".into(),
                code: "L001".into(),
                message: msg.clone(),
                line: tok.span.line as u64,
                col: tok.span.col as u64,
            });
        }
    }

    if !diagnostics.is_empty() {
        return serde_json::to_string(&CompileResult {
            success: false,
            ir: None,
            diagnostics,
        }).unwrap_or_default();
    }

    // Stage 2: Parse
    let mut parser = xiom_parser::Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            diagnostics.push(WasmDiagnostic {
                kind: "parse_error".into(),
                code: "P001".into(),
                message: e.message,
                line: e.span.line as u64,
                col: e.span.col as u64,
            });
            return serde_json::to_string(&CompileResult {
                success: false, ir: None, diagnostics,
            }).unwrap_or_default();
        }
    };

    // Collect parser errors
    for e in parser.errors() {
        diagnostics.push(WasmDiagnostic {
            kind: "parse_error".into(),
            code: "P001".into(),
            message: e.message.clone(),
            line: e.span.line as u64,
            col: e.span.col as u64,
        });
    }

    if !diagnostics.is_empty() {
        return serde_json::to_string(&CompileResult {
            success: false, ir: None, diagnostics,
        }).unwrap_or_default();
    }

    // Stage 3: Check
    let mut checker = xiom_check::Checker::new();
    checker.add_source_dir("/wasm/stdlib".into());
    checker.build_catalog_index();

    match checker.check_program(&program) {
        Ok(()) => {},
        Err(errors) => {
            for e in &errors {
                diagnostics.push(WasmDiagnostic {
                    kind: "type_error".into(),
                    code: "T001".into(),
                    message: e.message.clone(),
                    line: e.span.line as u64,
                    col: e.span.col as u64,
                });
            }
            return serde_json::to_string(&CompileResult {
                success: false, ir: None, diagnostics,
            }).unwrap_or_default();
        }
    };

    // Stage 4: Codegen
    let mut emitter = xiom_codegen::IrEmitter::new();
    emitter.set_check_contracts(true);
    let ir = match emitter.compile_program(&program) {
        Ok(ir) => ir,
        Err(e) => {
            diagnostics.push(WasmDiagnostic {
                kind: "codegen_error".into(),
                code: "C001".into(),
                message: e,
                line: 0, col: 0,
            });
            return serde_json::to_string(&CompileResult {
                success: false, ir: None, diagnostics,
            }).unwrap_or_default();
        }
    };

    serde_json::to_string(&CompileResult {
        success: true,
        ir: Some(ir),
        diagnostics,
    }).unwrap_or_default()
}

/// Get compiler version string.
#[wasm_bindgen]
pub fn get_version() -> String {
    format!("XIOM v{} (WASM)", env!("CARGO_PKG_VERSION"))
}
