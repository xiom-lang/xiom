// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_011
pub type Point = { x: Int; y: Int; }
  pub type Size = { w: Int; h: Int; }

  pub fn make_point(x: Int, y: Int) -> Point { return { x: x; y: y; }; }
  pub fn make_size(w: Int, h: Int) -> Size { return { w: w; h: h; }; }

  pub fn run() -> Int {
    var p = make_point(3, 4);
    var s = make_size(100, 200);
    if p.x == 3 && s.w == 100 { return 0; }
    return 1;
  }
use m21_module_011.run;
fn main() -> Int { return run(); }
