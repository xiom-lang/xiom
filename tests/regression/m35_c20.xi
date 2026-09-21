// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C20: if-expression -- let binding with if expression value
fn abs(x: Int) -> Int {
  var v: Int = if x >= 0 { x } else { -x };
  return v;
}
fn sign(x: Int) -> Int {
  var v: Int = if x > 0 { 1 } else { 0 };
  return v;
}
fn main() -> Int {
  var a: Int = if true { 42 } else { 0 };
  if a != 42 { return 1; }
  var b: Int = if 5 > 10 { 1 } else { 99 };
  if b != 99 { return 2; }
  if abs(-7) != 7 { return 3; }
  if abs(3) != 3 { return 4; }
  if sign(5) != 1 { return 5; }
  if sign(-5) != 0 { return 6; }
  return 0;
}
