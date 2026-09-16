// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module structures {
  pub enum BST[T] {
    Empty,
    Node(value: T, left: BST[T], right: BST[T]),
  }

  pub fn BST.new[T]() -> BST[T] {
    return Empty;
  }

  pub fn BST.insert[T](val: T) -> BST[T] {
    match self {
      Empty => Node(value: val, left: Empty, right: Empty),
      Node(value: v, left: l, right: r) => {
        if val < v {
          return Node(value: v, left: l.insert(val), right: r);
        }
        elif val > v {
          return Node(value: v, left: l, right: r.insert(val));
        } else {
          return Node(value: v, left: l, right: r);
        }
      }
    }
  }

  fn test() -> Int {
    var t: BST[Int] = BST.new[Int]();
    var t2 = t.insert(5);
    return 1;
  }

  pub fn run_all() -> Int { return test(); }
}

use structures.run_all;
fn main() -> Int { return run_all(); }
