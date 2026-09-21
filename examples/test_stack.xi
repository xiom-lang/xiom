// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module stack_mod {
  pub type Stack[T] = {
    data: Vec[T];
    top: Int;
  }

  pub fn Stack.new[T]() -> Stack[T] {
    return Stack[T]{ data: Vec[T].new(), top: 0 };
  }

  pub fn Stack.push[T](val: T) {
    data.push(val);
    top = data.len();
  }

  pub fn Stack.size[T]() -> Int {
    return top;
  }

  fn test() -> Int {
    var s: Stack[Int] = Stack.new[Int]();
    s.push(42);
    if s.size() == 1 { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return test(); }
}

use stack_mod.run_test;

fn main() -> Int { return run_test(); }
