// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 -- zero identity
fn main() -> Int {
  var a: Int16 = 12345;
  var b: Int16 = 0;
  var add: Int16 = a + b;
  var sub: Int16 = a - b;
  var mul: Int16 = a * b;
  if add == 12345 as Int16 && sub == 12345 as Int16 && mul == 0 as Int16 {
    return 0;
  }
  return 1;
}
