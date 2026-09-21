// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_015
enum Shape {
    Circle(radius: Float64),
    Rect(w: Float64, h: Float64),
    Line(x1: Float64, y1: Float64, x2: Float64, y2: Float64),
  }

  pub fn run() -> Int {
    var s = Shape.Circle(5.0);
    match s {
      Shape.Circle(r) => if r == 5.0 { return 0; },
      Shape.Rect(_, _) => return 1,
      Shape.Line(_, _, _, _) => return 1,
    }
  }
use m21_match_edge_015.run;
fn main() -> Int { return run(); }
