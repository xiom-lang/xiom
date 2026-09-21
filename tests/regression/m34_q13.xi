// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q13: Contract with boolean precondition -- chained Bool requires
fn is_even(x: Int) -> Bool
  ensures: result || !result
{
  return x % 2 == 0;
}
fn is_positive(x: Int) -> Bool
  ensures: result || !result
{
  return x > 0;
}
fn calc_even(x: Int) -> Int
  requires: is_even(x)
  ensures: result == x / 2
{
  return x / 2;
}
fn calc_positive_even(x: Int) -> Int
  requires: is_positive(x)
  requires: is_even(x)
  ensures: result > 0
{
  var half = calc_even(x);
  return half;
}
fn process_bool(x: Int) -> Int
  requires: is_positive(x)
  requires: is_even(x)
  ensures: result < x
{
  var half = calc_positive_even(x);
  return half + 1;
}
fn main() -> Int {
  var r1 = calc_even(20);
  var r2 = calc_positive_even(16);
  var r3 = process_bool(24);
  if r1 == 10 && r2 == 8 && r3 == 13 { return 0; }
  return 1;
}
