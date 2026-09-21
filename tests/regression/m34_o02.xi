// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O02: Simple ? on Err -- ? propagates Err immediately
fn fail() -> Result[Int, Str] { return Err("failed"); }
fn propagate() -> Result[Int, Str] {
  var v = fail()?;
  return Ok(v);
}
fn main() -> Int {
  match propagate() { Ok(_) => { return 1; } Err(_) => {} }
  return 0;
}
