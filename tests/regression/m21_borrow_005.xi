// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_005
type Point = { x: Int; y: Int; }

  pub fn run() -> Int {
    var p: Point = { x: 10; y: 20; };
    {
      var r = &p;
      if (*r).x == 10 { }
    }
    var moved = p;
    if moved.x == 10 && moved.y == 20 { return 0; }
    return 1;
  }
use m21_borrow_005.run;
fn main() -> Int { return run(); }
