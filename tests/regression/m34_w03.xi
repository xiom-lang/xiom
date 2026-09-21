// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W03: XOR self -- x ^ x == 0 on multiple types
fn main() -> Int {
  var a: Int = 12345;
  var b: Int32 = 12345 as Int32;
  var c: UInt = 12345;
  var r1: Int = a ^ a;
  var r2: Int32 = b ^ b;
  var r3: UInt = c ^ c;
  if r1 == 0 && r2 == 0 as Int32 && r3 == 0 { return 0; }
  return 1;
}
