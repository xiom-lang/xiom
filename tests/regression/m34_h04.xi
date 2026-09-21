// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H04: Int->UInt (signed to unsigned) -- positive values preserve identity
fn main() -> Int {
  var s: Int8 = 100;
  var u: UInt8 = s as UInt8;
  if u == 100 as UInt8 { return 0; }
  return 1;
}
