// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X19: Const folding -- constant expression evaluation at compile time
const MAX: Int = 100;
const HALF: Int = MAX / 2;
const DOUBLE: Int = HALF * 4;
const OFFSET: Int = 10;
const THRESHOLD: Int = HALF + OFFSET;
const PI: Float64 = 3.14159;
const RADIUS: Float64 = PI * 2.0;
const FIVE: Int = 5;
const FACT: Int = FIVE * 4 * 3 * 2 * 1;
const ARRAY_SIZE: Int = 32;
const INDEX: Int = ARRAY_SIZE / 8;
fn use_consts() -> Int {
  var s: Int = MAX + HALF + DOUBLE + OFFSET + THRESHOLD;
  return s;
}
fn main() -> Int {
  if MAX != 100 { return 1; }
  if HALF != 50 { return 2; }
  if DOUBLE != 200 { return 3; }
  if OFFSET != 10 { return 4; }
  if THRESHOLD != 60 { return 5; }
  if FACT != 120 { return 6; }
  if INDEX != 4 { return 7; }
  if use_consts() != 420 { return 8; }
  var arr_size: Int = ARRAY_SIZE;
  if arr_size != 32 { return 9; }
  return 0;
}
