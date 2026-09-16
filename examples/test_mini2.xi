// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module data_primes {
  pub fn primes_table() -> Vec[Int] {
    var p = Vec[Int].new();
    p.push(2);
    return p;
  }
}

module types {
  pub type Counter = {
    value: Int;
    step: Int;
  }

  pub fn Counter.new(start: Int, step: Int) -> Counter {
    return Counter{ value: start, step: step };
  }

  pub fn Counter.set(new_val: Int) -> Counter {
    return Counter{ value: new_val, step: step };
  }

  pub fn Counter.inc() -> Counter {
    return Counter{ value: value + step, step: step };
  }

  fn test_counter() -> Int {
    var c = Counter.new(0, 1);
    var c2 = c.inc();
    var c5 = c2.set(100);
    if c5.value == 100 { return 1; }
    return 0;
  }

  pub fn run_types() -> Int { return test_counter(); }
}

use types.run_types;

fn main() -> Int { return run_types(); }
