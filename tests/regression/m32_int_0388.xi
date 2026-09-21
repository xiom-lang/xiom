// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 boundary compare: 0 < 1 < ... < 254 < 255
fn main() -> Int {
  var min: UInt8 = 0;
  var max: UInt8 = 255;
  var mid: UInt8 = 128;
  if min < mid && mid < max { return 0; }
  return 1;
}
