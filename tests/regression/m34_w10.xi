// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W10: Bit counting -- popcount via while loop
fn popcount(x: Int) -> Int {
  var n: Int = x;
  var count: Int = 0;
  while n != 0 {
    count = count + (n & 1);
    n = n >> 1;
  }
  return count;
}
fn main() -> Int {
  if popcount(0) != 0 { return 1; }
  if popcount(1) != 1 { return 2; }
  if popcount(7) != 3 { return 3; }
  if popcount(0xFF) != 8 { return 4; }
  if popcount(0xAAAA) != 8 { return 5; }
  return 0;
}
