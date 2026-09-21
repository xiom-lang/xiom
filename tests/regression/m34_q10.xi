// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q10: Contract chain for bounds-guarded data access
fn check_lower(i: Int) -> Bool { return i >= 0; }
fn check_upper(i: Int) -> Bool { return i < 4; }

fn is_valid_index(i: Int) -> Bool
  ensures: check_lower(i) && check_upper(i) || !(check_lower(i) && check_upper(i))
{
  if i >= 0 && i < 4 { return true; }
  return false;
}
fn triple_index(i: Int) -> Int
  requires: is_valid_index(i)
  ensures: result >= 0
{
  return i * 3;
}
fn sum_indices(a: Int, b: Int) -> Int
  requires: is_valid_index(a)
  requires: is_valid_index(b)
  ensures: result >= 0
{
  return triple_index(a) + triple_index(b);
}
fn main() -> Int {
  var data = [10, 20, 30, 40];
  var i1 = sum_indices(0, 1);
  var i2 = triple_index(2);
  var i3 = data[i2 / 3];
  if i1 == 3 && i2 == 6 && i3 == 30 { return 0; }
  return 1;
}
