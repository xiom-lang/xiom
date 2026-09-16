// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn safe_div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn compute(a: Int) -> Result[Int, Str] {
  var x = safe_div(a, 2)?;
  var y = safe_div(x, 2)?;
  return Ok(y);
}
fn main() -> Int {
  match compute(32) { Ok(v) => { if v != 8 { return 1; } } Err(_) => { return 2; } }
  match compute(0) { Ok(v) => { if v != 0 { return 3; } } Err(_) => { return 4; } }
  return 0;
}
