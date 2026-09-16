// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(200) as Int32 must be 200, not -56 (zext vs sext)
fn main() -> Int {
  var a: UInt8 = 200;
  var b: Int32 = a as Int32;
  if b == 200 as Int32 { return 0; }
  return 1;
}
