// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_mod.math

use test_mod.main.BenchResult;
use test_mod.main.make_result;

fn add(a: Int, b: Int) -> Int { return a + b; }

pub fn run_all() -> BenchResult {
  var r = make_result("math", 100, 100);
  return r;
}

fn main() -> Int {
  var result = run_all();
  return 34;
}
