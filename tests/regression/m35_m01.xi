// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M01: sqrt via Newton method (Float64)
fn sqrt_newton(x: Float64) -> Float64 {
  if x <= 0.0 { return 0.0; }
  var guess: Float64 = x;
  var i: Int = 0;
  while i < 30 {
    guess = (guess + x / guess) * 0.5;
    i = i + 1;
  }
  return guess;
}
fn main() -> Int {
  var r25: Float64 = sqrt_newton(25.0);
  var r2: Float64 = sqrt_newton(2.0);
  var r100: Float64 = sqrt_newton(100.0);
  if r25 > 4.99 && r25 < 5.01 && r2 > 1.41 && r2 < 1.42 && r100 > 9.99 && r100 < 10.01 { return 0; }
  return 1;
}
