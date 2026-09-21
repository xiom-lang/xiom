// XIOM -- Algorithms Stress Benchmark
// Exercises classic algorithms: sorting, searching, graph, dynamic programming.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.algorithms

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Classic Numeric Algorithms
// ============================================================

pub fn sieve_eratosthenes(limit: Int) -> Int {
  if limit < 2 { return 0; }
  var count = 0;
  var n = 2;
  while n <= limit {
    var is_prime_n = 1;
    var d = 2;
    while d * d <= n {
      if n % d == 0 { is_prime_n = 0; d = n; }
      d = d + 1;
    }
    count = count + is_prime_n;
    n = n + 1;
  }
  return count;
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

pub fn lcm(a: Int, b: Int) -> Int {
  return a * b / gcd(a, b);
}

fn test_numeric_algorithms() -> Int {
  var score = 0;
  if sieve_eratosthenes(10) == 4 { score = score + 1; }
  if sieve_eratosthenes(30) == 10 { score = score + 1; }
  if sieve_eratosthenes(1) == 0 { score = score + 1; }

  if is_prime(17) { score = score + 1; }
  if is_prime(97) { score = score + 1; }
  if !(is_prime(91)) { score = score + 1; }
  if !(is_prime(1)) { score = score + 1; }

  if gcd(48, 18) == 6 { score = score + 1; }
  if gcd(7, 13) == 1 { score = score + 1; }

  if lcm(4, 6) == 12 { score = score + 1; }
  if lcm(7, 11) == 77 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Array Algorithms
// ============================================================

pub fn reverse(v: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = v.len();
  while i > 0 {
    i = i - 1;
    result.push(v[i]);
  }
  return result;
}

pub fn rotate_left(v: Vec[Int], k: Int) -> Vec[Int] {
  if v.len() == 0 { return v; }
  var shift = k % v.len();
  var result = Vec[Int].new();
  var i = shift;
  while i < v.len() {
    result.push(v[i]);
    i = i + 1;
  }
  i = 0;
  while i < shift {
    result.push(v[i]);
    i = i + 1;
  }
  return result;
}

pub fn min_max(v: &Vec[Int]) -> (Int, Int) {
  if v.len() == 0 { return (0, 0); }
  var min_val = v[0];
  var max_val = v[0];
  var i = 1;
  while i < v.len() {
    if v[i] < min_val { min_val = v[i]; }
    if v[i] > max_val { max_val = v[i]; }
    i = i + 1;
  }
  return (min_val, max_val);
}

pub fn prefix_sum(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var running = 0;
  var i = 0;
  while i < v.len() {
    running = running + v[i];
    result.push(running);
    i = i + 1;
  }
  return result;
}

fn test_array_algorithms() -> Int {
  var score = 0;
  var nums = [1, 2, 3, 4, 5];
  var rev = reverse(nums);
  if rev[0] == 5 { score = score + 1; }
  if rev[4] == 1 { score = score + 1; }

  var rotated = rotate_left(nums, 2);
  if rotated[0] == 3 { score = score + 1; }
  if rotated[3] == 1 { score = score + 1; }

  var (min_val, max_val) = min_max(&nums);
  if min_val == 1 { score = score + 1; }
  if max_val == 5 { score = score + 1; }

  var pref_sum = prefix_sum(&nums);
  if pref_sum.len() == 5 { score = score + 1; }
  if pref_sum[0] == 1 { score = score + 1; }
  if pref_sum[4] == 15 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Dynamic Programming
// ============================================================

pub fn fibonacci_dp(n: Int) -> Int {
  if n <= 1 { return n; }
  var prev2 = 0;
  var prev1 = 1;
  var i = 2;
  while i <= n {
    var curr = prev1 + prev2;
    prev2 = prev1;
    prev1 = curr;
    i = i + 1;
  }
  return prev1;
}

pub fn coin_change(amount: Int, coins: &Vec[Int]) -> Int {
  if amount == 0 { return 0; }
  if coins.len() == 0 { return -1; }
  var min_count = amount + 1;
  var i = 0;
  while i < coins.len() {
    if coins[i] <= amount {
      var sub_result = coin_change(amount - coins[i], coins);
      if sub_result >= 0 {
        var total = sub_result + 1;
        if total < min_count { min_count = total; }
      }
    }
    i = i + 1;
  }
  if min_count > amount { return -1; }
  return min_count;
}

pub fn max_subarray_sum(v: &Vec[Int]) -> Int {
  if v.len() == 0 { return 0; }
  var max_so_far = v[0];
  var max_ending = v[0];
  var i = 1;
  while i < v.len() {
    if max_ending + v[i] > v[i] {
      max_ending = max_ending + v[i];
    } else {
      max_ending = v[i];
    }
    if max_ending > max_so_far {
      max_so_far = max_ending;
    }
    i = i + 1;
  }
  return max_so_far;
}

fn test_dynamic_programming() -> Int {
  var score = 0;
  if fibonacci_dp(0) == 0 { score = score + 1; }
  if fibonacci_dp(1) == 1 { score = score + 1; }
  if fibonacci_dp(10) == 55 { score = score + 1; }
  if fibonacci_dp(20) == 6765 { score = score + 1; }

  var coins = [1, 5, 10, 25];
  if coin_change(30, &coins) == 2 { score = score + 1; }
  if coin_change(17, &coins) == 4 { score = score + 1; }
  if coin_change(0, &coins) == 0 { score = score + 1; }

  var arr = [-2, 1, -3, 4, -1, 2, 1, -5, 4];
  if max_subarray_sum(&arr) == 6 { score = score + 1; }

  var all_neg = [-5, -2, -3, -1];
  if max_subarray_sum(&all_neg) == -1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Distance & Geometry Algorithms
// ============================================================

pub fn manhattan(x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
  var dx = x1 - x2;
  if dx < 0 { dx = -dx; }
  var dy = y1 - y2;
  if dy < 0 { dy = -dy; }
  return dx + dy;
}

pub fn euclidean_sq(x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
  var dx = x1 - x2;
  var dy = y1 - y2;
  return dx * dx + dy * dy;
}

pub fn point_in_triangle(px: Int, py: Int, x1: Int, y1: Int, x2: Int, y2: Int, x3: Int, y3: Int) -> Bool {
  var d1 = sign_point(px, py, x1, y1, x2, y2);
  var d2 = sign_point(px, py, x2, y2, x3, y3);
  var d3 = sign_point(px, py, x3, y3, x1, y1);
  var has_neg = d1 < 0 || d2 < 0 || d3 < 0;
  var has_pos = d1 > 0 || d2 > 0 || d3 > 0;
  return !(has_neg && has_pos);
}

pub fn sign_point(px: Int, py: Int, x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
  return (px - x1) * (y2 - y1) - (py - y1) * (x2 - x1);
}

fn test_geometry() -> Int {
  var score = 0;
  if manhattan(0, 0, 3, 4) == 7 { score = score + 1; }
  if manhattan(1, 1, 4, 5) == 7 { score = score + 1; }

  if euclidean_sq(0, 0, 3, 4) == 25 { score = score + 1; }

  if point_in_triangle(2, 2, 0, 0, 4, 0, 2, 4) { score = score + 1; }
  if !(point_in_triangle(10, 10, 0, 0, 4, 0, 2, 4)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: String Algorithms (Int-based simulation)
// ============================================================

pub fn levenshtein(a: &Vec[Int], b: &Vec[Int]) -> Int {
  var m = a.len();
  var n = b.len();
  if m == 0 { return n; }
  if n == 0 { return m; }
  var cost = 0;
  if a[m - 1] != b[n - 1] { cost = 1; }
  var a_prefix = slice_vec(a, 0, m - 1);
  var b_prefix = slice_vec(b, 0, n - 1);
  var d1 = levenshtein(&a_prefix, b) + 1;
  var d2 = levenshtein(a, &b_prefix) + 1;
  var d3 = levenshtein(&a_prefix, &b_prefix) + cost;
  var min_val = d1;
  if d2 < min_val { min_val = d2; }
  if d3 < min_val { min_val = d3; }
  return min_val;
}

pub fn slice_vec(v: &Vec[Int], start: Int, end: Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = start;
  while i < end {
    result.push(v[i]);
    i = i + 1;
  }
  return result;
}

pub fn longest_common_prefix(a: &Vec[Int], b: &Vec[Int]) -> Int {
  var count = 0;
  var min_len = a.len();
  if b.len() < min_len { min_len = b.len(); }
  var i = 0;
  while i < min_len {
    if a[i] == b[i] { count = count + 1; }
    else { return count; }
    i = i + 1;
  }
  return count;
}

fn test_string_algorithms() -> Int {
  var score = 0;
  // LCP
  var s1 = [1, 2, 3, 4, 5];
  var s2 = [1, 2, 3, 9, 0];
  if longest_common_prefix(&s1, &s2) == 3 { score = score + 1; }

  var s3 = [1, 2, 3, 4, 5];
  var s4 = [1, 2, 3, 4, 5];
  if longest_common_prefix(&s3, &s4) == 5 { score = score + 1; }

  var s5 = [9, 8, 7];
  if longest_common_prefix(&s1, &s5) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_numeric_algorithms();
  total = total + s1;
  max_score = max_score + 11;

  var s2 = test_array_algorithms();
  total = total + s2;
  max_score = max_score + 8;

  var s3 = test_dynamic_programming();
  total = total + s3;
  max_score = max_score + 9;

  var s4 = test_geometry();
  total = total + s4;
  max_score = max_score + 5;

  var s5 = test_string_algorithms();
  total = total + s5;
  max_score = max_score + 3;

  return BenchResult{
    name: "algorithms",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
