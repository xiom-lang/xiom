// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_gcm2

pub fn make_pair() -> Result[(Vec[UInt8], Vec[UInt8]), Str] {
  var a = Vec[UInt8].new();
  a.push(1); a.push(2); a.push(3);
  var b = Vec[UInt8].new();
  b.push(9); b.push(8);
  return Ok((a, b));
}

pub fn consume_pair(x: &Vec[UInt8], y: &Vec[UInt8]) -> Int {
  if x.len() != 3 { return 1; }
  if y.len() != 2 { return 2; }
  if x[2] != 3 { return 3; }
  if y[1] != 8 { return 4; }
  return 0;
}
