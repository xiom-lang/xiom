// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C22: match on Bool -- boolean dispatch
fn to_int(b: Bool) -> Int {
  match b {
    true => 1,
    false => 0,
  }
}
fn main() -> Int {
  var x: Bool = true;
  match x {
    true => { var v: Int = 42; }
    false => { return 1; }
  }
  match x {
    false => { return 2; }
    true => {}
  }
  if to_int(true) != 1 { return 3; }
  if to_int(false) != 0 { return 4; }
  return 0;
}
