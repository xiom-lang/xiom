// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_005
type Coords = { x: Float64; y: Float64; }

  pub fn run() -> Int {
    var c: Coords = { x: 1.5; y: 2.5; };
    var x = c.x;
    var y = c.y;
    if x == 1.5 && y == 2.5 { return 0; }
    return 1;
  }
use m21_destructure_005.run;
fn main() -> Int { return run(); }
