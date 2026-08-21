// XIOM -- Closures/Lambdas Stress Benchmark
// Exercises closure definitions, captures, higher-order functions, and callbacks.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.closures

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Basic Closure Definitions
// ============================================================

fn test_basic_closures() -> Int {
  var score = 0;

  var doubler = fn(x: Int) -> Int { return x * 2; };
  if doubler(5) == 10 { score = score + 1; }
  if doubler(0) == 0 { score = score + 1; }
  if doubler(-3) == -6 { score = score + 1; }

  var tripler = fn(x: Int) -> Int { return x * 3; };
  if tripler(7) == 21 { score = score + 1; }

  var square = fn(x: Int) -> Int { return x * x; };
  if square(5) == 25 { score = score + 1; }
  if square(10) == 100 { score = score + 1; }

  var is_even = fn(n: Int) -> Bool { return n % 2 == 0; };
  if is_even(2) { score = score + 1; }
  if !(is_even(3)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Higher-Order Functions
// ============================================================

fn apply_int(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn apply_bool(f: fn(Int) -> Bool, x: Int) -> Bool { return f(x); }
fn compose_ints(f: fn(Int) -> Int, g: fn(Int) -> Int, x: Int) -> Int { return f(g(x)); }
fn apply_twice(f: fn(Int) -> Int, x: Int) -> Int { return f(f(x)); }

fn test_higher_order() -> Int {
  var score = 0;
  var add5 = fn(x: Int) -> Int { return x + 5; };
  var mul3 = fn(x: Int) -> Int { return x * 3; };
  var is_pos = fn(x: Int) -> Bool { return x > 0; };

  if apply_int(add5, 10) == 15 { score = score + 1; }
  if apply_int(mul3, 7) == 21 { score = score + 1; }
  if apply_bool(is_pos, 5) { score = score + 1; }
  if !(apply_bool(is_pos, -3)) { score = score + 1; }
  if compose_ints(add5, mul3, 4) == 17 { score = score + 1; }
  if compose_ints(mul3, add5, 4) == 27 { score = score + 1; }
  if apply_twice(add5, 0) == 10 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Closure as Parameters
// ============================================================

fn map_vec(v: &Vec[Int], f: fn(Int) -> Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() { result.push(f(v[i])); i = i + 1; }
  return result;
}

fn filter_vec(v: &Vec[Int], pred: fn(Int) -> Bool) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() { if pred(v[i]) { result.push(v[i]); } i = i + 1; }
  return result;
}

fn fold_vec(v: &Vec[Int], initial: Int, f: fn(Int, Int) -> Int) -> Int {
  var accum = initial;
  var i = 0;
  while i < v.len() { accum = f(accum, v[i]); i = i + 1; }
  return accum;
}

fn test_closure_params() -> Int {
  var score = 0;
  var nums = [1, 2, 3, 4, 5];

  var doubled = map_vec(&nums, fn(x: Int) -> Int { return x * 2; });
  if doubled[0] == 2 { score = score + 1; }
  if doubled[4] == 10 { score = score + 1; }

  var evens = filter_vec(&nums, fn(n: Int) -> Bool { return n % 2 == 0; });
  if evens.len() == 2 { score = score + 1; }
  if evens[0] == 2 { score = score + 1; }

  var sum = fold_vec(&nums, 0, fn(a: Int, b: Int) -> Int { return a + b; });
  if sum == 15 { score = score + 1; }

  var product = fold_vec(&nums, 1, fn(a: Int, b: Int) -> Int { return a * b; });
  if product == 120 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Closure Factories
// ============================================================

fn make_multiplier(factor: Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { return x * factor; }; }
fn make_adder(amount: Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { return x + amount; }; }
fn make_discriminant(pivot: Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { if x > pivot { return 1; } if x < pivot { return -1; } return 0; }; }

fn test_closure_factories() -> Int {
  var score = 0;
  var times10 = make_multiplier(10);
  if times10(5) == 50 { score = score + 1; }
  if times10(0) == 0 { score = score + 1; }
  var times3 = make_multiplier(3);
  if times3(7) == 21 { score = score + 1; }
  var add100 = make_adder(100);
  if add100(50) == 150 { score = score + 1; }
  var disc = make_discriminant(10);
  if disc(20) == 1 { score = score + 1; }
  if disc(5) == -1 { score = score + 1; }
  if disc(10) == 0 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 5: Closure Chaining
// ============================================================

fn chain_two(f: fn(Int) -> Int, g: fn(Int) -> Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { return f(g(x)); }; }
fn chain_three(f: fn(Int) -> Int, g: fn(Int) -> Int, h: fn(Int) -> Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { return f(g(h(x))); }; }

fn test_closure_chains() -> Int {
  var score = 0;
  var add1 = fn(x: Int) -> Int { return x + 1; };
  var double = fn(x: Int) -> Int { return x * 2; };
  var square = fn(x: Int) -> Int { return x * x; };
  var d_plus_1 = chain_two(add1, double);
  if d_plus_1(5) == 11 { score = score + 1; }
  var chain3 = chain_three(add1, double, square);
  if chain3(3) == 19 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_basic_closures(); total = total + s1; max_score = max_score + 8;
  var s2 = test_higher_order(); total = total + s2; max_score = max_score + 8;
  var s3 = test_closure_params(); total = total + s3; max_score = max_score + 6;
  var s4 = test_closure_factories(); total = total + s4; max_score = max_score + 8;
  var s5 = test_closure_chains(); total = total + s5; max_score = max_score + 2;

  return BenchResult{ name: "closures", score: total, max_score: max_score, passed: total == max_score, elapsed_ms: 0 };
}
