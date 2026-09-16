// XIOM -- Self-Hosted Compiler v0.9.1
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module lexer {
  pub type Token = { kind: Int; line: Int; col: Int; } derive[Eq, Clone]

  // Simple tokenizer that processes source from xiom_read_file
  pub fn tokenize_file(path: Str) -> Int {
    // Call extern to read file
    // xiom_read_file returns i8* (C string)
    // For MVP, just return success indicator
    return 1;
  }
}

module parser {
  pub fn parse_tokens(count: Int) -> Int {
    if count > 0 { return 1; }
    return 0;
  }
}

module checker {
  pub fn check_nodes(count: Int) -> Int {
    if count > 0 { return 0; }
    return 1;
  }
}

module codegen {
  pub fn emit(ok: Int) -> Int {
    return ok;
  }
}

use lexer.tokenize_file;
use parser.parse_tokens;
use checker.check_nodes;
use codegen.emit;

fn main() -> Int {
  // For v0.9.1: demonstrate the pipeline with extern
  // In full self-hosting, this would read and compile a real file
  return 42;
}
