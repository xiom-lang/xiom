// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test2 {
  pub type Counter[T] = {
    val: T;
    count: Int;
  }

  pub fn Counter.new[T](initial: T) -> Counter[T] {
    return Counter[T]{ val: initial, count: 0 };
  }

  pub fn Counter.set[T](new_val: T) {
    val = new_val;
    count = count + 1;
  }

  pub fn Counter.times_updated[T]() -> Int {
    return count;
  }

  fn test() -> Int {
    var ci = Counter.new[Int](0);
    ci.set(42);
    if ci.times_updated() == 1 { return 1; }
    return 0;
  }

  pub fn run2() -> Int { return test(); }
}

use test2.run2;
fn main() -> Int { return run2(); }
