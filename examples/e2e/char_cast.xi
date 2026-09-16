// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E regression: char/byte integer casts and widening in arithmetic.
// Locks in Char (i8) <-> Int (i64) casts and i8 widening in binops.
// Returns 0 on success.
module e2e_char_cast

fn main() -> Int {
  let c: Char = 'A';          // 65
  let n: Int = c as Int;      // sext i8 -> i64
  if n != 65 {
    return 1;
  }
  // Digit char to numeric value via arithmetic on a widened i8.
  let d: Char = '7';
  let dv: Int = (d as Int) - 48;
  if dv != 7 {
    return 2;
  }
  // Int -> Char (trunc) round-trip.
  let back: Char = n as Char;
  if (back as Int) != 65 {
    return 3;
  }
  return 0;
}
