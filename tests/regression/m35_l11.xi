// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L11: Enum memory pattern -- verify enum tag + payload memory layout
enum Shape { Circle(r: Float64), Rect(w: Int, h: Int), Point }

fn main() -> Int {
  var c = Shape.Circle(5.0);
  match c {
    Shape.Circle(r) => if r != 5.0 { return 1; },
    _ => { return 2; },
  }
  var r = Shape.Rect(3, 4);
  match r {
    Shape.Rect(w, h) => if w + h != 7 { return 3; },
    _ => { return 4; },
  }
  var p = Shape.Point;
  match p {
    Shape.Point => return 0,
    _ => { return 5; },
  }
}
