// XIOM -- Prime Number Stress Benchmark
// Exercises prime generation, primality testing, and sieve algorithms.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.primes

use benchmark.main.BenchResult;
use benchmark.math.is_prime;
use benchmark.math.power_iter;

// ============================================================
// SECTION 1: Trial Division Primality Testing
// ============================================================

pub fn is_prime_opt(n: Int) -> Bool {
  if n < 2 { return false; }
  if n == 2 { return true; }
  if n % 2 == 0 { return false; }
  if n % 3 == 0 { return n == 3; }
  var i = 5;
  while i * i <= n {
    if n % i == 0 { return false; }
    if n % (i + 2) == 0 { return false; }
    i = i + 6;
  }
  return true;
}

pub fn count_primes_up_to(limit: Int) -> Int {
  var count = 0;
  var i = 2;
  while i <= limit {
    if is_prime_opt(i) { count = count + 1; }
    i = i + 1;
  }
  return count;
}

pub fn nth_prime(n: Int) -> Int {
  if n <= 0 { return 0; }
  var count = 0;
  var candidate = 1;
  while count < n {
    candidate = candidate + 1;
    if is_prime_opt(candidate) { count = count + 1; }
  }
  return candidate;
}

pub fn prime_gap(n: Int) -> Int {
  var p1 = nth_prime(n);
  var p2 = nth_prime(n + 1);
  return p2 - p1;
}

fn test_primality() -> Int {
  var score = 0;
  // Known primes
  if is_prime_opt(2) { score = score + 1; }
  if is_prime_opt(3) { score = score + 1; }
  if is_prime_opt(5) { score = score + 1; }
  if is_prime_opt(7) { score = score + 1; }
  if is_prime_opt(11) { score = score + 1; }
  if is_prime_opt(13) { score = score + 1; }
  if is_prime_opt(97) { score = score + 1; }
  if is_prime_opt(997) { score = score + 1; }
  if is_prime_opt(7919) { score = score + 1; }
  if is_prime_opt(104729) { score = score + 1; }

  // Known composites
  if !(is_prime_opt(1)) { score = score + 1; }
  if !(is_prime_opt(4)) { score = score + 1; }
  if !(is_prime_opt(6)) { score = score + 1; }
  if !(is_prime_opt(100)) { score = score + 1; }
  if !(is_prime_opt(91)) { score = score + 1; }

  // Count primes
  if count_primes_up_to(10) == 4 { score = score + 1; }
  if count_primes_up_to(100) == 25 { score = score + 1; }
  if count_primes_up_to(2) == 1 { score = score + 1; }

  // nth prime
  if nth_prime(1) == 2 { score = score + 1; }
  if nth_prime(2) == 3 { score = score + 1; }
  if nth_prime(10) == 29 { score = score + 1; }
  if nth_prime(25) == 97 { score = score + 1; }

  // Prime gaps
  if prime_gap(1) == 1 { score = score + 1; }
  if prime_gap(2) == 2 { score = score + 1; }
  if prime_gap(3) == 2 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Sieve of Eratosthenes (Array-Simulated)
// ============================================================

pub fn sieve_count(limit: Int) -> Int {
  if limit < 2 { return 0; }
  var count = 0;
  var n = 2;
  while n <= limit {
    var is_prime_n = 1;
    var d = 2;
    while d * d <= n {
      if n % d == 0 {
        is_prime_n = 0;
        d = n;
      }
      d = d + 1;
    }
    count = count + is_prime_n;
    n = n + 1;
  }
  return count;
}

pub fn sieve_sum(limit: Int) -> Int {
  if limit < 2 { return 0; }
  var sum = 0;
  var n = 2;
  while n <= limit {
    if is_prime_opt(n) { sum = sum + n; }
    n = n + 1;
  }
  return sum;
}

fn test_sieve() -> Int {
  var score = 0;
  if sieve_count(10) == 4 { score = score + 1; }
  if sieve_count(30) == 10 { score = score + 1; }
  if sieve_count(2) == 1 { score = score + 1; }
  if sieve_count(1) == 0 { score = score + 1; }

  if sieve_sum(10) == 17 { score = score + 1; }
  if sieve_sum(5) == 5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Prime Factorization
// ============================================================

pub fn factor_count(n: Int) -> Int {
  if n < 2 { return 0; }
  var count = 0;
  var x = n;
  var d = 2;
  while d * d <= x {
    while x % d == 0 {
      count = count + 1;
      x = x / d;
    }
    d = d + 1;
  }
  if x > 1 { count = count + 1; }
  return count;
}

pub fn factor_sum(n: Int) -> Int {
  if n < 2 { return 0; }
  var sum = 0;
  var x = n;
  var d = 2;
  while d * d <= x {
    while x % d == 0 {
      sum = sum + d;
      x = x / d;
    }
    d = d + 1;
  }
  if x > 1 { sum = sum + x; }
  return sum;
}

pub fn largest_factor(n: Int) -> Int {
  if n < 2 { return 0; }
  var max_f = 1;
  var x = n;
  var d = 2;
  while d * d <= x {
    while x % d == 0 {
      if d > max_f { max_f = d; }
      x = x / d;
    }
    d = d + 1;
  }
  if x > 1 && x > max_f { max_f = x; }
  return max_f;
}

fn test_factorization() -> Int {
  var score = 0;
  if factor_count(12) == 3 { score = score + 1; }
  if factor_count(28) == 3 { score = score + 1; }
  if factor_count(100) == 4 { score = score + 1; }
  if factor_count(17) == 1 { score = score + 1; }
  if factor_count(1) == 0 { score = score + 1; }

  if factor_sum(12) == 7 { score = score + 1; }
  if factor_sum(28) == 11 { score = score + 1; }
  if factor_sum(100) == 14 { score = score + 1; }

  if largest_factor(12) == 3 { score = score + 1; }
  if largest_factor(100) == 5 { score = score + 1; }
  if largest_factor(17) == 17 { score = score + 1; }
  if largest_factor(1) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Mersenne & Special Primes
// ============================================================

pub fn is_mersenne_prime(p: Int) -> Bool {
  if !(is_prime_opt(p)) { return false; }
  var mersenne = power_iter(2, p) - 1;
  return is_prime_opt(mersenne);
}

pub fn is_twin_prime(n: Int) -> Bool {
  return is_prime_opt(n) && (is_prime_opt(n - 2) || is_prime_opt(n + 2));
}

pub fn mersenne_number(p: Int) -> Int {
  return power_iter(2, p) - 1;
}

fn test_special_primes() -> Int {
  var score = 0;
  if is_mersenne_prime(2) { score = score + 1; }
  if is_mersenne_prime(3) { score = score + 1; }
  if is_mersenne_prime(5) { score = score + 1; }
  if is_mersenne_prime(7) { score = score + 1; }
  if !(is_mersenne_prime(11)) { score = score + 1; }

  if mersenne_number(3) == 7 { score = score + 1; }
  if mersenne_number(5) == 31 { score = score + 1; }

  if is_twin_prime(5) { score = score + 1; }
  if is_twin_prime(7) { score = score + 1; }
  if is_twin_prime(13) { score = score + 1; }
  if !(is_twin_prime(23)) { score = score + 1; }
  if !(is_twin_prime(2)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Goldbach & Number Theory Conjectures
// ============================================================

pub fn is_goldbach(even_n: Int) -> Bool {
  if even_n <= 2 || even_n % 2 != 0 { return false; }
  var i = 2;
  while i <= even_n / 2 {
    if is_prime_opt(i) && is_prime_opt(even_n - i) { return true; }
    i = i + 1;
  }
  return false;
}

pub fn goldbach_representation(n: Int) -> Int {
  if n <= 2 || n % 2 != 0 { return 0; }
  var i = 2;
  while i <= n / 2 {
    if is_prime_opt(i) && is_prime_opt(n - i) { return i; }
    i = i + 1;
  }
  return 0;
}

fn test_goldbach() -> Int {
  var score = 0;
  if is_goldbach(4) { score = score + 1; }
  if is_goldbach(6) { score = score + 1; }
  if is_goldbach(10) { score = score + 1; }
  if is_goldbach(100) { score = score + 1; }
  if !(is_goldbach(2)) { score = score + 1; }

  if goldbach_representation(4) == 2 { score = score + 1; }
  if goldbach_representation(10) == 3 { score = score + 1; }
  if goldbach_representation(2) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Totient and Coprime Count
// ============================================================

pub fn gcd(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return gcd(b, a % b);
}

pub fn totient(n: Int) -> Int {
  if n <= 0 { return 0; }
  if n == 1 { return 1; }
  var count = 0;
  var i = 1;
  while i < n {
    if gcd(i, n) == 1 { count = count + 1; }
    i = i + 1;
  }
  return count;
}

pub fn is_coprime(a: Int, b: Int) -> Bool {
  return gcd(a, b) == 1;
}

fn test_totient() -> Int {
  var score = 0;
  if totient(1) == 1 { score = score + 1; }
  if totient(2) == 1 { score = score + 1; }
  if totient(3) == 2 { score = score + 1; }
  if totient(4) == 2 { score = score + 1; }
  if totient(5) == 4 { score = score + 1; }
  if totient(6) == 2 { score = score + 1; }
  if totient(7) == 6 { score = score + 1; }
  if totient(8) == 4 { score = score + 1; }
  if totient(9) == 6 { score = score + 1; }
  if totient(10) == 4 { score = score + 1; }
  if totient(12) == 4 { score = score + 1; }

  if is_coprime(3, 7) { score = score + 1; }
  if is_coprime(8, 15) { score = score + 1; }
  if !(is_coprime(6, 9)) { score = score + 1; }
  if !(is_coprime(12, 8)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Prime Sequences
// ============================================================

pub fn generate_primes(count: Int) -> Vec[Int] {
  var primes = Vec[Int].new();
  var n = 2;
  while primes.len() < count {
    if is_prime_opt(n) { primes.push(n); }
    n = n + 1;
  }
  return primes;
}

pub fn prime_sum_in_range(start: Int, end: Int) -> Int {
  var sum = 0;
  var n = start;
  while n <= end {
    if is_prime_opt(n) { sum = sum + n; }
    n = n + 1;
  }
  return sum;
}

fn test_prime_sequences() -> Int {
  var score = 0;
  var primes = generate_primes(5);
  if primes.len() == 5 { score = score + 1; }
  if primes.len() >= 1 && primes[0] == 2 { score = score + 1; }
  if primes.len() >= 5 && primes[4] == 11 { score = score + 1; }

  if prime_sum_in_range(2, 10) == 17 { score = score + 1; }
  if prime_sum_in_range(10, 20) == 31 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Primality by Wilson's Theorem
// ============================================================

pub fn wilson_check(n: Int) -> Bool {
  if n < 2 { return false; }
  if n == 2 { return true; }
  var fact = 1;
  var i = 2;
  while i < n {
    fact = (fact * i) % n;
    i = i + 1;
  }
  return fact == n - 1;
}

fn test_wilson() -> Int {
  var score = 0;
  if wilson_check(2) { score = score + 1; }
  if wilson_check(3) { score = score + 1; }
  if wilson_check(5) { score = score + 1; }
  if wilson_check(7) { score = score + 1; }
  if wilson_check(11) { score = score + 1; }
  if !(wilson_check(4)) { score = score + 1; }
  if !(wilson_check(9)) { score = score + 1; }
  if !(wilson_check(1)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 9: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_primality();
  total = total + s1;
  max_score = max_score + 26;

  var s2 = test_sieve();
  total = total + s2;
  max_score = max_score + 6;

  var s3 = test_factorization();
  total = total + s3;
  max_score = max_score + 12;

  var s4 = test_special_primes();
  total = total + s4;
  max_score = max_score + 12;

  var s5 = test_goldbach();
  total = total + s5;
  max_score = max_score + 8;

  var s6 = test_totient();
  total = total + s6;
  max_score = max_score + 15;

  var s7 = test_prime_sequences();
  total = total + s7;
  max_score = max_score + 5;

  var s8 = test_wilson();
  total = total + s8;
  max_score = max_score + 8;

  return BenchResult{
    name: "primes",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
