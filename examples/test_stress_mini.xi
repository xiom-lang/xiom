// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module types {
  pub type Counter = {
    value: Int;
    step: Int;
  } derive[Clone]

  pub fn Counter.new(start: Int, step: Int) -> Counter {
    return Counter{ value: start, step: step };
  }

  pub fn Counter.inc() -> Counter {
    return Counter{ value: value + step, step: step };
  }

  pub fn Counter.dec() -> Counter {
    return Counter{ value: value - step, step: step };
  }

  pub fn Counter.reset() -> Counter {
    return Counter{ value: 0, step: step };
  }

  pub fn Counter.set(new_val: Int) -> Counter {
    return Counter{ value: new_val, step: step };
  }

  fn test_counter() -> Int {
    var score = 0;
    var c = Counter.new(0, 1);
    if c.value == 0 { score = score + 1; }
    if c.step == 1 { score = score + 1; }

    var c2 = c.inc();
    if c2.value == 1 { score = score + 1; }

    var c3 = c2.inc().inc().inc();
    if c3.value == 4 { score = score + 1; }

    var c4 = c3.dec();
    if c4.value == 3 { score = score + 1; }

    var c5 = c4.set(100);
    if c5.value == 100 { score = score + 1; }

    var c6 = c5.reset();
    if c6.value == 0 { score = score + 1; }

    return score;
  }

  pub fn run_types() -> Int { return test_counter(); }
}

use types.run_types;

fn main() -> Int {
  return run_types();
}
