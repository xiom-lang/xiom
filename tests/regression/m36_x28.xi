// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X28: Integer overflow edges -- boundary values and edge cases
fn main() -> Int {
  var max: Int = 2147483647;
  var min: Int = -2147483647 - 1;
  if max < 0 { return 1; }
  if min > 0 { return 2; }
  var zero: Int = 0;
  var one: Int = 1;
  var neg_one: Int = -1;
  if zero + one != 1 { return 3; }
  if zero - one != -1 { return 4; }
  if neg_one + one != 0 { return 5; }
  var near_max: Int = max - 1;
  if near_max + 1 != max { return 6; }
  var near_min: Int = min + 1;
  if near_min - 1 != min { return 7; }
  var small: Int = 100;
  var big: Int = 100000;
  var product: Int = small * big;
  if product != 10000000 { return 8; }
  var quotient: Int = product / small;
  if quotient != big { return 9; }
  var remainder: Int = product - quotient * small;
  if remainder != 0 { return 10; }
  if max - max != 0 { return 11; }
  if min - min != 0 { return 12; }
  var a: Int = -5;
  var abs_a: Int = a;
  if a < 0 { abs_a = 0 - a; }
  if abs_a != 5 { return 13; }
  return 0;
}
