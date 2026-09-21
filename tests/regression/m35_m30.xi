// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M30: quadratic discriminant + distance between two points
fn discriminant(a: Float64, b: Float64, c: Float64) -> Float64 {
  return b * b - 4.0 * a * c;
}
fn sqrt_newton(x: Float64) -> Float64 {
  if x <= 0.0 { return 0.0; }
  var guess: Float64 = x;
  var i: Int = 0;
  while i < 20 {
    guess = (guess + x / guess) * 0.5;
    i = i + 1;
  }
  return guess;
}
fn distance(x1: Float64, y1: Float64, x2: Float64, y2: Float64) -> Float64 {
  var dx = x2 - x1;
  var dy = y2 - y1;
  return sqrt_newton(dx * dx + dy * dy);
}
fn main() -> Int {
  var d1: Float64 = discriminant(1.0, -3.0, 2.0);
  var dist: Float64 = distance(0.0, 0.0, 3.0, 4.0);
  if d1 > 0.99 && d1 < 1.01 && dist > 4.99 && dist < 5.01 { return 0; }
  return 1;
}
