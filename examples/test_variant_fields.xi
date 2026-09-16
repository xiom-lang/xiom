// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module bst {
  pub enum BST[T] {
    Empty,
    Node(value: T, left: BST[T], right: BST[T]),
  }

  pub fn BST.new[T]() -> BST[T] {
    return Empty;
  }

  pub fn make_node[T](val: T, l: BST[T], r: BST[T]) -> BST[T] {
    return Node(value: val, left: l, right: r);
  }

  fn test() -> Int {
    var t = make_node(5, Empty, Empty);
    return 1;
  }

  pub fn run() -> Int { return test(); }
}

use bst.run;

fn main() -> Int { return run(); }
