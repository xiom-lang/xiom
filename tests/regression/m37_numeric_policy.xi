// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_numeric_policy
// BUG 26 (secure numeric policy): INT <-> FLOAT mixing requires an explicit
// `as` cast. Auto-widening stays for same-family; INT LITERALS may adopt
// the float type (exact); FLOAT literals never adopt an integer type.
// (The rejected cases are compile errors -- verified manually; this test
// covers the ALLOWED surface.)

fn main() -> Int {
  var d = 2.5;
  // int literal adopts float (exact)
  var x = d * 2;
  if x != 5.0 { return 1; }
  if d > 0 { } else { return 2; }
  // int literal to float binding
  var f: Float64 = 5;
  if f != 5.0 { return 3; }
  // int widening stays automatic
  var big: Int64 = 10;
  var s: Int32 = 3;
  if s + big != 13 { return 4; }
  // explicit `as` is the sanctioned conversion
  var i = 7;
  var conv = i as Float64;
  if conv != 7.0 { return 5; }
  var back = d as Int;
  if back != 2 { return 6; }
  // same-family float arithmetic with int literals
  var acc = 0.0;
  acc = acc + 1;
  acc = acc * 2;
  if acc != 2.0 { return 7; }
  return 0;
}
