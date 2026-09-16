// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 neg of min (-(-2147483648) wraps)
fn main() -> Int {
  var a: Int32 = -2147483648 as Int32;
  var b: Int32 = -a;
  if b == -2147483648 as Int32 { return 0; }
  return 1;
}
