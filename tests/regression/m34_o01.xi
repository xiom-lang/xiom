// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O01: Simple ? on Ok -- ? unwraps Ok value in Result-returning fn
fn get_val() -> Result[Int, Str] { return Ok(42); }
fn use_val() -> Result[Int, Str] {
  var v = get_val()?;
  return Ok(v * 2);
}
fn main() -> Int {
  match use_val() { Ok(v) => { if v != 84 { return 1; } } Err(_) => { return 2; } }
  return 0;
}
