// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 negation unsigned wraparound (0 - 1 = 65535)
fn main() -> Int {
  var a: UInt16 = 1;
  var b: UInt16 = 0 - a;
  if b == 65535 as UInt16 { return 0; }
  return 1;
}
