// XIOM -- Math Stress Benchmark
// Pushes arithmetic, recursion, iteration, and numerical methods to extremes.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.math

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Basic Arithmetic -- verifying integer operations
// ============================================================

pub fn add_int(a: Int, b: Int) -> Int { return a + b; }
pub fn sub_int(a: Int, b: Int) -> Int { return a - b; }
pub fn mul_int(a: Int, b: Int) -> Int { return a * b; }
pub fn div_int(a: Int, b: Int) -> Int { return a / b; }
pub fn mod_int(a: Int, b: Int) -> Int { return a % b; }
pub fn neg_int(a: Int) -> Int { return -a; }

fn test_basic_arithmetic() -> Int {
  var score = 0;
  if add_int(2, 3) == 5 { score = score + 1; }
  if add_int(-5, 10) == 5 { score = score + 1; }
  if add_int(0, 0) == 0 { score = score + 1; }
  if add_int(1000000, 2000000) == 3000000 { score = score + 1; }
  if add_int(-1, -1) == -2 { score = score + 1; }

  if sub_int(10, 3) == 7 { score = score + 1; }
  if sub_int(0, 5) == -5 { score = score + 1; }
  if sub_int(-5, -3) == -2 { score = score + 1; }
  if sub_int(100, 100) == 0 { score = score + 1; }
  if sub_int(42, 100) == -58 { score = score + 1; }

  if mul_int(7, 8) == 56 { score = score + 1; }
  if mul_int(-3, 4) == -12 { score = score + 1; }
  if mul_int(-2, -5) == 10 { score = score + 1; }
  if mul_int(0, 999) == 0 { score = score + 1; }
  if mul_int(1, 12345) == 12345 { score = score + 1; }

  if div_int(100, 4) == 25 { score = score + 1; }
  if div_int(10, 3) == 3 { score = score + 1; }
  if div_int(7, 1) == 7 { score = score + 1; }
  if div_int(0, 5) == 0 { score = score + 1; }
  if div_int(-12, 4) == -3 { score = score + 1; }

  if mod_int(10, 3) == 1 { score = score + 1; }
  if mod_int(17, 5) == 2 { score = score + 1; }
  if mod_int(8, 2) == 0 { score = score + 1; }
  if mod_int(0, 7) == 0 { score = score + 1; }
  if mod_int(100, 7) == 2 { score = score + 1; }

  if neg_int(5) == -5 { score = score + 1; }
  if neg_int(-7) == 7 { score = score + 1; }
  if neg_int(0) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Float Operations -- Float64 arithmetic
// ============================================================

pub fn fadd(a: Float64, b: Float64) -> Float64 { return a + b; }
pub fn fsub(a: Float64, b: Float64) -> Float64 { return a - b; }
pub fn fmul(a: Float64, b: Float64) -> Float64 { return a * b; }
pub fn fdiv(a: Float64, b: Float64) -> Float64 { return a / b; }
pub fn fneg(a: Float64) -> Float64 { return -a; }

fn test_float_arithmetic() -> Int {
  var score = 0;
  if fadd(1.5, 2.5) == 4.0 { score = score + 1; }
  if fsub(10.0, 3.5) == 6.5 { score = score + 1; }
  if fmul(2.0, 3.5) == 7.0 { score = score + 1; }
  if fdiv(10.0, 4.0) == 2.5 { score = score + 1; }
  if fneg(3.14) == -3.14 { score = score + 1; }
  if fadd(-1.0, -1.0) == -2.0 { score = score + 1; }
  if fmul(0.0, 42.0) == 0.0 { score = score + 1; }
  if fdiv(1.0, 2.0) == 0.5 { score = score + 1; }
  if fsub(0.0, 5.0) == -5.0 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 3: Factorial -- iterative and recursive
// ============================================================

pub fn factorial_rec(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial_rec(n - 1);
}

pub fn factorial_iter(n: Int) -> Int {
  var result = 1;
  var i = 1;
  while i <= n {
    result = result * i;
    i = i + 1;
  }
  return result;
}

fn test_factorial() -> Int {
  var score = 0;
  if factorial_rec(0) == 1 { score = score + 1; }
  if factorial_rec(1) == 1 { score = score + 1; }
  if factorial_rec(5) == 120 { score = score + 1; }
  if factorial_rec(7) == 5040 { score = score + 1; }
  if factorial_rec(10) == 3628800 { score = score + 1; }

  if factorial_iter(0) == 1 { score = score + 1; }
  if factorial_iter(5) == 120 { score = score + 1; }
  if factorial_iter(7) == 5040 { score = score + 1; }
  if factorial_iter(10) == 3628800 { score = score + 1; }
  if factorial_iter(12) == 479001600 { score = score + 1; }

  // Cross-verify rec vs iter
  if factorial_rec(8) == factorial_iter(8) { score = score + 1; }
  if factorial_rec(9) == factorial_iter(9) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Fibonacci -- 3 implementations
// ============================================================

pub fn fibonacci_rec(n: Int) -> Int {
  if n <= 1 { return n; }
  return fibonacci_rec(n - 1) + fibonacci_rec(n - 2);
}

pub fn fibonacci_iter(n: Int) -> Int {
  if n <= 1 { return n; }
  var a = 0;
  var b = 1;
  var i = 2;
  while i <= n {
    var temp = a + b;
    a = b;
    b = temp;
    i = i + 1;
  }
  return b;
}

pub fn fib_nth(n: Int) -> Int {
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  return fibonacci_iter(n);
}

fn test_fibonacci() -> Int {
  var score = 0;
  if fibonacci_rec(0) == 0 { score = score + 1; }
  if fibonacci_rec(1) == 1 { score = score + 1; }
  if fibonacci_rec(10) == 55 { score = score + 1; }
  if fibonacci_rec(15) == 610 { score = score + 1; }
  if fibonacci_rec(20) == 6765 { score = score + 1; }

  if fibonacci_iter(0) == 0 { score = score + 1; }
  if fibonacci_iter(1) == 1 { score = score + 1; }
  if fibonacci_iter(10) == 55 { score = score + 1; }
  if fibonacci_iter(20) == 6765 { score = score + 1; }
  if fibonacci_iter(25) == 75025 { score = score + 1; }

  if fibonacci_rec(12) == fibonacci_iter(12) { score = score + 1; }
  if fibonacci_rec(16) == fibonacci_iter(16) { score = score + 1; }
  if fib_nth(10) == 55 { score = score + 1; }
  if fib_nth(0) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: GCD and LCM
// ============================================================

pub fn gcd(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return gcd(b, a % b);
}

pub fn lcm(a: Int, b: Int) -> Int {
  return a * b / gcd(a, b);
}

pub fn gcd_iter(a: Int, b: Int) -> Int {
  var x = a;
  var y = b;
  while y != 0 {
    var temp = y;
    y = x % y;
    x = temp;
  }
  return x;
}

fn test_gcd_lcm() -> Int {
  var score = 0;
  if gcd(48, 18) == 6 { score = score + 1; }
  if gcd(100, 10) == 10 { score = score + 1; }
  if gcd(7, 13) == 1 { score = score + 1; }
  if gcd(0, 5) == 5 { score = score + 1; }
  if gcd(5, 0) == 5 { score = score + 1; }
  if gcd(270, 192) == 6 { score = score + 1; }
  if gcd(1071, 462) == 21 { score = score + 1; }
  if gcd(1, 1) == 1 { score = score + 1; }
  if gcd(999999, 1) == 1 { score = score + 1; }

  if lcm(4, 6) == 12 { score = score + 1; }
  if lcm(7, 11) == 77 { score = score + 1; }
  if lcm(12, 18) == 36 { score = score + 1; }
  if lcm(1, 99) == 99 { score = score + 1; }

  if gcd(48, 18) == gcd_iter(48, 18) { score = score + 1; }
  if gcd(1071, 462) == gcd_iter(1071, 462) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Power and Exponentiation
// ============================================================

pub fn power_rec(base: Int, exp: Int) -> Int {
  if exp == 0 { return 1; }
  return base * power_rec(base, exp - 1);
}

pub fn power_iter(base: Int, exp: Int) -> Int {
  var result = 1;
  var e = exp;
  var b = base;
  while e > 0 {
    result = result * b;
    e = e - 1;
  }
  return result;
}

pub fn fast_pow(base: Int, exp: Int) -> Int {
  if exp == 0 { return 1; }
  var half = fast_pow(base, exp / 2);
  if exp % 2 == 0 {
    return half * half;
  } else {
    return base * half * half;
  }
}

pub fn pow_mod(base: Int, exp: Int, mod_m: Int) -> Int {
  var result = 1;
  var b = base % mod_m;
  var e = exp;
  while e > 0 {
    if e % 2 == 1 {
      result = (result * b) % mod_m;
    }
    e = e / 2;
    b = (b * b) % mod_m;
  }
  return result;
}

fn test_power() -> Int {
  var score = 0;
  if power_rec(2, 0) == 1 { score = score + 1; }
  if power_rec(2, 10) == 1024 { score = score + 1; }
  if power_rec(3, 5) == 243 { score = score + 1; }
  if power_rec(5, 4) == 625 { score = score + 1; }
  if power_rec(10, 3) == 1000 { score = score + 1; }

  if power_iter(2, 10) == 1024 { score = score + 1; }
  if power_iter(3, 5) == 243 { score = score + 1; }
  if power_iter(10, 0) == 1 { score = score + 1; }

  if fast_pow(2, 10) == 1024 { score = score + 1; }
  if fast_pow(3, 4) == 81 { score = score + 1; }
  if fast_pow(5, 3) == 125 { score = score + 1; }

  if power_rec(2, 8) == power_iter(2, 8) { score = score + 1; }
  if fast_pow(3, 6) == power_iter(3, 6) { score = score + 1; }

  if pow_mod(2, 10, 1000) == 24 { score = score + 1; }
  if pow_mod(3, 5, 13) == 9 { score = score + 1; }
  if pow_mod(7, 3, 5) == 3 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Combinatorics
// ============================================================

pub fn binomial(n: Int, k: Int) -> Int {
  if k == 0 || k == n { return 1; }
  if k > n { return 0; }
  return binomial(n - 1, k - 1) + binomial(n - 1, k);
}

pub fn combinations(n: Int, r: Int) -> Int {
  if r > n { return 0; }
  if r == 0 || r == n { return 1; }
  var result = 1;
  var i = 1;
  while i <= r {
    result = result * (n - r + i) / i;
    i = i + 1;
  }
  return result;
}

fn test_combinatorics() -> Int {
  var score = 0;
  if binomial(5, 0) == 1 { score = score + 1; }
  if binomial(5, 5) == 1 { score = score + 1; }
  if binomial(5, 2) == 10 { score = score + 1; }
  if binomial(6, 3) == 20 { score = score + 1; }
  if binomial(10, 5) == 252 { score = score + 1; }
  if binomial(7, 2) == 21 { score = score + 1; }

  if combinations(5, 2) == 10 { score = score + 1; }
  if combinations(10, 3) == 120 { score = score + 1; }
  if combinations(8, 4) == 70 { score = score + 1; }

  if binomial(6, 2) == combinations(6, 2) { score = score + 1; }
  if binomial(8, 3) == combinations(8, 3) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Number Theory
// ============================================================

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

pub fn is_perfect_square(n: Int) -> Bool {
  if n < 0 { return false; }
  var x = 1;
  var sum = 1;
  while sum < n {
    x = x + 2;
    sum = sum + x;
  }
  return sum == n;
}

pub fn digit_sum(n: Int) -> Int {
  if n < 0 { return digit_sum(-n); }
  if n < 10 { return n; }
  return n % 10 + digit_sum(n / 10);
}

pub fn digit_count(n: Int) -> Int {
  if n < 0 { return digit_count(-n); }
  if n == 0 { return 1; }
  var count = 0;
  var x = n;
  while x > 0 {
    count = count + 1;
    x = x / 10;
  }
  return count;
}

pub fn reverse_num(n: Int) -> Int {
  var sign = 1;
  var x = n;
  if n < 0 { sign = -1; x = -n; }
  var rev = 0;
  while x > 0 {
    rev = rev * 10 + x % 10;
    x = x / 10;
  }
  return sign * rev;
}

pub fn is_palindrome_num(n: Int) -> Bool {
  return n == reverse_num(n);
}

fn test_number_theory() -> Int {
  var score = 0;
  // Primes
  if is_prime(2) { score = score + 1; }
  if is_prime(3) { score = score + 1; }
  if is_prime(17) { score = score + 1; }
  if is_prime(97) { score = score + 1; }
  if is_prime(7919) { score = score + 1; }
  if !(is_prime(1)) { score = score + 1; }
  if !(is_prime(4)) { score = score + 1; }
  if !(is_prime(100)) { score = score + 1; }
  if !(is_prime(9991 * 9991)) { score = score + 1; }

  // Perfect squares
  if is_perfect_square(0) { score = score + 1; }
  if is_perfect_square(1) { score = score + 1; }
  if is_perfect_square(4) { score = score + 1; }
  if is_perfect_square(16) { score = score + 1; }
  if is_perfect_square(10000) { score = score + 1; }
  if !(is_perfect_square(2)) { score = score + 1; }
  if !(is_perfect_square(99)) { score = score + 1; }

  // Digit operations
  if digit_sum(123) == 6 { score = score + 1; }
  if digit_sum(0) == 0 { score = score + 1; }
  if digit_sum(99999) == 45 { score = score + 1; }
  if digit_sum(-123) == 6 { score = score + 1; }

  if digit_count(0) == 1 { score = score + 1; }
  if digit_count(5) == 1 { score = score + 1; }
  if digit_count(12345) == 5 { score = score + 1; }
  if digit_count(1000000) == 7 { score = score + 1; }

  if reverse_num(123) == 321 { score = score + 1; }
  if reverse_num(100) == 1 { score = score + 1; }
  if reverse_num(-123) == -321 { score = score + 1; }
  if reverse_num(0) == 0 { score = score + 1; }

  if is_palindrome_num(121) { score = score + 1; }
  if is_palindrome_num(12321) { score = score + 1; }
  if !(is_palindrome_num(123)) { score = score + 1; }
  if is_palindrome_num(0) { score = score + 1; }
  if is_palindrome_num(11) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 9: Sequences and Series
// ============================================================

pub fn triangular(n: Int) -> Int {
  if n <= 0 { return 0; }
  return n * (n + 1) / 2;
}

pub fn sum_range(lo: Int, hi: Int) -> Int {
  var sum = 0;
  var i = lo;
  while i <= hi {
    sum = sum + i;
    i = i + 1;
  }
  return sum;
}

pub fn collatz(n: Int) -> Int {
  if n <= 1 { return 0; }
  if n % 2 == 0 { return 1 + collatz(n / 2); }
  return 1 + collatz(3 * n + 1);
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

pub fn clamp(val: Int, lo: Int, hi: Int) -> Int {
  if val < lo { return lo; }
  if val > hi { return hi; }
  return val;
}

pub fn sign(n: Int) -> Int {
  if n > 0 { return 1; }
  if n < 0 { return -1; }
  return 0;
}

fn test_sequences() -> Int {
  var score = 0;
  if triangular(1) == 1 { score = score + 1; }
  if triangular(5) == 15 { score = score + 1; }
  if triangular(10) == 55 { score = score + 1; }
  if triangular(100) == 5050 { score = score + 1; }
  if triangular(0) == 0 { score = score + 1; }

  if sum_range(1, 10) == 55 { score = score + 1; }
  if sum_range(1, 100) == 5050 { score = score + 1; }
  if sum_range(5, 5) == 5 { score = score + 1; }
  if sum_range(-5, 5) == 0 { score = score + 1; }

  if collatz(1) == 0 { score = score + 1; }
  if collatz(6) == 8 { score = score + 1; }
  if collatz(27) == 111 { score = score + 1; }

  if abs(5) == 5 { score = score + 1; }
  if abs(-5) == 5 { score = score + 1; }
  if abs(0) == 0 { score = score + 1; }

  if max(10, 20) == 20 { score = score + 1; }
  if max(-5, -2) == -2 { score = score + 1; }
  if max(5, 5) == 5 { score = score + 1; }

  if min(10, 20) == 10 { score = score + 1; }
  if min(-5, -2) == -5 { score = score + 1; }

  if clamp(5, 0, 10) == 5 { score = score + 1; }
  if clamp(-1, 0, 10) == 0 { score = score + 1; }
  if clamp(15, 0, 10) == 10 { score = score + 1; }

  if sign(42) == 1 { score = score + 1; }
  if sign(-7) == -1 { score = score + 1; }
  if sign(0) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 10: Parity and Divisibility
// ============================================================

pub fn is_even(n: Int) -> Bool { return n % 2 == 0; }
pub fn is_odd(n: Int) -> Bool { return n % 2 != 0; }
pub fn is_multiple_of(a: Int, b: Int) -> Bool {
  if b == 0 { return false; }
  return a % b == 0;
}
pub fn count_divisors(n: Int) -> Int {
  if n <= 0 { return 0; }
  var count = 0;
  var i = 1;
  while i <= n {
    if n % i == 0 { count = count + 1; }
    i = i + 1;
  }
  return count;
}
pub fn divisor_sum(n: Int) -> Int {
  if n <= 0 { return 0; }
  var sum = 0;
  var i = 1;
  while i <= n {
    if n % i == 0 { sum = sum + i; }
    i = i + 1;
  }
  return sum;
}
pub fn is_perfect(n: Int) -> Bool {
  return n > 0 && divisor_sum(n) == 2 * n;
}

fn test_parity() -> Int {
  var score = 0;
  if is_even(0) { score = score + 1; }
  if is_even(2) { score = score + 1; }
  if is_even(100) { score = score + 1; }
  if !(is_even(1)) { score = score + 1; }
  if !(is_even(99)) { score = score + 1; }

  if is_odd(1) { score = score + 1; }
  if is_odd(99) { score = score + 1; }
  if !(is_odd(0)) { score = score + 1; }
  if !(is_odd(100)) { score = score + 1; }

  if is_multiple_of(10, 5) { score = score + 1; }
  if is_multiple_of(15, 3) { score = score + 1; }
  if !(is_multiple_of(10, 3)) { score = score + 1; }
  if !(is_multiple_of(5, 0)) { score = score + 1; }
  if is_multiple_of(0, 5) { score = score + 1; }

  if count_divisors(1) == 1 { score = score + 1; }
  if count_divisors(6) == 4 { score = score + 1; }
  if count_divisors(12) == 6 { score = score + 1; }
  if count_divisors(28) == 6 { score = score + 1; }

  if divisor_sum(6) == 12 { score = score + 1; }
  if divisor_sum(28) == 56 { score = score + 1; }
  if divisor_sum(12) == 28 { score = score + 1; }

  if is_perfect(6) { score = score + 1; }
  if is_perfect(28) { score = score + 1; }
  if !(is_perfect(12)) { score = score + 1; }
  if !(is_perfect(1)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 11: Numerical Methods
// ============================================================

pub fn sqrt_newton(x: Float64, epsilon: Float64) -> Float64 {
  if x < 0.0 { return -1.0; }
  if x == 0.0 { return 0.0; }
  var guess = x / 2.0;
  var i = 0;
  while i < 100 {
    var next_guess = (guess + x / guess) / 2.0;
    var diff = next_guess - guess;
    if diff < 0.0 { diff = -diff; }
    if diff < epsilon { return next_guess; }
    guess = next_guess;
    i = i + 1;
  }
  return guess;
}

pub fn abs_float(x: Float64) -> Float64 {
  if x >= 0.0 { return x; }
  return -x;
}

pub fn exp_taylor(x: Float64, terms: Int) -> Float64 {
  var result = 1.0;
  var term = 1.0;
  var i = 1;
  while i <= terms {
    term = term * x / (i as Float64);
    result = result + term;
    i = i + 1;
  }
  return result;
}

pub fn sin_taylor(x: Float64, terms: Int) -> Float64 {
  var result = x;
  var term = x;
  var i = 1;
  while i <= terms {
    term = -term * x * x / (((2 * i) * (2 * i + 1)) as Float64);
    result = result + term;
    i = i + 1;
  }
  return result;
}

pub fn cos_taylor(x: Float64, terms: Int) -> Float64 {
  var result = 1.0;
  var term = 1.0;
  var i = 1;
  while i <= terms {
    term = -term * x * x / (((2 * i - 1) * (2 * i)) as Float64);
    result = result + term;
    i = i + 1;
  }
  return result;
}

pub fn trapezoidal(f: fn(Float64) -> Float64, a: Float64, b: Float64, n: Int) -> Float64 {
  var h = (b - a) / (n as Float64);
  var sum = (f(a) + f(b)) / 2.0;
  var i = 1;
  while i < n {
    var x = a + (i as Float64) * h;
    sum = sum + f(x);
    i = i + 1;
  }
  return sum * h;
}

fn test_numerical() -> Int {
  var score = 0;
  var s4 = sqrt_newton(4.0, 0.0001);
  var s4_ok = s4 > 1.9 && s4 < 2.1;
  if s4_ok { score = score + 1; }

  var s9 = sqrt_newton(9.0, 0.0001);
  var s9_ok = s9 > 2.9 && s9 < 3.1;
  if s9_ok { score = score + 1; }

  var s0 = sqrt_newton(0.0, 0.0001);
  if s0 == 0.0 { score = score + 1; }

  if sqrt_newton(-1.0, 0.0001) == -1.0 { score = score + 1; }

  if abs_float(3.14) == 3.14 { score = score + 1; }
  if abs_float(-2.71) == 2.71 { score = score + 1; }
  if abs_float(0.0) == 0.0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 12: Modular Arithmetic
// ============================================================

pub fn mod_add(a: Int, b: Int, m: Int) -> Int {
  return (a + b) % m;
}

pub fn mod_mul(a: Int, b: Int, m: Int) -> Int {
  return (a * b) % m;
}

pub fn mod_inv(a: Int, m: Int) -> Int {
  var t = 0;
  var newt = 1;
  var r = m;
  var newr = a % m;
  while newr != 0 {
    var q = r / newr;
    var temp_t = t - q * newt;
    t = newt;
    newt = temp_t;
    var temp_r = r - q * newr;
    r = newr;
    newr = temp_r;
  }
  if r > 1 {
    return -1;
  }
  if t < 0 {
    t = t + m;
  }
  return t;
}

fn test_modular() -> Int {
  var score = 0;
  if mod_add(7, 8, 10) == 5 { score = score + 1; }
  if mod_add(5, 5, 7) == 3 { score = score + 1; }
  if mod_mul(3, 4, 11) == 1 { score = score + 1; }
  if mod_mul(7, 8, 13) == 4 { score = score + 1; }

  var inv35 = mod_inv(3, 5);
  if inv35 > 0 && (3 * inv35) % 5 == 1 { score = score + 1; }

  var inv711 = mod_inv(7, 11);
  if inv711 > 0 && (7 * inv711) % 11 == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 13: Bitwise Operations (simulated with arithmetic)
// ============================================================

pub fn bit_and(a: Int, b: Int) -> Int {
  var result = 0;
  var shift = 0;
  var aa = a;
  var bb = b;
  while aa > 0 || bb > 0 {
    if aa % 2 == 1 && bb % 2 == 1 {
      result = result + power_iter(2, shift);
    }
    aa = aa / 2;
    bb = bb / 2;
    shift = shift + 1;
  }
  return result;
}

pub fn bit_or(a: Int, b: Int) -> Int {
  var result = 0;
  var shift = 0;
  var aa = a;
  var bb = b;
  while aa > 0 || bb > 0 {
    if aa % 2 == 1 || bb % 2 == 1 {
      result = result + power_iter(2, shift);
    }
    aa = aa / 2;
    bb = bb / 2;
    shift = shift + 1;
  }
  return result;
}

pub fn bit_xor(a: Int, b: Int) -> Int {
  var result = 0;
  var shift = 0;
  var aa = a;
  var bb = b;
  while aa > 0 || bb > 0 {
    if aa % 2 != bb % 2 {
      result = result + power_iter(2, shift);
    }
    aa = aa / 2;
    bb = bb / 2;
    shift = shift + 1;
  }
  return result;
}

fn test_bitwise() -> Int {
  var score = 0;
  if bit_and(6, 3) == 2 { score = score + 1; }
  if bit_and(12, 10) == 8 { score = score + 1; }
  if bit_and(0, 5) == 0 { score = score + 1; }
  if bit_and(7, 7) == 7 { score = score + 1; }

  if bit_or(6, 3) == 7 { score = score + 1; }
  if bit_or(8, 1) == 9 { score = score + 1; }
  if bit_or(0, 5) == 5 { score = score + 1; }

  if bit_xor(6, 3) == 5 { score = score + 1; }
  if bit_xor(5, 5) == 0 { score = score + 1; }
  if bit_xor(0, 7) == 7 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 14: Interpolation and Mapping
// ============================================================

pub fn lerp(a: Int, b: Int, t: Int) -> Int {
  return a + (b - a) * t;
}

pub fn lerp_float(a: Float64, b: Float64, t: Float64) -> Float64 {
  return a + (b - a) * t;
}

pub fn map_range(val: Int, in_min: Int, in_max: Int, out_min: Int, out_max: Int) -> Int {
  return out_min + (val - in_min) * (out_max - out_min) / (in_max - in_min);
}

fn test_interpolation() -> Int {
  var score = 0;
  if lerp(0, 10, 0) == 0 { score = score + 1; }
  if lerp(0, 10, 1) == 10 { score = score + 1; }
  if lerp(0, 10, 2) == 20 { score = score + 1; }
  if lerp(10, 20, 3) == 40 { score = score + 1; }

  if map_range(5, 0, 10, 0, 100) == 50 { score = score + 1; }
  if map_range(0, 0, 10, 0, 100) == 0 { score = score + 1; }
  if map_range(10, 0, 10, 0, 100) == 100 { score = score + 1; }
  if map_range(5, 0, 100, 0, 10) == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 15: Aggregate Scoring Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_basic_arithmetic();
  total = total + s1;
  max_score = max_score + 27;

  var s2 = test_float_arithmetic();
  total = total + s2;
  max_score = max_score + 9;

  var s3 = test_factorial();
  total = total + s3;
  max_score = max_score + 12;

  var s4 = test_fibonacci();
  total = total + s4;
  max_score = max_score + 14;

  var s5 = test_gcd_lcm();
  total = total + s5;
  max_score = max_score + 15;

  var s6 = test_power();
  total = total + s6;
  max_score = max_score + 15;

  var s7 = test_combinatorics();
  total = total + s7;
  max_score = max_score + 11;

  var s8 = test_number_theory();
  total = total + s8;
  max_score = max_score + 32;

  var s9 = test_sequences();
  total = total + s9;
  max_score = max_score + 27;

  var s10 = test_parity();
  total = total + s10;
  max_score = max_score + 26;

  var s11 = test_numerical();
  total = total + s11;
  max_score = max_score + 7;

  var s12 = test_modular();
  total = total + s12;
  max_score = max_score + 6;

  var s13 = test_bitwise();
  total = total + s13;
  max_score = max_score + 10;

  var s14 = test_interpolation();
  total = total + s14;
  max_score = max_score + 8;

  return BenchResult{
    name: "math",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
