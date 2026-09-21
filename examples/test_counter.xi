// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test {
  pub type Counter = {
    value: Int;
    step: Int;
  }

  pub fn Counter.new(start: Int, step: Int) -> Counter {
    return Counter{ value: start, step: step };
  }

  pub fn Counter.inc() -> Counter {
    return Counter{ value: value + step, step: step };
  }

  pub fn Counter.reset() -> Counter {
    return Counter{ value: 0, step: step };
  }

  fn test_counter() -> Int {
    var c = Counter.new(0, 1);
    var c2 = c.inc();
    if c2.value == 1 { return 1; }
    var c3 = c2.reset();
    if c3.value == 0 { return 2; }
    return 0;
  }

  pub fn run() -> Int { return test_counter(); }
}

use test.run;
fn main() -> Int { return run(); }
