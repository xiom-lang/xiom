// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Recursive function with narrow int parameter (simple countdown)
fn countdown_u8(n: UInt8) -> UInt8 {
  if n == 0 as UInt8 { return 0; }
  return 1 as UInt8 + countdown_u8(n - 1 as UInt8);
}
fn main() -> Int {
  var r: UInt8 = countdown_u8(5);
  if r == 5 as UInt8 { return 0; }
  return 1;
}
