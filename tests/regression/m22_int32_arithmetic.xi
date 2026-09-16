// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Int32 arithmetic operations
fn main() -> Int {
  var a: Int32 = 100;
  var b: Int32 = 200;
  var add: Int32 = a + b;
  var sub: Int32 = b - a;
  var mul: Int32 = a * 2;
  var div: Int32 = b / 2;
  if add == 300 as Int32 && sub == 100 as Int32 && mul == 200 as Int32 && div == 100 as Int32 {
    return 0;
  }
  return 1;
}
