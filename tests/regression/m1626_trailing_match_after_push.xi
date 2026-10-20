// m1626 (REL-20261011-1626-stdlib): a bare trailing `match` after a
// `Vec.push` statement was parsed as Stmt::Match, so the block had no tail
// expression and the checker reported T001 (expected Result[...], found ()).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// A semicolon-less match at the END of a block is now the block's tail
// expression. An explicit `match ...;` keeps statement/discard semantics
// (locked by `discard` below).

module m1626_trailing_match_after_push

use xiom.io;

fn classify() -> Result[Float64, Str] {
  var scratch = Vec[UInt8].new();
  let b: UInt8 = 53;
  scratch.push(b);
  let parsed: Result[Float64, Str] = Ok(5.0);
  match parsed {
    Ok(v) => { Ok(v) }
    Err(_) => { Err("bad") }
  }
}

fn discard(x: Int) {
  match x {
    1 => { let _ = 1; }
    _ => { let _ = 0; }
  };
}

fn side(x: Int) {
  let _ = x;
}

fn unit_match_expr() {
  // m1626 follow-up (selfhost T3): a value-shaped match whose arms are Unit
  // calls is an EXPRESSION; the result slot must not be `alloca void`.
  let y = match 1 {
    1 => { side(1); }
    _ => { side(2); }
  };
  let _ = y;
}

fn main() -> Int {
  match classify() {
    Ok(v) => { if v != 5.0 { return 1; } }
    Err(_) => { return 2; }
  }
  discard(1);
  unit_match_expr();
  io.println("trailing_match_ok");
  return 0;
}
