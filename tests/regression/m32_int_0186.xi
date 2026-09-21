// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt64 comparisons (safe literal range)
fn main() -> Int {
  var a: UInt64 = 0;
  var b: UInt64 = 1000000 as UInt64;
  if a <= b && b >= a && a != b { return 0; }
  return 1;
}
