// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum Shape { Circle(radius: Int), Square(side: Int), Triangle(base: Int, height: Int) }
fn area(s: Shape) -> Int {
  match s {
    Circle(r) => r * r * 3,
    Square(side) => side * side,
    Triangle(b, h) => b * h / 2,
  }
}
fn main() -> Int {
  if area(Shape.Circle(5)) != 75 { return 1; }
  if area(Shape.Square(4)) != 16 { return 2; }
  if area(Shape.Triangle(6, 8)) != 24 { return 3; }
  return 0;
}
