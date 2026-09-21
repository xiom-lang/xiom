// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O20: ? with cross-function chain -- standalone functions using ? together
fn safe_add(a: Int, b: Int) -> Result[Int, Str] {
  if a > 1000 || b > 1000 { return Err("overflow"); }
  return Ok(a + b);
}
fn safe_mul(a: Int, b: Int) -> Result[Int, Str] {
  if a > 100 || b > 100 { return Err("overflow"); }
  return Ok(a * b);
}
fn compute(a: Int, b: Int, c: Int) -> Result[Int, Str] {
  var sum = safe_add(a, b)?;
  var prod = safe_mul(sum, c)?;
  return Ok(prod);
}
fn main() -> Int {
  match compute(10, 20, 3) { Ok(v) => { if v != 90 { return 1; } } Err(_) => { return 2; } }
  match compute(10, 20, 0) { Ok(v) => { if v != 0 { return 3; } } Err(_) => { return 4; } }
  match compute(2000, 1, 1) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
