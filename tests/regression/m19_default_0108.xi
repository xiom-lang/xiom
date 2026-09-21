// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0108

interface Midpoint {
  fn midpoint(&self) -> Point { var a = start(); var b = end(); return Point{ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }; }
  fn start(&self) -> Point;
  fn end(&self) -> Point;
}

type Point = { x: Int; y: Int; }

type Segment = { s: Point; e: Point; }

fn Segment.start(&self) -> Point { return s; }
fn Segment.end(&self) -> Point { return e; }

fn main() -> Int {
  var seg: Segment = Segment{ s: Point{ x: 0, y: 0 }, e: Point{ x: 10, y: 20 } };
  var m = seg.midpoint();
  if m.x != 5 { return 1; }
  if m.y != 10 { return 2; }
  return 0;
}
