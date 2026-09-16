// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O03: 2-chain ? -- two consecutive ? in sequence
fn add_one(a: Int) -> Result[Int, Str] {
  if a < 0 { return Err("negative"); }
  return Ok(a + 1);
}
fn add_two(a: Int) -> Result[Int, Str] {
  var x = add_one(a)?;
  var y = add_one(x)?;
  return Ok(y);
}
fn main() -> Int {
  match add_two(5) { Ok(v) => { if v != 7 { return 1; } } Err(_) => { return 2; } }
  match add_two(-1) { Ok(_) => { return 3; } Err(_) => {} }
  return 0;
}
