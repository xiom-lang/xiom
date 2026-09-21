// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V13: Vec of struct -- Vec[Point] index access
use xiom.collections;

type Point = { x: Int; y: Int; }

fn main() -> Int {
  var v = Vec[Point].new();
  var p1 = Point { x: 1; y: 2; };
  var p2 = Point { x: 3; y: 4; };
  v.push(p1);
  v.push(p2);
  if v.len() != 2 { return 1; }
  var pt0 = v[0];
  if pt0.x != 1 { return 2; }
  if pt0.y != 2 { return 3; }
  var pt1 = v[1];
  if pt1.x != 3 { return 4; }
  if pt1.y != 4 { return 5; }
  // Remove first element
  v.remove(0);
  if v.len() != 1 { return 6; }
  var pt_rem = v[0];
  if pt_rem.x != 3 { return 7; }
  if pt_rem.y != 4 { return 8; }
  return 0;
}
