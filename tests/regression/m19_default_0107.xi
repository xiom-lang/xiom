// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0107

interface Origin {
  fn origin(&self) -> Point { return Point{ x: 0, y: 0 }; }
  fn position(&self) -> Point;
}

type Point = { x: Int; y: Int; }

fn Point.position(&self) -> Point { return Point{ x: x, y: y }; }

fn main() -> Int {
  var p: Point = Point{ x: 5, y: 7 };
  var o = p.origin();
  if o.x != 0 { return 1; }
  if o.y != 0 { return 2; }
  var pos = p.position();
  if pos.x != 5 { return 3; }
  if pos.y != 7 { return 4; }
  return 0;
}
