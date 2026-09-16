// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0031

type Point = { x: Int; y: Int; }

interface Locatable {
  fn default_point(&self) -> Point { return Point{ x: 0, y: 0 }; }
  fn current(&self) -> Point;
}

type Tracker = { pos: Point; }

fn Tracker.current(&self) -> Point { return pos; }

fn main() -> Int {
  var t: Tracker = Tracker{ pos: Point{ x: 5, y: 10 } };
  var cur = t.current();
  var def = t.default_point();
  if cur.x == 5 && cur.y == 10 && def.x == 0 && def.y == 0 { return 0; }
  return 1;
}
