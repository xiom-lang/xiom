// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module s {
  pub enum BST {
    Empty,
    Node(value: Int, left: Int, right: Int),
  }

  pub fn make(v: Int, l: Int, r: Int) -> BST {
    return Node(value: v, left: l, right: r);
  }

  pub fn run() -> Int {
    var x = make(1, 2, 3);
    return 1;
  }
}

use s.run;
fn main() -> Int { return run(); }
