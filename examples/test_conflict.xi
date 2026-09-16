// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module first {
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
  fn test_first() -> Int {
    var c = Counter.new(0, 1);
    var c2 = c.set(100);
    if c2.value == 100 { return 1; }
    return 0;
  }
  pub fn run_first() -> Int { return test_first(); }
}

module second {
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
  fn test_second() -> Int {
    var c = Counter.new(0, 1);
    var c2 = c.set(200);
    if c2.value == 200 { return 1; }
    return 0;
  }
  pub fn run_second() -> Int { return test_second(); }
}

use first.run_first;
use second.run_second;

fn main() -> Int {
  var total = 0;
  total = total + run_first();
  total = total + run_second();
  return total;
}
