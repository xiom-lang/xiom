// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_004
enum Node {
    Leaf(val: Int),
    Branch(left: Int, right: Int),
  }

  pub fn run() -> Int {
    var n = Node.Branch(10, 20);
    match n {
      Node.Leaf(v) => return 1,
      Node.Branch(l, r) => if l == 10 && r == 20 { return 0; },
    }
  }
use m21_match_edge_004.run;
fn main() -> Int { return run(); }
