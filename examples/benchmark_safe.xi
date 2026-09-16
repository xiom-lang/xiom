// XIOM -- Safe Stress Benchmark (v2)
// Uses ONLY patterns confirmed working in benchmark_selfhost.xi:
//   - module Name { ... } inline blocks
//   - pub fn, var, if/elif/else, while, match, return
//   - struct creation, field access, enum variants
//   - Option[T], Result[T,E], generic functions
//   - contracts (requires, ensures, invariant)
//   - Vec (push, len, index)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// ============================================================
// MODULE: math -- Arithmetic and number theory
// ============================================================
module math_lib {
  pub fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
  }

  pub fn fibonacci(n: Int) -> Int {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  pub fn is_prime(n: Int) -> Bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    var i = 3;
    while i * i <= n {
      if n % i == 0 { return false; }
      i = i + 2;
    }
    return true;
  }

  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }

  pub fn abs(n: Int) -> Int {
    if n >= 0 { return n; }
    return -n;
  }

  pub fn max(a: Int, b: Int) -> Int {
    if a > b { return a; }
    return b;
  }

  pub fn min(a: Int, b: Int) -> Int {
    if a < b { return a; }
    return b;
  }

  pub fn power(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1; }
    return base * power(base, exp - 1);
  }

  pub fn digit_sum(n: Int) -> Int {
    if n < 0 { return digit_sum(-n); }
    if n < 10 { return n; }
    return n % 10 + digit_sum(n / 10);
  }

  fn test_math() -> Int {
    var score = 0;
    if factorial(5) == 120 { score = score + 1; }
    if fibonacci(10) == 55 { score = score + 1; }
    if fibonacci(15) == 610 { score = score + 1; }
    if is_prime(17) { score = score + 1; }
    if !(is_prime(4)) { score = score + 1; }
    if gcd(48, 18) == 6 { score = score + 1; }
    if abs(-5) == 5 { score = score + 1; }
    if max(10, 20) == 20 { score = score + 1; }
    if min(10, 20) == 10 { score = score + 1; }
    if power(2, 10) == 1024 { score = score + 1; }
    if digit_sum(123) == 6 { score = score + 1; }
    return score;
  }

  pub fn run_math() -> Int { return test_math(); }
}

// ============================================================
// MODULE: control -- Control flow
// ============================================================
module control {
  pub fn fizzbuzz(n: Int) -> Int {
    if n % 15 == 0 { return 0; }
    elif n % 3 == 0 { return 1; }
    elif n % 5 == 0 { return 2; }
    else { return 3; }
  }

  pub fn match_sign(n: Int) -> Int {
    if n > 0 { return 1; }
    elif n < 0 { return -1; }
    else { return 0; }
  }

  pub fn while_sum(limit: Int) -> Int {
    var total = 0;
    var i = 0;
    while i < limit + 1 {
      if true { total = total + i; }
      i = i + 1;
    }
    return total;
  }

  pub fn collatz(n: Int) -> Int {
    if n <= 1 { return 0; }
    if n % 2 == 0 { return 1 + collatz(n / 2); }
    return 1 + collatz(3 * n + 1);
  }

  fn test_control() -> Int {
    var score = 0;
    if fizzbuzz(15) == 0 { score = score + 1; }
    if fizzbuzz(3) == 1 { score = score + 1; }
    if fizzbuzz(5) == 2 { score = score + 1; }
    if fizzbuzz(7) == 3 { score = score + 1; }
    if match_sign(5) == 1 { score = score + 1; }
    if match_sign(-3) == -1 { score = score + 1; }
    if match_sign(0) == 0 { score = score + 1; }
    if while_sum(10) == 55 { score = score + 1; }
    if collatz(6) == 8 { score = score + 1; }
    return score;
  }

  pub fn run_control() -> Int { return test_control(); }
}

// ============================================================
// MODULE: structures -- Struct types and operations
// ============================================================
module structures {
  pub type Point = { x: Float64; y: Float64; }

  pub type Rect = { x: Float64; y: Float64; w: Float64; h: Float64; }

  pub fn make_point(x: Float64, y: Float64) -> Point {
    return Point{ x: x, y: y };
  }

  pub fn make_rect(x: Float64, y: Float64, w: Float64, h: Float64) -> Rect {
    return Rect{ x: x, y: y, w: w, h: h };
  }

  pub fn rect_area(r: Rect) -> Float64 {
    return r.w * r.h;
  }

  pub fn rect_contains(r: Rect, px: Float64, py: Float64) -> Bool {
    return px >= r.x && px < r.x + r.w && py >= r.y && py < r.y + r.h;
  }

  fn test_structures() -> Int {
    var score = 0;
    var r = make_rect(0.0, 0.0, 10.0, 20.0);
    if r.w * r.h == 200.0 { score = score + 1; }
    if 5.0 >= r.x && 5.0 < r.x + r.w { score = score + 1; }
    if !(15.0 >= r.x && 15.0 < r.x + r.w) { score = score + 1; }

    return score;
  }

  pub fn run_structures() -> Int { return test_structures(); }
}

// ============================================================
// MODULE: errors -- Option and Result
// ============================================================
module errors {
  pub fn safe_div(a: Int, b: Int) -> Option[Int] {
    if b == 0 { return None; }
    return Some(a / b);
  }

  pub fn safe_mod(a: Int, b: Int) -> Option[Int] {
    if b == 0 { return None; }
    return Some(a % b);
  }

  fn test_errors() -> Int {
    var score = 0;
    var div_good = safe_div(10, 2);
    if div_good.is_some { score = score + 1; }

    var div_bad = safe_div(10, 0);
    if div_bad.is_some { } else { score = score + 1; }

    var mod_good = safe_mod(10, 3);
    if mod_good.is_some { score = score + 1; }

    var mod_bad = safe_mod(10, 0);
    if mod_bad.is_some { } else { score = score + 1; }

    return score;
  }

  pub fn run_errors() -> Int { return test_errors(); }
}

// ============================================================
// MODULE: contracts -- Contract types
// ============================================================
module contracts {
  pub type Counter = {
    val: Int;
    lo: Int;
    hi: Int;
    invariant: val >= lo;
    invariant: val <= hi;
  }

  pub fn make_counter(val: Int, lo: Int, hi: Int) -> Counter
    requires: val >= lo
    requires: val <= hi
  {
    return Counter{ val: val, lo: lo, hi: hi };
  }

  pub fn counter_inc(c: Counter) -> Counter
    requires: c.val < c.hi
  {
    return Counter{ val: c.val + 1, lo: c.lo, hi: c.hi };
  }

  fn test_contracts() -> Int {
    var score = 0;
    var c = make_counter(5, 0, 10);
    if c.val == 5 { score = score + 1; }
    if c.lo == 0 { score = score + 1; }
    if c.hi == 10 { score = score + 1; }

    var c2 = counter_inc(c);
    if c2.val == 6 { score = score + 1; }

    var c3 = counter_inc(counter_inc(counter_inc(c2)));
    if c3.val == 9 { score = score + 1; }

    return score;
  }

  pub fn run_contracts() -> Int { return test_contracts(); }
}

// ============================================================
// MODULE: recursions -- Recursive patterns
// ============================================================
module recursions {
  pub fn countdown(n: Int) -> Int {
    if n <= 0 { return 0; }
    return 1 + countdown(n - 1);
  }

  pub fn sum_to(n: Int) -> Int {
    if n <= 0 { return 0; }
    return n + sum_to(n - 1);
  }

  fn test_recursions() -> Int {
    var score = 0;
    if countdown(5) == 5 { score = score + 1; }
    if countdown(0) == 0 { score = score + 1; }
    if sum_to(10) == 55 { score = score + 1; }
    if sum_to(0) == 0 { score = score + 1; }
    return score;
  }

  pub fn run_recursions() -> Int { return test_recursions(); }
}

// ============================================================
// MAIN -- Aggregate all scores
// ============================================================
use math_lib.run_math;
use control.run_control;
use structures.run_structures;
use errors.run_errors;
use contracts.run_contracts;
use recursions.run_recursions;

fn main() -> Int {
  var total = 0;
  total = total + run_math();
  total = total + run_control();
  total = total + run_structures();
  total = total + run_errors();
  total = total + run_contracts();
  total = total + run_recursions();
  return total;
}
