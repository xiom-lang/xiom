// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W18: Bitwise compound patterns -- sequential & | ^ operations
fn main() -> Int {
  var x: Int = 0x0F;
  var y: Int = 0xF0;
  var z: Int = 0x55;
  var a: Int = x & y;
  if a != 0 { return 1; }
  var b: Int = x | y;
  if b != 0xFF { return 2; }
  var c: Int = x ^ y;
  if c != 0xFF { return 3; }
  // ((x & z) | y) ^ z = ((0x0F & 0x55) | 0xF0) ^ 0x55 = (0x05 | 0xF0) ^ 0x55 = 0xF5 ^ 0x55 = 0xA0
  var chain: Int = ((x & z) | y) ^ z;
  if chain != 0xA0 { return 4; }
  var acc: Int = 1;
  acc = acc | (1 << 3);
  acc = acc & ~(1 << 0);
  acc = acc ^ (1 << 2);
  if acc != 12 { return 5; }
  return 0;
}
