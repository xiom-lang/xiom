// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C07: Float contract -- Float64 precision guard
fn scale(value: Float64, factor: Float64) -> Float64
  requires: factor != 0.0
  ensures: result / factor == value
{
  return value * factor;
}
fn main() -> Int {
  var r: Float64 = scale(3.0, 2.5);
  if r == 7.5 { return 0; }
  return 1;
}
