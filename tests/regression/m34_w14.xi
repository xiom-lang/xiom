// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W14: Bit rotation -- rotl/rotr on Int using shift + OR
fn rotl8(x: Int) -> Int {
  var hi: Int = (x >> 56) & 0xFF;
  return ((x << 8) & (0xFFFFFFFFFFFFFF00 as Int)) | hi;
}
fn rotr8(x: Int) -> Int {
  var lo: Int = (x & 0xFF) << 56;
  return (x >> 8) | lo;
}
fn main() -> Int {
  var a: Int = 0x0102030405060708;
  var rl: Int = rotl8(a);
  var rr: Int = rotr8(rl);
  if rr == a { return 0; }
  return 1;
}
