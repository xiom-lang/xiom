// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X09: Error message patterns -- exercise various error path scenarios
enum MathErr { DivByZero, Overflow, Underflow, Invalid }
fn checked_div(a: Int, b: Int) -> Result[Int, MathErr] {
  if b == 0 { return Err(MathErr.DivByZero); }
  return Ok(a / b);
}
fn checked_add(a: Int, b: Int, max: Int) -> Result[Int, MathErr] {
  if a + b > max { return Err(MathErr.Overflow); }
  return Ok(a + b);
}
fn validate_input(x: Int) -> Result[Int, MathErr] {
  if x < 0 { return Err(MathErr.Invalid); }
  return Ok(x);
}
fn process(a: Int, b: Int, max: Int) -> Result[Int, MathErr] {
  var v = match validate_input(a) {
    Ok(val) => val,
    Err(e) => { return Err(e); }
  };
  var d = match checked_div(v, b) {
    Ok(val) => val,
    Err(e) => { return Err(e); }
  };
  return checked_add(d, v, max);
}
fn err_to_int(e: MathErr) -> Int {
  match e {
    MathErr.DivByZero => 100,
    MathErr.Overflow => 200,
    MathErr.Underflow => 300,
    MathErr.Invalid => 400,
  }
}
fn main() -> Int {
  var r1 = process(10, 2, 100);
  match r1 { Ok(v) => if v != 15 { return 1; } Err(_) => { return 2; } }
  var r2 = process(10, 0, 100);
  match r2 { Ok(_) => { return 3; } Err(e) => if err_to_int(e) != 100 { return 4; } }
  var r3 = process(5, 1, 8);
  match r3 { Ok(_) => { return 5; } Err(e) => if err_to_int(e) != 200 { return 6; } }
  var r4 = process(-1, 1, 100);
  match r4 { Ok(_) => { return 7; } Err(e) => if err_to_int(e) != 400 { return 8; } }
  return 0;
}
