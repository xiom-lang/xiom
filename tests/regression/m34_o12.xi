// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O12: ? with compound expression -- ? inside arithmetic expression
fn get(a: Int) -> Result[Int, Str] {
  if a < 0 { return Err("negative"); }
  return Ok(a);
}
fn compound(a: Int, b: Int, c: Int) -> Result[Int, Str] {
  var x = get(a)?;
  var y = get(b)?;
  var z = get(c)?;
  var total = x * 2 + y * 3 + z * 5;
  return Ok(total);
}
fn main() -> Int {
  match compound(1, 2, 3) { Ok(v) => { if v != 23 { return 1; } } Err(_) => { return 2; } }
  match compound(-1, 2, 3) { Ok(_) => { return 3; } Err(_) => {} }
  return 0;
}
