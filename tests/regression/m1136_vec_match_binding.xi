// m1136 (REL-20261021-1136-orbitdb): `let v = match ... { Ok(x) => { x } }`
// bound only 8 bytes of the Vec; len 3 became 1 (silent truncation).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The match result slot was inferred as i64 because `infer_match_llvm_type`
// did not apply the scrutinee-payload inference to BLOCK-tailed arms (only to
// direct `Ok(x) => x` arms). The 32-byte Vec was stored through an i64 slot,
// so the let-bound header was truncated and v.len() dispatched as Str.len on
// the data pointer. Block arms now use the same payload inference.

module m1136_vec_match_binding

use xiom.io;
use xiom.convert;

fn make3() -> Result[Vec[Int], Int] {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  return Ok(v);
}

fn via_let() -> Int {
  let v = match make3() {
    Ok(x) => { x }
    Err(_) => { return -1; }
  };
  return v.len();
}

fn via_var() -> Int {
  var v = Vec[Int].new();
  match make3() {
    Ok(x) => { v = x; }
    Err(_) => { return -1; }
  };
  return v.len();
}

fn direct_len() -> Int {
  match make3() {
    Ok(x) => { return x.len(); }
    Err(_) => { return -1; }
  }
}

fn main() -> Int {
  let a = via_let();
  let b = via_var();
  let c = direct_len();
  io.println("via_let=" + convert.int_to_string(a) + " via_var=" + convert.int_to_string(b) + " direct=" + convert.int_to_string(c));
  if a != 3 { return 1; }
  if b != 3 { return 2; }
  if c != 3 { return 3; }
  return 0;
}
