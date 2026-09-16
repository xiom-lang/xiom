// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed sign cast Int16 to UInt16 (-1 -> 65535)
fn main() -> Int {
  var a: Int16 = -1 as Int16;
  var b: UInt16 = a as UInt16;
  if b == 65535 as UInt16 { return 0; }
  return 1;
}
