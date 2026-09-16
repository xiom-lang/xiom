// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 multiplication
fn main() -> Int {
  var a: Int8 = 10;
  var b: Int8 = 11;
  var prod: Int8 = a * b;
  if prod == 110 as Int8 {
    return 0;
  }
  return 1;
}
