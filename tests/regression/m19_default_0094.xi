// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0094

interface Describable {
  fn desc(&self) -> Str { return name() + ":"; }
  fn name(&self) -> Str;
  fn area(&self) -> Float64;
}

enum Shape {
  Circle(r: Float64),
  Square(s: Float64)
}

fn Shape.name(&self) -> Str {
  match self {
    Circle(_) => "circle",
    Square(_) => "square"
  }
}

fn Shape.area(&self) -> Float64 {
  match self {
    Circle(r) => 3.14 * r * r,
    Square(s) => s * s
  }
}

fn main() -> Int {
  var c: Shape = Shape.Circle{ r: 5.0 };
  var s: Shape = Shape.Square{ s: 3.0 };
  if c.name() != "circle" { return 1; }
  if c.desc() != "circle:" { return 2; }
  if s.name() != "square" { return 3; }
  if s.desc() != "square:" { return 4; }
  return 0;
}
