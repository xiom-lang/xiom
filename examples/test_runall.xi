// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub type BenchResult = {
  name: Str;
  score: Int;
  max_score: Int;
  passed: Bool;
  elapsed_ms: Int;
} derive[Clone]

module types {
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
    var c2 = c.inc();
    if c2.value == 1 { score = score + 1; }
    var c4 = c2.inc().inc();  
    var c5 = c4.set(100);
    if c5.value == 100 { score = score + 1; }
    return score;
  }

  pub fn run_all() -> BenchResult {
    var total = test_counter();
    return BenchResult{
      name: "types",
      score: total,
      max_score: 10,
      passed: total == 10,
      elapsed_ms: 0,
    };
  }
}

use types.run_all;

fn main() -> Int {
  var r = run_all();
  return r.score;
}
