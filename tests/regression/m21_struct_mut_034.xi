// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_034
type Point = { x: Int; y: Int; }

  fn make_point(a: Int, b: Int) -> Point {
    return { x: a; y: b; };
  }

pub fn run() -> Int {
    var p: Point = make_point(1, 2);
    p.x = 99;
    if p.x == 99 && p.y == 2 { return 0; }
    return 1;
  }
use m21_struct_mut_034.run;
fn main() -> Int { return run(); }
