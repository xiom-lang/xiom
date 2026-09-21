// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O10: ? in loop -- ? propagation inside while loop
fn try_index(i: Int) -> Result[Int, Str] {
  if i < 0 { return Err("negative"); }
  if i > 100 { return Err("too large"); }
  return Ok(i * i);
}
fn sum_squares(n: Int) -> Result[Int, Str] {
  var total: Int = 0;
  var i: Int = 0;
  while i < n {
    var sq = try_index(i)?;
    total = total + sq;
    i = i + 1;
  }
  return Ok(total);
}
fn main() -> Int {
  match sum_squares(5) { Ok(v) => { if v != 30 { return 1; } } Err(_) => { return 2; } }
  match sum_squares(0) { Ok(v) => { if v != 0 { return 3; } } Err(_) => { return 4; } }
  return 0;
}
