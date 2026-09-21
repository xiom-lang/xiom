// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0078

enum Shape { Circle(r: Float64), Square(s: Float64) }

fn main() -> Int {
  var s: Shape = Circle(2.5);
  match s {
    Circle(r) | Square(r) if r > 1.0 => { return 0; }
    _ => { return 1; }
  }
}
