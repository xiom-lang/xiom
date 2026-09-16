// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 narrowing to Int16 (high bits dropped, sign preserved)
fn main() -> Int {
  var a: Int32 = 32767;
  var b: Int16 = a as Int16;
  if b == 32767 { return 0; }
  return 1;
}
