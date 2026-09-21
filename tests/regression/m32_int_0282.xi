// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 function param crossing to Int64
fn to_i64(a: UInt32) -> Int64 {
  return a as Int64;
}
fn main() -> Int {
  var x: UInt32 = 3000000000;
  var y: Int64 = to_i64(x);
  if y > 0 { return 0; }
  return 1;
}
