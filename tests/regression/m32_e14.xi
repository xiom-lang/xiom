// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E14: Exhaustive match with wildcard catch-all and mixed variants
enum Shape { Circle(radius: Float64), Square(side: Float64), Triangle(base: Float64, height: Float64) }
fn area(s: Shape) -> Float64 {
  match s {
    Circle(r) => 3.14159 * r * r,
    Square(side) => side * side,
    Triangle(b, h) => 0.5 * b * h,
  }
}
fn main() -> Int {
  var a1 = area(Shape.Circle(1.0));
  var a2 = area(Shape.Square(2.0));
  var a3 = area(Shape.Triangle(3.0, 4.0));
  var diff1 = a1 - 3.14159;
  if diff1 < 0.0 { diff1 = -diff1; }
  if diff1 > 0.01 { return 1; }
  if a2 != 4.0 { return 2; }
  if a3 != 6.0 { return 3; }
  return 0;
}
