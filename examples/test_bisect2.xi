// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

pub type BenchResult = {
  name: Str;
  score: Int;
  max_score: Int;
  passed: Bool;
  elapsed_ms: Int;
} derive[Clone]

module data_primes {
  pub fn primes_table() -> Vec[Int] {
    var p = Vec[Int].new();
    p.push(2); p.push(3); p.push(5); p.push(7);
    return p;
  }
  pub fn run_all() -> BenchResult {
    return BenchResult{ name: "data", score: 0, max_score: 0, passed: true, elapsed_ms: 0 };
  }
}

module math {
  pub fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
  }
  pub fn run_all() -> BenchResult {
    return BenchResult{ name: "math", score: 0, max_score: 0, passed: true, elapsed_ms: 0 };
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
  pub fn Counter.inc() -> Counter {
    return Counter{ value: value + step, step: step };
  }
  pub fn Counter.set(new_val: Int) -> Counter {
    return Counter{ value: new_val, step: step };
  }
  fn test() -> Int {
    var c = Counter.new(0, 1);
    var c2 = c.inc();
    var c3 = c2.set(100);
    if c3.value == 100 { return 1; }
    return 0;
  }
  pub fn run_all() -> BenchResult {
    var total = test();
    return BenchResult{ name: "types", score: total, max_score: 10, passed: total == 10, elapsed_ms: 0 };
  }
}

use types.run_all;

fn main() -> Int {
  var r = run_all();
  return r.score;
}
