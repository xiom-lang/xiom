// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C23: match on enum -- dispatch through enum variants with payloads
enum Color { Red, Green, Blue }
enum Shape { Circle(r: Int), Square(s: Int), Point }
fn area(s: Shape) -> Int {
  match s {
    Circle(r) => r * r * 3,
    Square(s) => s * s,
    Point => 0,
  }
}
fn rgb_value(c: Color) -> Int {
  match c {
    Red => 1,
    Green => 2,
    Blue => 3,
  }
}
fn main() -> Int {
  var c1 = Shape.Circle(5);
  var c2 = Shape.Square(4);
  var c3 = Shape.Point;
  if area(c1) != 75 { return 1; }
  if area(c2) != 16 { return 2; }
  if area(c3) != 0 { return 3; }
  if rgb_value(Color.Red) != 1 { return 4; }
  if rgb_value(Color.Green) != 2 { return 5; }
  if rgb_value(Color.Blue) != 3 { return 6; }
  return 0;
}
