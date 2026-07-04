// XIOM — Recursion Stress Benchmark
// Tests deep recursion, mutual recursion, tail-call patterns,
// divide-and-conquer, tree traversal, and recursive algorithms.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.recursion

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Classic Recursive Functions
// ============================================================

pub fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}

pub fn fibonacci(n: Int) -> Int {
  if n <= 1 { return n; }
  return fibonacci(n - 1) + fibonacci(n - 2);
}

pub fn ackermann(m: Int, n: Int) -> Int {
  if m == 0 { return n + 1; }
  if n == 0 { return ackermann(m - 1, 1); }
  return ackermann(m - 1, ackermann(m, n - 1));
}

fn test_classic_recursion() -> Int {
  var score = 0;
  if factorial(0) == 1 { score = score + 1; }
  if factorial(5) == 120 { score = score + 1; }
  if factorial(7) == 5040 { score = score + 1; }

  if fibonacci(0) == 0 { score = score + 1; }
  if fibonacci(1) == 1 { score = score + 1; }
  if fibonacci(10) == 55 { score = score + 1; }
  if fibonacci(15) == 610 { score = score + 1; }

  // Ackermann small values
  if ackermann(0, 0) == 1 { score = score + 1; }
  if ackermann(0, 1) == 2 { score = score + 1; }
  if ackermann(1, 0) == 2 { score = score + 1; }
  if ackermann(1, 1) == 3 { score = score + 1; }
  if ackermann(2, 0) == 3 { score = score + 1; }
  if ackermann(2, 1) == 5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Mutual Recursion
// ============================================================

pub fn is_even_mut(n: Int) -> Bool {
  if n == 0 { return true; }
  return is_odd_mut(n - 1);
}

pub fn is_odd_mut(n: Int) -> Bool {
  if n == 0 { return false; }
  return is_even_mut(n - 1);
}

pub fn hofstadter_female(n: Int) -> Int {
  if n == 0 { return 1; }
  return n - hofstadter_male(hofstadter_female(n - 1));
}

pub fn hofstadter_male(n: Int) -> Int {
  if n == 0 { return 0; }
  return n - hofstadter_female(hofstadter_male(n - 1));
}

fn test_mutual_recursion() -> Int {
  var score = 0;
  if is_even_mut(0) { score = score + 1; }
  if is_even_mut(2) { score = score + 1; }
  if is_even_mut(10) { score = score + 1; }
  if !(is_even_mut(1)) { score = score + 1; }
  if !(is_even_mut(99)) { score = score + 1; }

  if is_odd_mut(1) { score = score + 1; }
  if is_odd_mut(99) { score = score + 1; }
  if !(is_odd_mut(0)) { score = score + 1; }

  // Hofstadter sequences
  if hofstadter_female(0) == 1 { score = score + 1; }
  if hofstadter_male(0) == 0 { score = score + 1; }
  if hofstadter_female(1) == 1 { score = score + 1; }
  if hofstadter_male(1) == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Divide and Conquer
// ============================================================

pub fn binary_search(arr: Vec[Int], target: Int, lo: Int, hi: Int) -> Int {
  if lo > hi { return -1; }
  var mid = lo + (hi - lo) / 2;
  if arr[mid] == target { return mid; }
  if arr[mid] < target { return binary_search(arr, target, mid + 1, hi); }
  return binary_search(arr, target, lo, mid - 1);
}

pub fn power_div_conq(base: Int, exp: Int) -> Int {
  if exp == 0 { return 1; }
  var half = power_div_conq(base, exp / 2);
  var half_sq = half * half;
  if exp % 2 == 0 { return half_sq; }
  return base * half_sq;
}

pub fn gcd(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return gcd(b, a % b);
}

pub fn sum_range_rec(lo: Int, hi: Int) -> Int {
  if lo > hi { return 0; }
  if lo == hi { return lo; }
  var mid = (lo + hi) / 2;
  return sum_range_rec(lo, mid) + sum_range_rec(mid + 1, hi);
}

pub fn max_rec(arr: Vec[Int], idx: Int) -> Int {
  if idx >= arr.len() { return -1000000; }
  var max_of_rest = max_rec(arr, idx + 1);
  if arr[idx] > max_of_rest { return arr[idx]; }
  return max_of_rest;
}

fn test_divide_conquer() -> Int {
  var score = 0;
  var sorted = [1, 3, 5, 7, 9, 11, 13, 15];
  if binary_search(sorted, 7, 0, sorted.len() - 1) == 3 { score = score + 1; }
  if binary_search(sorted, 1, 0, sorted.len() - 1) == 0 { score = score + 1; }
  if binary_search(sorted, 15, 0, sorted.len() - 1) == 7 { score = score + 1; }
  if binary_search(sorted, 8, 0, sorted.len() - 1) == -1 { score = score + 1; }
  var empty: Vec[Int] = [];
  if binary_search(empty, 5, 0, -1) == -1 { score = score + 1; }

  if power_div_conq(2, 10) == 1024 { score = score + 1; }
  if power_div_conq(3, 4) == 81 { score = score + 1; }
  if power_div_conq(5, 3) == 125 { score = score + 1; }

  if gcd(48, 18) == 6 { score = score + 1; }
  if gcd(1071, 462) == 21 { score = score + 1; }

  if sum_range_rec(1, 10) == 55 { score = score + 1; }
  if sum_range_rec(1, 1) == 1 { score = score + 1; }
  if sum_range_rec(10, 1) == 0 { score = score + 1; }

  var nums = [3, 7, 2, 9, 1, 5];
  if max_rec(nums, 0) == 9 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Recursive Sequence Generators
// ============================================================

pub fn padovan(n: Int) -> Int {
  if n == 0 || n == 1 || n == 2 { return 1; }
  return padovan(n - 2) + padovan(n - 3);
}

pub fn catalan(n: Int) -> Int {
  if n <= 1 { return 1; }
  var sum = 0;
  var i = 0;
  while i < n {
    sum = sum + catalan(i) * catalan(n - 1 - i);
    i = i + 1;
  }
  return sum;
}

pub fn tribonacci(n: Int) -> Int {
  if n == 0 { return 0; }
  if n == 1 || n == 2 { return 1; }
  return tribonacci(n - 1) + tribonacci(n - 2) + tribonacci(n - 3);
}

fn test_sequences() -> Int {
  var score = 0;
  if padovan(0) == 1 { score = score + 1; }
  if padovan(3) == 2 { score = score + 1; }
  if padovan(5) == 3 { score = score + 1; }
  if padovan(8) == 7 { score = score + 1; }

  if catalan(0) == 1 { score = score + 1; }
  if catalan(1) == 1 { score = score + 1; }
  if catalan(2) == 2 { score = score + 1; }
  if catalan(3) == 5 { score = score + 1; }
  if catalan(4) == 14 { score = score + 1; }

  if tribonacci(0) == 0 { score = score + 1; }
  if tribonacci(3) == 2 { score = score + 1; }
  if tribonacci(5) == 7 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Tree-Like Recursion
// ============================================================

pub fn binomial_rec(n: Int, k: Int) -> Int {
  if k == 0 || k == n { return 1; }
  return binomial_rec(n - 1, k - 1) + binomial_rec(n - 1, k);
}

pub fn stirling_second(n: Int, k: Int) -> Int {
  if k == 0 && n == 0 { return 1; }
  if k == 0 || n == 0 { return 0; }
  if k == n { return 1; }
  return k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1);
}

pub fn bell(n: Int) -> Int {
  if n == 0 { return 1; }
  var sum = 0;
  var k = 0;
  while k < n {
    sum = sum + binomial_rec(n - 1, k) * bell(k);
    k = k + 1;
  }
  return sum;
}

fn test_tree_recursion() -> Int {
  var score = 0;
  if binomial_rec(5, 2) == 10 { score = score + 1; }
  if binomial_rec(10, 5) == 252 { score = score + 1; }
  if binomial_rec(6, 0) == 1 { score = score + 1; }
  if binomial_rec(6, 6) == 1 { score = score + 1; }

  if stirling_second(0, 0) == 1 { score = score + 1; }
  if stirling_second(3, 2) == 3 { score = score + 1; }
  if stirling_second(4, 2) == 7 { score = score + 1; }

  if bell(0) == 1 { score = score + 1; }
  if bell(1) == 1 { score = score + 1; }
  if bell(2) == 2 { score = score + 1; }
  if bell(3) == 5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Palindromes & String Recursion
// ============================================================

pub fn reverse_num(n: Int) -> Int {
  if n < 10 { return n; }
  var digits = 1;
  var pow = 1;
  var x = n;
  while x >= 10 {
    x = x / 10;
    digits = digits + 1;
    pow = pow * 10;
  }
  return (n % 10) * pow + reverse_num(n / 10);
}

pub fn is_palindrome_num(n: Int) -> Bool {
  return n == reverse_num(n);
}

pub fn sum_digits_rec(n: Int) -> Int {
  if n == 0 { return 0; }
  if n < 0 { return sum_digits_rec(-n); }
  return n % 10 + sum_digits_rec(n / 10);
}

pub fn count_digits_rec(n: Int) -> Int {
  if n == 0 { return 1; }
  if n < 0 { return count_digits_rec(-n); }
  if n < 10 { return 1; }
  return 1 + count_digits_rec(n / 10);
}

fn test_string_recursion() -> Int {
  var score = 0;
  if reverse_num(123) == 321 { score = score + 1; }
  if reverse_num(100) == 1 { score = score + 1; }
  if reverse_num(5) == 5 { score = score + 1; }

  if is_palindrome_num(121) { score = score + 1; }
  if is_palindrome_num(12321) { score = score + 1; }
  if !(is_palindrome_num(123)) { score = score + 1; }

  if sum_digits_rec(123) == 6 { score = score + 1; }
  if sum_digits_rec(0) == 0 { score = score + 1; }
  if sum_digits_rec(-456) == 15 { score = score + 1; }

  if count_digits_rec(0) == 1 { score = score + 1; }
  if count_digits_rec(12345) == 5 { score = score + 1; }
  if count_digits_rec(1000000) == 7 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Hanoi & Backtracking
// ============================================================

pub fn hanoi_moves(n: Int) -> Int {
  if n == 0 { return 0; }
  return 2 * hanoi_moves(n - 1) + 1;
}

pub fn josephus(n: Int, k: Int) -> Int {
  if n == 1 { return 0; }
  return (josephus(n - 1, k) + k) % n;
}

pub fn mc_carthy(n: Int) -> Int {
  if n > 100 { return n - 10; }
  return mc_carthy(mc_carthy(n + 11));
}

fn test_backtracking() -> Int {
  var score = 0;
  if hanoi_moves(1) == 1 { score = score + 1; }
  if hanoi_moves(3) == 7 { score = score + 1; }
  if hanoi_moves(5) == 31 { score = score + 1; }
  if hanoi_moves(0) == 0 { score = score + 1; }

  if josephus(1, 3) == 0 { score = score + 1; }
  if josephus(5, 2) == 2 { score = score + 1; }

  if mc_carthy(80) == 91 { score = score + 1; }
  if mc_carthy(99) == 91 { score = score + 1; }
  if mc_carthy(101) == 91 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Recursive Math
// ============================================================

pub fn lucas(n: Int) -> Int {
  if n == 0 { return 2; }
  if n == 1 { return 1; }
  return lucas(n - 1) + lucas(n - 2);
}

pub fn jacobsthal(n: Int) -> Int {
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  return jacobsthal(n - 1) + 2 * jacobsthal(n - 2);
}

pub fn pell(n: Int) -> Int {
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  return 2 * pell(n - 1) + pell(n - 2);
}

fn test_recursive_series() -> Int {
  var score = 0;
  if lucas(0) == 2 { score = score + 1; }
  if lucas(1) == 1 { score = score + 1; }
  if lucas(5) == 11 { score = score + 1; }

  if jacobsthal(0) == 0 { score = score + 1; }
  if jacobsthal(1) == 1 { score = score + 1; }
  if jacobsthal(4) == 5 { score = score + 1; }

  if pell(0) == 0 { score = score + 1; }
  if pell(1) == 1 { score = score + 1; }
  if pell(4) == 12 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 9: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_classic_recursion();
  total = total + s1;
  max_score = max_score + 13;

  var s2 = test_mutual_recursion();
  total = total + s2;
  max_score = max_score + 12;

  var s3 = test_divide_conquer();
  total = total + s3;
  max_score = max_score + 14;

  var s4 = test_sequences();
  total = total + s4;
  max_score = max_score + 12;

  var s5 = test_tree_recursion();
  total = total + s5;
  max_score = max_score + 11;

  var s6 = test_string_recursion();
  total = total + s6;
  max_score = max_score + 12;

  var s7 = test_backtracking();
  total = total + s7;
  max_score = max_score + 9;

  var s8 = test_recursive_series();
  total = total + s8;
  max_score = max_score + 9;

  return BenchResult{
    name: "recursion",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
