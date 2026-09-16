// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt maximum value -- wraps on overflow
fn main() -> Int {
  var a: UInt = 18446744073709551615;
  var b: UInt = 1;
  var sum: UInt = a + b;
  if sum == 0 {
    return 0;
  }
  return 1;
}
