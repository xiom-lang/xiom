// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 at max boundary (255 + 1 = 0)
fn main() -> Int {
  var max: UInt8 = 255;
  var x: UInt8 = max + 1 as UInt8;
  if x == 0 as UInt8 { return 0; }
  return 1;
}
