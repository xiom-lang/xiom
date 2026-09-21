// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C05: Multiple ensures -- range guarantees on clamp
fn clamp(val: Int, lo: Int, hi: Int) -> Int
  ensures: result >= lo
  ensures: result <= hi
  ensures: result == val || result == lo || result == hi
{
  if val < lo { return lo; }
  if val > hi { return hi; }
  return val;
}
fn main() -> Int {
  var a: Int = clamp(5, 0, 10);
  var b: Int = clamp(-2, 0, 10);
  var c: Int = clamp(15, 0, 10);
  if a == 5 && b == 0 && c == 10 { return 0; }
  return 1;
}
