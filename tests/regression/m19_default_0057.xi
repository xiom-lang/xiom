// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0057

type Vec = { x: Int; y: Int; }

interface Positioned {
  fn origin(&self) -> Vec { return Vec{ x: 0, y: 0 }; }
  fn position(&self) -> Vec;
}

type Entity = { pos: Vec; }

fn Entity.position(&self) -> Vec { return pos; }

fn main() -> Int {
  var e: Entity = Entity{ pos: Vec{ x: 5, y: 10 } };
  var o = e.origin();
  var p = e.position();
  if o.x == 0 && o.y == 0 && p.x == 5 && p.y == 10 { return 0; }
  return 1;
}
