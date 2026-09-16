// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 mid-range values: 128-255 casting to wider and back
fn main() -> Int {
  var a: UInt8 = 200;
  var b: Int32 = a as Int32;
  var c: UInt8 = b as UInt8;
  if c == a { return 0; }
  return 1;
}
