// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0103

interface Area {
  fn circle_area(&self) -> Float64 { return 3.14159 * radius() * radius(); }
  fn radius(&self) -> Float64;
}

type Circle = { r: Float64; }

fn Circle.circle_area(self) -> Float64 { return 3.14159 * self.radius() * self.radius(); }


fn Circle.radius(&self) -> Float64 { return r; }

fn main() -> Int {
  var c: Circle = Circle{ r: 2.0 };
  var area = c.circle_area();
  if area > 12.5 && area < 12.6 { return 0; }
  return 1;
}
