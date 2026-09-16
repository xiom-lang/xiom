// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O17: Chained ? with and_then pattern -- manual and_then via match+?
fn validate_positive(n: Int) -> Result[Int, Str] {
  if n > 0 { return Ok(n); }
  return Err("not positive");
}
fn validate_even(n: Int) -> Result[Int, Str] {
  if n % 2 == 0 { return Ok(n); }
  return Err("not even");
}
fn validate_lt100(n: Int) -> Result[Int, Str] {
  if n < 100 { return Ok(n); }
  return Err("too large");
}
fn and_then_chain(n: Int) -> Result[Int, Str] {
  var a = validate_positive(n)?;
  var b = validate_even(a)?;
  var c = validate_lt100(b)?;
  return Ok(c);
}
fn main() -> Int {
  match and_then_chain(42) { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  match and_then_chain(-1) { Ok(_) => { return 3; } Err(_) => {} }
  match and_then_chain(3) { Ok(_) => { return 4; } Err(_) => {} }
  match and_then_chain(200) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
