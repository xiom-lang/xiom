// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Minimal repro: enum with data variants + this-based method + match
module tests.ecosystem.test_enum_this_match

pub enum Color {
  Red,
  Green,
  Blue(val: Int),
}

fn Color.is_red() -> Bool {
  match this {
    Red => { return true; }
    _ => { return false; }
  }
}

fn Color.is_blue() -> Bool {
  match this {
    Blue(_) => { return true; }
    _ => { return false; }
  }
}

fn Color.get_blue_val() -> Option[Int] {
  match this {
    Blue(val) => { return Some(val); }
    _ => { return None; }
  }
}

fn main() -> Int {
  var passed = 0; var total = 0;

  // Test 1: is_red on Red variant
  total = total + 1;
  var r = Color.Red;
  if Color.is_red(&r) { passed = passed + 1; }

  // Test 2: is_red on non-Red
  total = total + 1;
  var g = Color.Green;
  if !Color.is_red(&g) { passed = passed + 1; }

  // Test 3: is_blue on Blue variant with data
  total = total + 1;
  var b = Color.Blue(42);
  if Color.is_blue(&b) { passed = passed + 1; }

  // Test 4: get_blue_val
  total = total + 1;
  var b2 = Color.Blue(99);
  let v = Color.get_blue_val(&b2);
  if v.is_some() && v.unwrap() == 99 { passed = passed + 1; }

  // Test 5: multiple matches
  total = total + 1;
  var b3 = Color.Blue(7);
  if Color.is_blue(&b3) && !Color.is_red(&b3) { passed = passed + 1; }

  // Test 6: is_red with Blue variant
  total = total + 1;
  var b4 = Color.Blue(1);
  if !Color.is_red(&b4) { passed = passed + 1; }

  // Test 7: match on non-Blue
  total = total + 1;
  var r2 = Color.Red;
  let v2 = Color.get_blue_val(&r2);
  if v2.is_none() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
