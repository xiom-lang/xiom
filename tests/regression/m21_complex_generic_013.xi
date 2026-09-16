// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_013
type Tree[T] = enum {
    Leaf,
    Node(value: T, left: Tree[T], right: Tree[T]),
  }

  pub fn run() -> Int {
    var t: Tree[Int] = Tree.Leaf;
    match t {
      Tree.Leaf => return 0,
      Tree.Node(_, _, _) => return 1,
    }
  }
use m21_complex_generic_013.run;
fn main() -> Int { return run(); }
