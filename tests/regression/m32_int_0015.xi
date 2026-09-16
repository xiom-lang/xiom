// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 multiplication
fn main() -> Int {
  var a: Int16 = 100;
  var b: Int16 = 300;
  var prod: Int16 = a * b;
  if prod == 30000 as Int16 {
    return 0;
  }
  return 1;
}
