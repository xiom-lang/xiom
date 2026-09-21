// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_mod.main

pub type BenchResult = {
  name: Str;
  score: Int;
  max_score: Int;
  passed: Bool;
  elapsed_ms: Int;
} derive[Clone, Eq]

pub fn make_result(name: Str, score: Int, max: Int) -> BenchResult {
  return BenchResult{ name: name, score: score, max_score: max, passed: true, elapsed_ms: 0 };
}
