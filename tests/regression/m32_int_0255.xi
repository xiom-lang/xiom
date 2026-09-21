// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(129) as Int should be 129, not -127
fn main() -> Int {
  var a: UInt8 = 129;
  var b: Int = a as Int;
  if b == 129 { return 0; }
  return 1;
}
