// XIOM -- Selfhost compiler entry point (Phase 0 skeleton)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Driver: reads arguments, loads the source file, runs the stages in order
// (lexer -> parser -> checker -> codegen), and writes IR to stdout.
//
// Usage:
//   xiomc-self <source.xi>    compile-to-IR (Phase 0: stub module)
//   xiomc-self --selfcheck    run the runtime_ffi behavior checks
//
// Build (the harness does this): xiom -o target/selfhost/xiomc-self.exe
// selfhost/src/main.xi

module selfhost_main

use xiom.io;
use selfhost_checker;
use selfhost_codegen;
use selfhost_lexer;
use selfhost_parser;
use selfhost_selfcheck;

fn main() -> Int {
  var args = io.args();
  if args.len() < 2 {
    io.println("usage: xiomc-self <source.xi> | --selfcheck");
    return 2;
  }
  let first = args[1];
  if first == "--selfcheck" {
    return selfhost_selfcheck.run();
  }

  var source = "";
  match io.read_file(first) {
    Ok(text) => { source = text; }
    Err(e) => {
      io.println("xiomc-self: cannot read " + first + ": " + e.message);
      return 3;
    }
  }

  let token_count = selfhost_lexer.lex_count(&source);
  let node_count = selfhost_parser.parse_count(&source);
  let error_count = selfhost_checker.check_count(&source);
  if error_count > 0 { return 4; }
  if token_count < 0 || node_count < 0 { return 5; }
  return selfhost_codegen.emit_program(&source);
}
