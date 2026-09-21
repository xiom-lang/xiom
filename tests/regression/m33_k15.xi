// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K15: Closure in match arm -- enum dispatch via closure
type Op = enum { Add(a: Int, b: Int), Mul(a: Int, b: Int), }
fn main() -> Int {
  var op = Op.Add(3, 4);
  if match op { Add(a, b) => a + b; Mul(a, b) => a * b; } != 7 { return 1; }
  return 0;
}
