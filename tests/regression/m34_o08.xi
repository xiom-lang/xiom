// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O08: ? mixed with match -- ? propagation inside match arms (fixed)
fn div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn process(a: Int, b: Int, c: Int) -> Result[Int, Str] {
  var mid = div(a, b)?;
  match mid {
    0 => div(c, 2),
    _ => Ok(mid * c)
  }
}
fn main() -> Int {
  match process(10, 2, 3) { Ok(v) => { if v != 15 { return 1; } } Err(_) => { return 2; } }
  match process(0, 2, 3) { Ok(v) => { if v != 1 { return 3; } } Err(_) => { return 4; } }
  match process(10, 0, 3) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
