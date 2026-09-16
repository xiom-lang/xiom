// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 roundtrip UInt32 -> Int64 -> UInt32
fn main() -> Int {
  var a: UInt32 = 3000000000;
  var b: Int64 = a as Int64;
  var c: UInt32 = b as UInt32;
  if c == a { return 0; }
  return 1;
}
