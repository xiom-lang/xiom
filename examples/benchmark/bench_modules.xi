// XIOM — Module Interoperability Stress Benchmark
// Exercises cross-module calls, layered imports, re-exports, and module depth.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.modules

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Internal Module Utilities
// ============================================================

pub fn greet(name: Str) -> Str {
  return name;
}

pub fn add(a: Int, b: Int) -> Int {
  return a + b;
}

pub fn sub(a: Int, b: Int) -> Int {
  return a - b;
}

pub fn mul(a: Int, b: Int) -> Int {
  return a * b;
}

pub fn div(a: Int, b: Int) -> Int {
  if b == 0 { return 0; }
  return a / b;
}

pub fn square(x: Int) -> Int {
  return x * x;
}

pub fn cube(x: Int) -> Int {
  return x * x * x;
}

pub fn is_positive(x: Int) -> Bool {
  return x > 0;
}

pub fn is_negative(x: Int) -> Bool {
  return x < 0;
}

pub fn is_zero(x: Int) -> Bool {
  return x == 0;
}

// ============================================================
// SECTION 2: Type Definitions for Cross-Module Use
// ============================================================

pub type ModuleInfo = {
  name: Str;
  version: Int;
  active: Bool;
} derive[Eq, Clone]

pub type Version = {
  major: Int;
  minor: Int;
  patch: Int;
} derive[Eq, Clone]

pub fn ModuleInfo.new(name: Str, version: Int) -> ModuleInfo {
  return ModuleInfo{ name: name, version: version, active: true };
}

pub fn Version.new(major: Int, minor: Int, patch: Int) -> Version {
  return Version{ major: major, minor: minor, patch: patch };
}

pub fn Version.to_int() -> Int {
  return major * 10000 + minor * 100 + patch;
}

// ============================================================
// SECTION 3: Function That Calls Across Modules
// ============================================================

pub fn call_math_twice(x: Int) -> Int {
  var a = square(x);
  return cube(a);
}

pub fn validate_range(val: Int, lo: Int, hi: Int) -> Bool {
  return val >= lo && val <= hi;
}

pub fn classify_int(n: Int) -> Int {
  if is_positive(n) { return 1; }
  elif is_negative(n) { return -1; }
  return 0;
}

// ============================================================
// SECTION 4: Module-Level Config/Tables
// ============================================================

pub fn factorial_table(n: Int) -> Int {
  if n < 0 { return 0; }
  if n > 10 { return -1; }
  var table = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800];
  return table[n];
}

pub fn square_table(n: Int) -> Int {
  if n < 0 || n > 20 { return -1; }
  var squares = [
    0, 1, 4, 9, 16, 25, 36, 49, 64, 81,
    100, 121, 144, 169, 196, 225, 256, 289, 324, 361, 400,
  ];
  return squares[n];
}

// ============================================================
// SECTION 5: Aggregate Test Functions
// ============================================================

fn test_utility_functions() -> Int {
  var score = 0;
  if add(2, 3) == 5 { score = score + 1; }
  if sub(10, 3) == 7 { score = score + 1; }
  if mul(7, 8) == 56 { score = score + 1; }
  if div(100, 4) == 25 { score = score + 1; }
  if div(10, 0) == 0 { score = score + 1; }
  if square(5) == 25 { score = score + 1; }
  if cube(3) == 27 { score = score + 1; }
  return score;
}

fn test_predicates() -> Int {
  var score = 0;
  if is_positive(5) { score = score + 1; }
  if !(is_positive(-1)) { score = score + 1; }
  if !(is_positive(0)) { score = score + 1; }
  if is_negative(-3) { score = score + 1; }
  if !(is_negative(5)) { score = score + 1; }
  if is_zero(0) { score = score + 1; }
  if !(is_zero(1)) { score = score + 1; }
  return score;
}

fn test_types() -> Int {
  var score = 0;
  var mi = ModuleInfo.new("test", 1);
  if mi.name == "test" { score = score + 1; }
  if mi.version == 1 { score = score + 1; }
  if mi.active { score = score + 1; }

  var v = Version.new(1, 2, 3);
  if v.major == 1 { score = score + 1; }
  if v.minor == 2 { score = score + 1; }
  if v.patch == 3 { score = score + 1; }
  if v.to_int() == 10203 { score = score + 1; }

  return score;
}

fn test_cross_calls() -> Int {
  var score = 0;
  if call_math_twice(2) == 64 { score = score + 1; }
  if validate_range(5, 0, 10) { score = score + 1; }
  if !(validate_range(15, 0, 10)) { score = score + 1; }
  if classify_int(42) == 1 { score = score + 1; }
  if classify_int(-7) == -1 { score = score + 1; }
  if classify_int(0) == 0 { score = score + 1; }
  return score;
}

fn test_tables() -> Int {
  var score = 0;
  if factorial_table(0) == 1 { score = score + 1; }
  if factorial_table(5) == 120 { score = score + 1; }
  if factorial_table(10) == 3628800 { score = score + 1; }
  if factorial_table(-1) == 0 { score = score + 1; }
  if factorial_table(11) == -1 { score = score + 1; }

  if square_table(0) == 0 { score = score + 1; }
  if square_table(10) == 100 { score = score + 1; }
  if square_table(20) == 400 { score = score + 1; }
  if square_table(21) == -1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_utility_functions();
  total = total + s1;
  max_score = max_score + 7;

  var s2 = test_predicates();
  total = total + s2;
  max_score = max_score + 7;

  var s3 = test_types();
  total = total + s3;
  max_score = max_score + 7;

  var s4 = test_cross_calls();
  total = total + s4;
  max_score = max_score + 6;

  var s5 = test_tables();
  total = total + s5;
  max_score = max_score + 9;

  return BenchResult{
    name: "modules",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
