// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module bst_mod {
  pub enum BST[T] {
    Empty,
    Node(value: T, left: BST[T], right: BST[T]),
  }

  pub fn BST.new[T]() -> BST[T] {
    return Empty;
  }

  pub fn BST.insert[T](val: T) -> BST[T] {
    return Empty;
  }

  pub fn BST.size[T]() -> Int {
    return 0;
  }

  fn test() -> Int {
    var tree: BST[Int] = BST.new[Int]();
    var t1 = tree.insert(5);
    if t1.size() == 0 { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return test(); }
}

use bst_mod.run_test;

fn main() -> Int { return run_test(); }
