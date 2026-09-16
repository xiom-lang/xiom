// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Multi-type arithmetic with hex, neg, shift
fn main() -> Int {
  var a: Int = 0x40;
  var b: Int = -8;
  var c: Int = a >> 2;
  var d: Int = c + b;
  if d == 8 {
    return 0;
  }
  return 1;
}
