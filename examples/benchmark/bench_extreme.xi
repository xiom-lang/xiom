// XIOM -- Extreme Edge Case Stress
// Pushes the compiler with extreme patterns: deep nesting, large switch/match,
// many function parameters, deeply nested expressions, and corner cases.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.extreme

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Deep Nesting Patterns
// ============================================================

pub fn deep_if_nested(n: Int) -> Int {
  if n > 0 {
    return 1;
  } elif n < 0 {
    return -1;
  } else {
    if n == 0 {
      if n * 1 == 0 {
        if n + 0 == 0 {
          return 0;
        } else {
          return -99;
        }
      } else {
        return -98;
      }
    } else {
      return -97;
    }
  }
}

pub fn nested_loops(depth: Int) -> Int {
  var total = 0;
  var i = 0;
  while i < depth {
    var j = 0;
    while j < depth {
      var k = 0;
      while k < depth {
        total = total + 1;
        k = k + 1;
      }
      j = j + 1;
    }
    i = i + 1;
  }
  return total;
}

pub fn deep_expression_tree(a: Int, b: Int, c: Int, d: Int) -> Int {
  return ((a + b) * (c - d)) + ((a - b) * (c + d)) + ((a * b) / (if c + d == 0 { 1; } else { c + d; }));
}

fn test_deep_nesting() -> Int {
  var score = 0;
  if deep_if_nested(5) == 1 { score = score + 1; }
  if deep_if_nested(-3) == -1 { score = score + 1; }
  if deep_if_nested(0) == 0 { score = score + 1; }

  if nested_loops(3) == 27 { score = score + 1; }
  if nested_loops(1) == 1 { score = score + 1; }
  if nested_loops(5) == 125 { score = score + 1; }

  if deep_expression_tree(1, 2, 3, 4) == -4 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Many Arguments
// ============================================================

pub fn sum10(a0: Int, a1: Int, a2: Int, a3: Int, a4: Int,
  a5: Int, a6: Int, a7: Int, a8: Int, a9: Int) -> Int {
  return a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9;
}

pub fn mul5(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
  return a * b * c * d * e;
}

pub fn mixed_args(a: Int, b: Float64, c: Bool, d: Int, e: Float64) -> Float64 {
  var result = 0.0;
  if c { result = result + (a as Float64); }
  result = result + b * e;
  result = result + (d as Float64);
  return result;
}

fn test_many_args() -> Int {
  var score = 0;
  var s10 = sum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
  if s10 == 55 { score = score + 1; }
  var s10b = sum10(0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
  if s10b == 0 { score = score + 1; }

  if mul5(1, 2, 3, 4, 5) == 120 { score = score + 1; }
  if mul5(2, 2, 2, 2, 2) == 32 { score = score + 1; }

  var m1 = mixed_args(10, 2.0, true, 5, 3.0);
  if m1 == 21.0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Edge Cases -- Zero, Negative, Boundary Values
// ============================================================

pub fn zero_division_handler(a: Int, b: Int) -> Int {
  if b == 0 { return 0; }
  return a / b;
}

pub fn mod_negative(a: Int, b: Int) -> Int {
  if b == 0 { return -1; }
  return a % b;
}

pub fn max_edge(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
  var m = a;
  if b > m { m = b; }
  if c > m { m = c; }
  if d > m { m = d; }
  if e > m { m = e; }
  return m;
}

pub fn min_edge(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
  var m = a;
  if b < m { m = b; }
  if c < m { m = c; }
  if d < m { m = d; }
  if e < m { m = e; }
  return m;
}

fn test_edge_cases() -> Int {
  var score = 0;
  if zero_division_handler(10, 0) == 0 { score = score + 1; }
  if zero_division_handler(10, 2) == 5 { score = score + 1; }

  if mod_negative(10, 3) == 1 { score = score + 1; }
  if mod_negative(10, 0) == -1 { score = score + 1; }

  if max_edge(1, 2, 3, 4, 5) == 5 { score = score + 1; }
  if max_edge(-5, -4, -3, -2, -1) == -1 { score = score + 1; }
  if max_edge(0, 0, 0, 0, 0) == 0 { score = score + 1; }

  if min_edge(5, 4, 3, 2, 1) == 1 { score = score + 1; }
  if min_edge(-1, -2, -3, -4, -5) == -5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Large Match/Switch Pattern
// ============================================================

pub fn match_50(value: Int) -> Int {
  match value {
    0 => 0,
    1 => 1,
    2 => 4,
    3 => 9,
    4 => 16,
    5 => 25,
    6 => 36,
    7 => 49,
    8 => 64,
    9 => 81,
    10 => 100,
    11 => 121,
    12 => 144,
    13 => 169,
    14 => 196,
    15 => 225,
    16 => 256,
    17 => 289,
    18 => 324,
    19 => 361,
    20 => 400,
    21 => 441,
    22 => 484,
    23 => 529,
    24 => 576,
    25 => 625,
    26 => 676,
    27 => 729,
    28 => 784,
    29 => 841,
    30 => 900,
    31 => 961,
    32 => 1024,
    33 => 1089,
    34 => 1156,
    35 => 1225,
    36 => 1296,
    37 => 1369,
    38 => 1444,
    39 => 1521,
    40 => 1600,
    41 => 1681,
    42 => 1764,
    43 => 1849,
    44 => 1936,
    45 => 2025,
    46 => 2116,
    47 => 2209,
    48 => 2304,
    49 => 2401,
    _ => -1,
  }
}

fn test_large_match() -> Int {
  var score = 0;
  if match_50(0) == 0 { score = score + 1; }
  if match_50(5) == 25 { score = score + 1; }
  if match_50(10) == 100 { score = score + 1; }
  if match_50(25) == 625 { score = score + 1; }
  if match_50(49) == 2401 { score = score + 1; }
  if match_50(50) == -1 { score = score + 1; }
  if match_50(100) == -1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Recursion Depth
// ============================================================

pub fn sum_to_n(n: Int) -> Int {
  if n <= 0 { return 0; }
  return n + sum_to_n(n - 1);
}

pub fn count_down_to_zero(n: Int) -> Int {
  if n <= 0 { return 0; }
  return 1 + count_down_to_zero(n - 1);
}

pub fn nested_sum(depth: Int, value: Int) -> Int {
  if depth <= 0 { return value; }
  return nested_sum(depth - 1, value + 1);
}

fn test_recursion_depth() -> Int {
  var score = 0;
  if sum_to_n(10) == 55 { score = score + 1; }
  if sum_to_n(0) == 0 { score = score + 1; }
  if sum_to_n(100) == 5050 { score = score + 1; }

  if count_down_to_zero(5) == 5 { score = score + 1; }
  if count_down_to_zero(0) == 0 { score = score + 1; }

  if nested_sum(10, 0) == 10 { score = score + 1; }
  if nested_sum(0, 42) == 42 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_deep_nesting();
  total = total + s1;
  max_score = max_score + 7;

  var s2 = test_many_args();
  total = total + s2;
  max_score = max_score + 5;

  var s3 = test_edge_cases();
  total = total + s3;
  max_score = max_score + 9;

  var s4 = test_large_match();
  total = total + s4;
  max_score = max_score + 7;

  var s5 = test_recursion_depth();
  total = total + s5;
  max_score = max_score + 7;

  return BenchResult{
    name: "extreme",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
