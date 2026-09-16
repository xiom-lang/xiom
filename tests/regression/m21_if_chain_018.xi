// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_018
type Point = { x: Int; y: Int; }

  fn is_origin(p: Point) -> Bool {
    return p.x == 0 && p.y == 0;
  }

  pub fn run() -> Int {
    var p: Point = { x: 0; y: 0; };
    if is_origin(p) { return 0; }
    elif p.x > 0 { return 1; }
    else { return 1; }
  }
use m21_if_chain_018.run;
fn main() -> Int { return run(); }
