// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C03: Requires + ensures -- division with both checks
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures: result * b == a
{
  return a / b;
}
fn main() -> Int {
  var r: Float64 = divide(15.0, 3.0);
  if r == 5.0 { return 0; }
  return 1;
}
