// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C26: Every int pattern -- add, sub, mul, div, mod, bitwise AND/OR/XOR/NOT, shift left/right, cmp, cast, negate
fn int_add(a: Int, b: Int) -> Int { return a + b; }
fn int_sub(a: Int, b: Int) -> Int { return a - b; }
fn int_mul(a: Int, b: Int) -> Int { return a * b; }
fn int_div(a: Int, b: Int) -> Int { return a / b; }
fn int_mod(a: Int, b: Int) -> Int { return a % b; }
fn int_and(a: Int, b: Int) -> Int { return a & b; }
fn int_or(a: Int, b: Int) -> Int { return a | b; }
fn int_xor(a: Int, b: Int) -> Int { return a ^ b; }
fn int_not(a: Int) -> Int { return ~a; }
fn int_shl(a: Int, n: Int) -> Int { return a << n; }
fn int_shr(a: Int, n: Int) -> Int { return a >> n; }
fn int_neg(a: Int) -> Int { return -a; }
fn int_cmp(a: Int, b: Int) -> Int {
  if a < b { return -1; }
  if a > b { return 1; }
  return 0;
}
fn int_cast_to_f64(a: Int) -> Float64 { return a as Float64; }
fn int_cast_to_i8(a: Int) -> Int8 { return a as Int8; }
fn main() -> Int {
  if int_add(40, 2) != 42 { return 1; }
  if int_sub(50, 8) != 42 { return 2; }
  if int_mul(6, 7) != 42 { return 3; }
  if int_div(84, 2) != 42 { return 4; }
  if int_mod(85, 43) != 42 { return 5; }
  if int_mod(10, 3) != 1 { return 6; }
  if int_and(0xFF, 0x0F) != 15 { return 7; }
  if int_or(0xF0, 0x0F) != 255 { return 8; }
  if int_xor(0xFF, 0x0F) != 240 { return 9; }
  if int_shl(1, 5) != 32 { return 10; }
  if int_shl(3, 2) != 12 { return 11; }
  if int_shr(64, 2) != 16 { return 12; }
  if int_shr(8, 1) != 4 { return 13; }
  if int_neg(42) != -42 { return 14; }
  if int_neg(-42) != 42 { return 15; }
  if int_neg(0) != 0 { return 16; }
  if int_cmp(10, 20) != -1 { return 17; }
  if int_cmp(20, 10) != 1 { return 18; }
  if int_cmp(15, 15) != 0 { return 19; }
  var fv = int_cast_to_f64(7);
  if fv < 6.99 || fv > 7.01 { return 20; }
  if int_cast_to_i8(100) != 100 { return 21; }
  var not_val = int_not(0);
  if not_val != -1 { return 22; }
  return 0;
}
