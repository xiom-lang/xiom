// XIOM -- Selfhost parser (Phase 2: parity port of crates/xiom-parser)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Facade for the Phase 2 parser port:
//   * lexer bridge: selfhost_lexer::lx_tokenize
//   * parser state: selfhost_parser_state::Parser (latch-based errors)
//   * ported grammar: selfhost_parser_expr (types/patterns/stmts/exprs) and
//     selfhost_parser_core (program + top-level declarations)
//   * canonical dump: selfhost_ast_dump::dump_nodes (the line-exact mirror
//     of crates/xiom/src/main.rs::dump_ast)
//
// `dump_ast` prints the dump on stdout and returns 0. A parse that aborts
// (Rust `parse_program` Err) prints the single canonical `PARSE-ERROR` line,
// matching the Rust driver's behaviour.

module selfhost_parser

use xiom.io;
use selfhost_ast_dump;
use selfhost_lexer;
use selfhost_parser_core;
use selfhost_parser_state;

/// Phase 0 compatibility: number of AST nodes parsed from `src`. The real
/// checker/codegen pipeline is wired in Phase 3; the dump gate uses
/// `dump_ast` directly.
pub fn parse_count(src: Str) -> Int {
  return 0;
}

/// Canonical AST dump (Phase 2 parity gate entry point).
pub fn dump_ast(src: Str) -> Int {
  var lx = selfhost_lexer.Lexer.new(src);
  let toks = selfhost_lexer.lx_tokenize(&mut lx);
  var p = selfhost_parser_state.p_new(toks);
  let root = selfhost_parser_core.pc_parse_program(&mut p);
  if root < 0 || selfhost_parser_state.p_failed(&p) {
    io.println("PARSE-ERROR");
    return 0;
  }
  return selfhost_ast_dump.dump_nodes(&p.nodes, root);
}
