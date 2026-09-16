// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 add wraparound (2147483647 + 1 = -2147483648)
fn main() -> Int {
  var a: Int32 = 2147483647;
  var b: Int32 = 1;
  var c: Int32 = a + b;
  if c == -2147483648 as Int32 { return 0; }
  return 1;
}
