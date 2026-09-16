// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A29: Count set bits -- Brian Kernighan's algorithm and loop-based popcount
fn popcount_kernighan(n: Int) -> Int {
  var x: Int = n;
  var c: Int = 0;
  while x != 0 {
    x = x & (x - 1);
    c = c + 1;
  }
  return c;
}
fn popcount_loop(n: Int) -> Int {
  var x: Int = n;
  var c: Int = 0;
  while x > 0 {
    c = c + (x & 1);
    x = x >> 1;
  }
  return c;
}
fn main() -> Int {
  var i: Int = 0;
  while i < 256 {
    if popcount_kernighan(i) != popcount_loop(i) { return 1; }
    i = i + 1;
  }
  if popcount_kernighan(0xF) != 4 { return 2; }
  if popcount_kernighan(0xFF) != 8 { return 3; }
  if popcount_kernighan(1023) != 10 { return 4; }
  return 0;
}
