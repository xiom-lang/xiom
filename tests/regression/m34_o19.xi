// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O19: ? with method -- struct with associated function using ?
type Token = { kind: Int; val: Int; }

fn token_to_val(t: Token) -> Result[Int, Str] {
  if t.kind == 0 { return Ok(t.val); }
  return Err("not a number");
}
fn token_double(t: Token) -> Result[Int, Str] {
  var v = token_to_val(t)?;
  return Ok(v * 2);
}
fn main() -> Int {
  var t = Token{ kind: 0, val: 21 };
  match token_double(t) { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  var o = Token{ kind: 1, val: 0 };
  match token_double(o) { Ok(_) => { return 3; } Err(_) => {} }
  return 0;
}
