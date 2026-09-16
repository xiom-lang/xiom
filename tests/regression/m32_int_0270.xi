// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 multiplication inside range (65536 * 32768 = 2147483648)
fn main() -> Int {
  var a: UInt32 = 65536;
  var b: UInt32 = 32768;
  var c: UInt32 = a * b;
  if c == 2147483648 as UInt32 { return 0; }
  return 1;
}
