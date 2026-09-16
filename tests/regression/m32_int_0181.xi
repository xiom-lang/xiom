// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt64 sub wraparound (0 - 1 = max)
fn main() -> Int {
  var a: UInt64 = 0;
  var b: UInt64 = 1;
  var c: UInt64 = a - b;
  if c == 18446744073709551615 as UInt64 { return 0; }
  return 1;
}
