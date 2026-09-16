// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O05: 5-chain ? -- five consecutive ? in sequence
fn step(a: Int, delta: Int) -> Result[Int, Str] {
  if a < 0 { return Err("negative"); }
  return Ok(a + delta);
}
fn five_step(start: Int) -> Result[Int, Str] {
  var a = step(start, 1)?;
  var b = step(a, 2)?;
  var c = step(b, 3)?;
  var d = step(c, 4)?;
  var e = step(d, 5)?;
  return Ok(e);
}
fn main() -> Int {
  match five_step(0) { Ok(v) => { if v != 15 { return 1; } } Err(_) => { return 2; } }
  match five_step(-1) { Ok(_) => { return 3; } Err(_) => {} }
  match five_step(-2) { Ok(_) => { return 4; } Err(_) => {} }
  return 0;
}
