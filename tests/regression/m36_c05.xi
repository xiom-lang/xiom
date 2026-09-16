// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C05: Every contract pattern with every function shape -- requires, ensures, invariant, multi-clause
type PosNum = { val: Int; invariant: val > 0; }
type Bounded = { x: Int; lo: Int; hi: Int; invariant: lo <= hi; invariant: x >= lo; invariant: x <= hi; }
fn safe_add(a: Int, b: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  ensures: result >= a
  ensures: result >= b
{ return a + b; }
fn safe_mul(a: Int, b: Int) -> Int
  requires: a >= 0
  ensures: result >= 0
{ if a == 0 { return 0; } return a * b; }
fn clamp(x: Int, lo: Int, hi: Int) -> Int
  requires: lo <= hi
  ensures: result >= lo
  ensures: result <= hi
{
  if x < lo { return lo; }
  if x > hi { return hi; }
  return x;
}
fn abs_val(n: Int) -> Int
  ensures: result >= 0
{
  if n < 0 { return -n; }
  return n;
}
fn zero_or_pos(n: Int) -> Int
  ensures: result >= 0
{
  if n < 0 { return -n; }
  return n;
}
fn main() -> Int {
  if safe_add(3, 5) != 8 { return 1; }
  if safe_add(0, 7) != 7 { return 2; }
  if safe_mul(5, 3) != 15 { return 3; }
  if safe_mul(0, 999) != 0 { return 4; }
  if clamp(50, 0, 100) != 50 { return 5; }
  if clamp(-5, 0, 100) != 0 { return 6; }
  if clamp(200, 0, 100) != 100 { return 7; }
  if clamp(100, 0, 100) != 100 { return 8; }
  if clamp(0, 0, 100) != 0 { return 9; }
  if abs_val(-42) != 42 { return 10; }
  if abs_val(42) != 42 { return 11; }
  if abs_val(0) != 0 { return 12; }
  if zero_or_pos(-99) != 99 { return 13; }
  if zero_or_pos(0) != 0 { return 14; }
  return 0;
}
