// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W12: Bit set, clear, and toggle patterns on Int
fn main() -> Int {
  var x: Int = 0;
  // Set bit 3: x | (1 << 3)
  x = x | (1 << 3);
  if x != 8 { return 1; }
  // Set bit 0: x | (1 << 0)
  x = x | (1 << 0);
  if x != 9 { return 2; }
  // Clear bit 3: x & ~(1 << 3)
  x = x & ~(1 << 3);
  if x != 1 { return 3; }
  // Toggle bit 1: x ^ (1 << 1)
  x = x ^ (1 << 1);
  if x != 3 { return 4; }
  // Toggle back
  x = x ^ (1 << 1);
  if x != 1 { return 5; }
  // Test multiple bits: set bits 4 and 5
  x = x | (1 << 4) | (1 << 5);
  if x != 49 { return 6; }
  // Clear bit 4
  x = x & ~(1 << 4);
  if x != 33 { return 7; }
  return 0;
}
