// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 mul wraparound (65536 * 65536 = 0)
fn main() -> Int {
  var a: UInt32 = 65536;
  var b: UInt32 = 65536;
  var c: UInt32 = a * b;
  if c == 0 as UInt32 { return 0; }
  return 1;
}
