// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 increment loop wraparound test
fn main() -> Int {
  var i: UInt8 = 253;
  while i < 255 as UInt8 {
    i = i + 1 as UInt8;
  }
  var j: UInt8 = i + 1 as UInt8;
  if j == 0 as UInt8 { return 0; }
  return 1;
}
