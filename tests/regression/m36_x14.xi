// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X14: All type casts -- comprehensive cast operations between types
fn main() -> Int {
  var i: Int = 42;
  var f: Float64 = i as Float64;
  if f != 42.0 { return 1; }
  var d: Float64 = 3.14159;
  var j: Int = d as Int;
  if j != 3 { return 2; }
  var k: Int = 65;
  var c: Char = k as Char;
  if k < 60 || k > 90 { return 3; }
  var big: Int = 1000;
  var small: Int = big as Int;
  if small != 1000 { return 4; }
  var small_f: Float64 = 1.5;
  var small_d: Float64 = small_f as Float64;
  if small_d != 1.5 { return 5; }
  var x: Int = 7;
  var y: Float64 = x as Float64;
  var z: Int = y as Int;
  if z != 7 { return 6; }
  var neg: Float64 = -3.0;
  var neg_i: Int = neg as Int;
  if neg_i != -3 { return 7; }
  return 0;
}
