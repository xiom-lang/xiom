// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 arithmetic vs UInt8 logical right shift difference
fn main() -> Int {
  var s: Int8 = -128 as Int8;
  var u: UInt8 = 128;
  var ss: Int8 = s >> 1;
  var uu: UInt8 = u >> 1;
  if ss == -64 as Int8 && uu == 64 as UInt8 { return 0; }
  return 1;
}
