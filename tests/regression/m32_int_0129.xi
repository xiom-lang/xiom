// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 mul wraparound (1073741824 * 2 = -2147483648)
fn main() -> Int {
  var a: Int32 = 1073741824;
  var b: Int32 = 2;
  var c: Int32 = a * b;
  if c == -2147483648 as Int32 { return 0; }
  return 1;
}
