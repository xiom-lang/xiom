// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(128) > Int8(127) should be true (zext needed)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: Int8 = 127;
  var ai: Int = a as Int;
  var bi: Int = b as Int;
  if ai > bi { return 0; }
  return 1;
}
