// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Right shift on UInt32 (logical for unsigned)
fn main() -> Int {
  var a: UInt32 = 2147483648;
  var result: UInt32 = a >> 31;
  if result == 1 as UInt32 {
    return 0;
  }
  return 1;
}
