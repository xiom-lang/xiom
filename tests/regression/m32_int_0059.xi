// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Signed to unsigned conversion Int32 -> UInt32
fn main() -> Int {
  var a: Int32 = -1;
  var b: UInt32 = a as UInt32;
  if b == 4294967295 as UInt32 {
    return 0;
  }
  return 1;
}
