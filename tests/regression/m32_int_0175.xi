// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 div at max
fn main() -> Int {
  var a: UInt32 = 4294967295;
  var b: UInt32 = 1;
  var c: UInt32 = a / b;
  if c == a { return 0; }
  return 1;
}
