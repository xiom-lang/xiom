// XIOM — Ecosystem Algorithm Hardening Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Self-contained algorithm test suite. Exercises recursion, loops,
// match, Option/Result, Vec operations, references, and numeric edge cases.

module tests.ecosystem.test_algo

// ============================================================================
// Algorithm Implementations
// ============================================================================

fn binary_search(arr: &Vec[Int], target: Int) -> Option[Int] {
  var lo = 0;
  var hi = arr.len();
  while lo < hi {
    var mid = (lo + hi) / 2;
    var val = arr[mid];
    if val == target { return Some(mid); }
    elif val < target { lo = mid + 1; }
    else { hi = mid; }
  }
  return None;
}

fn quicksort(arr: Vec[Int]) -> Vec[Int] {
  if arr.len() <= 1 { return arr; }
  var pivot = arr[arr.len() - 1];
  var left = Vec[Int].new();
  var right = Vec[Int].new();
  var i = 0;
  while i < arr.len() - 1 {
    if arr[i] <= pivot { left.push(arr[i]); }
    else { right.push(arr[i]); }
    i = i + 1;
  }
  var sorted_left = quicksort(left);
  var sorted_right = quicksort(right);
  sorted_left.push(pivot);
  return concat(sorted_left, sorted_right);
}

fn concat(a: Vec[Int], b: Vec[Int]) -> Vec[Int] {
  var result = a;
  var i = 0;
  while i < b.len() {
    result.push(b[i]);
    i = i + 1;
  }
  return result;
}

fn merge(left: Vec[Int], right: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  var j = 0;
  while i < left.len() && j < right.len() {
    if left[i] <= right[j] {
      result.push(left[i]);
      i = i + 1;
    } else {
      result.push(right[j]);
      j = j + 1;
    }
  }
  while i < left.len() {
    result.push(left[i]);
    i = i + 1;
  }
  while j < right.len() {
    result.push(right[j]);
    j = j + 1;
  }
  return result;
}

fn merge_sort(arr: Vec[Int]) -> Vec[Int] {
  if arr.len() <= 1 { return arr; }
  var mid = arr.len() / 2;
  var left = Vec[Int].new();
  var right = Vec[Int].new();
  var i = 0;
  while i < mid {
    left.push(arr[i]);
    i = i + 1;
  }
  while i < arr.len() {
    right.push(arr[i]);
    i = i + 1;
  }
  var sorted_left = merge_sort(left);
  var sorted_right = merge_sort(right);
  return merge(sorted_left, sorted_right);
}

fn gcd(a: Int, b: Int) -> Int
requires: a>0, b>0 {
  var x = a;
  var y = b;
  while y != 0 {
    var t = y;
    y = x % y;
    x = t;
  }
  return x;
}

fn fibonacci(n: Int) -> Int
requires: n>=0 {
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  var a = 0;
  var b = 1;
  var i = 2;
  while i <= n {
    var c = a + b;
    a = b;
    b = c;
    i = i + 1;
  }
  return b;
}

fn sieve_of_eratosthenes(n: Int) -> Vec[Int]
requires: n>=2 {
  var is_prime = Vec[Bool].new();
  var i = 0;
  while i <= n {
    is_prime.push(true);
    i = i + 1;
  }
  is_prime[0] = false;
  is_prime[1] = false;
  i = 2;
  while i * i <= n {
    if is_prime[i] {
      var j = i * i;
      while j <= n {
        is_prime[j] = false;
        j = j + i;
      }
    }
    i = i + 1;
  }
  var primes = Vec[Int].new();
  i = 2;
  while i <= n {
    if is_prime[i] { primes.push(i); }
    i = i + 1;
  }
  return primes;
}

fn is_prime(n: Int) -> Bool
requires: n>=0 {
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

fn factorial(n: Int) -> Int
requires: n>=0, n<=20 {
  var result = 1;
  var i = 2;
  while i <= n {
    result = result * i;
    i = i + 1;
  }
  return result;
}

fn lcm(a: Int, b: Int) -> Int {
  var g = gcd(a, b);
  return (a / g) * b;
}

fn power(base: Int, exp: Int) -> Int {
  var result = 1;
  var e = exp;
  var b = base;
  while e > 0 {
    if e % 2 == 1 { result = result * b; }
    b = b * b;
    e = e / 2;
  }
  return result;
}

fn is_sorted(arr: &Vec[Int]) -> Bool {
  var i = 1;
  while i < arr.len() {
    if arr[i] < arr[i - 1] { return false; }
    i = i + 1;
  }
  return true;
}

fn reverse(arr: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = arr.len();
  while i > 0 {
    i = i - 1;
    result.push(arr[i]);
  }
  return result;
}

fn unique(sorted: Vec[Int]) -> Vec[Int] {
  if sorted.len() == 0 { return sorted; }
  var result = Vec[Int].new();
  result.push(sorted[0]);
  var i = 1;
  while i < sorted.len() {
    if sorted[i] != sorted[i - 1] {
      result.push(sorted[i]);
    }
    i = i + 1;
  }
  return result;
}

fn max_subarray_sum(arr: &Vec[Int]) -> Int {
  if arr.len() == 0 { return 0; }
  var max_so_far = arr[0];
  var max_ending = arr[0];
  var i = 1;
  while i < arr.len() {
    if max_ending + arr[i] > arr[i] {
      max_ending = max_ending + arr[i];
    } else {
      max_ending = arr[i];
    }
    if max_ending > max_so_far { max_so_far = max_ending; }
    i = i + 1;
  }
  return max_so_far;
}

// ============================================================================
// Vector Helper
// ============================================================================

fn vec_eq(a: &Vec[Int], b: &Vec[Int]) -> Bool {
  if a.len() != b.len() { return false; }
  var i = 0;
  while i < a.len() {
    if a[i] != b[i] { return false; }
    i = i + 1;
  }
  return true;
}

// ============================================================================
// Test Functions — Binary Search
// ============================================================================

fn test_binary_search_found() -> Bool {
  let arr = [1, 3, 5, 7, 9, 11, 13];
  let result = binary_search(&arr, 7);
  return result == Some(3);
}

fn test_binary_search_not_found() -> Bool {
  let arr = [1, 3, 5, 7, 9];
  let result = binary_search(&arr, 4);
  return result.is_none();
}

fn test_binary_search_first() -> Bool {
  let arr = [2, 4, 6, 8];
  let result = binary_search(&arr, 2);
  return result == Some(0);
}

fn test_binary_search_last() -> Bool {
  let arr = [2, 4, 6, 8];
  let result = binary_search(&arr, 8);
  return result == Some(3);
}

fn test_binary_search_empty() -> Bool {
  let arr = Vec[Int].new();
  let result = binary_search(&arr, 5);
  return result.is_none();
}

fn test_binary_search_single_found() -> Bool {
  let arr = [42];
  let result = binary_search(&arr, 42);
  return result == Some(0);
}

fn test_binary_search_single_not_found() -> Bool {
  let arr = [42];
  let result = binary_search(&arr, 7);
  return result.is_none();
}

// ============================================================================
// Test Functions — Quicksort
// ============================================================================

fn test_quicksort_empty() -> Bool {
  let sorted = quicksort(Vec[Int].new());
  return sorted.len() == 0;
}

fn test_quicksort_single() -> Bool {
  let arr = [5];
  let sorted = quicksort(arr);
  return sorted.len() == 1 && sorted[0] == 5;
}

fn test_quicksort_sorted() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  let sorted = quicksort(arr);
  let expected = [1, 2, 3, 4, 5];
  return vec_eq(&sorted, &expected);
}

fn test_quicksort_reverse() -> Bool {
  let arr = [5, 4, 3, 2, 1];
  let sorted = quicksort(arr);
  let expected = [1, 2, 3, 4, 5];
  return vec_eq(&sorted, &expected);
}

fn test_quicksort_duplicates() -> Bool {
  let arr = [3, 1, 3, 2, 1, 2];
  let sorted = quicksort(arr);
  let expected = [1, 1, 2, 2, 3, 3];
  return vec_eq(&sorted, &expected);
}

fn test_quicksort_random() -> Bool {
  let arr = [9, 3, 7, 1, 5, 4, 8, 2, 6];
  let sorted = quicksort(arr);
  let expected = [1, 2, 3, 4, 5, 6, 7, 8, 9];
  return vec_eq(&sorted, &expected);
}

fn test_quicksort_all_equal() -> Bool {
  let arr = [7, 7, 7, 7];
  let sorted = quicksort(arr);
  let expected = [7, 7, 7, 7];
  return vec_eq(&sorted, &expected);
}

// ============================================================================
// Test Functions — Merge Sort
// ============================================================================

fn test_merge_sort_empty() -> Bool {
  let sorted = merge_sort(Vec[Int].new());
  return sorted.len() == 0;
}

fn test_merge_sort_single() -> Bool {
  let arr = [8];
  let sorted = merge_sort(arr);
  return sorted[0] == 8 && sorted.len() == 1;
}

fn test_merge_sort_sorted() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  let sorted = merge_sort(arr);
  let expected = [1, 2, 3, 4, 5];
  return vec_eq(&sorted, &expected);
}

fn test_merge_sort_reverse() -> Bool {
  let arr = [9, 7, 5, 3, 1];
  let sorted = merge_sort(arr);
  let expected = [1, 3, 5, 7, 9];
  return vec_eq(&sorted, &expected);
}

fn test_merge_sort_duplicates() -> Bool {
  let arr = [4, 2, 4, 1, 2, 3];
  let sorted = merge_sort(arr);
  let expected = [1, 2, 2, 3, 4, 4];
  return vec_eq(&sorted, &expected);
}

fn test_merge_sort_two_elements() -> Bool {
  let arr = [99, 1];
  let sorted = merge_sort(arr);
  let expected = [1, 99];
  return vec_eq(&sorted, &expected);
}

// ============================================================================
// Test Functions — GCD
// ============================================================================

fn test_gcd_basic() -> Bool {
  return gcd(12, 8) == 4;
}

fn test_gcd_coprime() -> Bool {
  return gcd(7, 13) == 1;
}

fn test_gcd_one_divisible() -> Bool {
  return gcd(15, 5) == 5;
}

fn test_gcd_equal() -> Bool {
  return gcd(10, 10) == 10;
}

fn test_gcd_large() -> Bool {
  return gcd(1071, 462) == 21;
}

// ============================================================================
// Test Functions — Fibonacci
// ============================================================================

fn test_fibonacci_zero() -> Bool {
  return fibonacci(0) == 0;
}

fn test_fibonacci_one() -> Bool {
  return fibonacci(1) == 1;
}

fn test_fibonacci_two() -> Bool {
  return fibonacci(2) == 1;
}

fn test_fibonacci_three() -> Bool {
  return fibonacci(3) == 2;
}

fn test_fibonacci_six() -> Bool {
  return fibonacci(6) == 8;
}

fn test_fibonacci_ten() -> Bool {
  return fibonacci(10) == 55;
}

fn test_fibonacci_fifteen() -> Bool {
  return fibonacci(15) == 610;
}

// ============================================================================
// Test Functions — Sieve of Eratosthenes
// ============================================================================

fn test_sieve_primes_up_to_10() -> Bool {
  let primes = sieve_of_eratosthenes(10);
  let expected = [2, 3, 5, 7];
  return vec_eq(&primes, &expected);
}

fn test_sieve_primes_up_to_30() -> Bool {
  let primes = sieve_of_eratosthenes(30);
  let expected = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
  return vec_eq(&primes, &expected);
}

fn test_sieve_min_n() -> Bool {
  let primes = sieve_of_eratosthenes(2);
  let expected = [2];
  return vec_eq(&primes, &expected);
}

fn test_sieve_large() -> Bool {
  let primes = sieve_of_eratosthenes(100);
  return primes.len() == 25;
}

// ============================================================================
// Test Functions — Is Prime
// ============================================================================

fn test_is_prime_zero() -> Bool {
  return !is_prime(0);
}

fn test_is_prime_one() -> Bool {
  return !is_prime(1);
}

fn test_is_prime_two() -> Bool {
  return is_prime(2);
}

fn test_is_prime_small_primes() -> Bool {
  return is_prime(3) && is_prime(5) && is_prime(7) && is_prime(11);
}

fn test_is_prime_composites() -> Bool {
  return !is_prime(4) && !is_prime(6) && !is_prime(9) && !is_prime(15);
}

fn test_is_prime_large_prime() -> Bool {
  return is_prime(97);
}

fn test_is_prime_large_composite() -> Bool {
  return !is_prime(100);
}

fn test_is_prime_perfect_square() -> Bool {
  return !is_prime(49);
}

// ============================================================================
// Test Functions — Factorial
// ============================================================================

fn test_factorial_zero() -> Bool {
  return factorial(0) == 1;
}

fn test_factorial_one() -> Bool {
  return factorial(1) == 1;
}

fn test_factorial_five() -> Bool {
  return factorial(5) == 120;
}

fn test_factorial_ten() -> Bool {
  return factorial(10) == 3628800;
}

fn test_factorial_twenty() -> Bool {
  return factorial(20) > 0;
}

// ============================================================================
// Test Functions — LCM
// ============================================================================

fn test_lcm_basic() -> Bool {
  return lcm(12, 8) == 24;
}

fn test_lcm_coprime() -> Bool {
  return lcm(7, 5) == 35;
}

fn test_lcm_one_divisible() -> Bool {
  return lcm(6, 3) == 6;
}

fn test_lcm_equal() -> Bool {
  return lcm(9, 9) == 9;
}

fn test_lcm_one() -> Bool {
  return lcm(1, 42) == 42;
}

// ============================================================================
// Test Functions — Power
// ============================================================================

fn test_power_exp_zero() -> Bool {
  return power(5, 0) == 1;
}

fn test_power_exp_one() -> Bool {
  return power(7, 1) == 7;
}

fn test_power_positive() -> Bool {
  return power(2, 10) == 1024;
}

fn test_power_large() -> Bool {
  return power(3, 5) == 243;
}

fn test_power_base_one() -> Bool {
  return power(1, 100) == 1;
}

fn test_power_base_zero() -> Bool {
  return power(0, 5) == 0;
}

// ============================================================================
// Test Functions — Is Sorted
// ============================================================================

fn test_is_sorted_empty() -> Bool {
  let arr = Vec[Int].new();
  return is_sorted(&arr);
}

fn test_is_sorted_single() -> Bool {
  let arr = [7];
  return is_sorted(&arr);
}

fn test_is_sorted_true() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  return is_sorted(&arr);
}

fn test_is_sorted_false() -> Bool {
  let arr = [1, 3, 2, 4];
  return !is_sorted(&arr);
}

fn test_is_sorted_with_duplicates() -> Bool {
  let arr = [1, 1, 2, 2, 3];
  return is_sorted(&arr);
}

fn test_is_sorted_descending() -> Bool {
  let arr = [5, 4, 3, 2, 1];
  return !is_sorted(&arr);
}

// ============================================================================
// Test Functions — Reverse
// ============================================================================

fn test_reverse_empty() -> Bool {
  let rev = reverse(Vec[Int].new());
  return rev.len() == 0;
}

fn test_reverse_single() -> Bool {
  let arr = [42];
  let rev = reverse(arr);
  return rev.len() == 1 && rev[0] == 42;
}

fn test_reverse_multiple() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  let rev = reverse(arr);
  let expected = [5, 4, 3, 2, 1];
  return vec_eq(&rev, &expected);
}

fn test_reverse_two() -> Bool {
  let arr = [1, 2];
  let rev = reverse(arr);
  let expected = [2, 1];
  return vec_eq(&rev, &expected);
}

// ============================================================================
// Test Functions — Concat
// ============================================================================

fn test_concat_both_nonempty() -> Bool {
  let a = [1, 2, 3];
  let b = [4, 5, 6];
  let result = concat(a, b);
  let expected = [1, 2, 3, 4, 5, 6];
  return vec_eq(&result, &expected);
}

fn test_concat_first_empty() -> Bool {
  let a = Vec[Int].new();
  let b = [1, 2, 3];
  let result = concat(a, b);
  let expected = [1, 2, 3];
  return vec_eq(&result, &expected);
}

fn test_concat_second_empty() -> Bool {
  let a = [1, 2, 3];
  let b = Vec[Int].new();
  let result = concat(a, b);
  let expected = [1, 2, 3];
  return vec_eq(&result, &expected);
}

fn test_concat_both_empty() -> Bool {
  let a = Vec[Int].new();
  let b = Vec[Int].new();
  let result = concat(a, b);
  return result.len() == 0;
}

// ============================================================================
// Test Functions — Unique
// ============================================================================

fn test_unique_empty() -> Bool {
  let result = unique(Vec[Int].new());
  return result.len() == 0;
}

fn test_unique_single() -> Bool {
  let arr = [5];
  let result = unique(arr);
  let expected = [5];
  return vec_eq(&result, &expected);
}

fn test_unique_no_duplicates() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  let result = unique(arr);
  let expected = [1, 2, 3, 4, 5];
  return vec_eq(&result, &expected);
}

fn test_unique_with_duplicates() -> Bool {
  let arr = [1, 1, 2, 2, 3, 3, 3];
  let result = unique(arr);
  let expected = [1, 2, 3];
  return vec_eq(&result, &expected);
}

fn test_unique_all_same() -> Bool {
  let arr = [7, 7, 7, 7];
  let result = unique(arr);
  let expected = [7];
  return vec_eq(&result, &expected);
}

// ============================================================================
// Test Functions — Max Subarray Sum (Kadane)
// ============================================================================

fn test_max_subarray_all_positive() -> Bool {
  let arr = [1, 2, 3, 4, 5];
  return max_subarray_sum(&arr) == 15;
}

fn test_max_subarray_all_negative() -> Bool {
  let arr = [-5, -2, -3, -1];
  return max_subarray_sum(&arr) == -1;
}

fn test_max_subarray_mixed() -> Bool {
  let arr = [-2, 1, -3, 4, -1, 2, 1, -5, 4];
  return max_subarray_sum(&arr) == 6;
}

fn test_max_subarray_single_positive() -> Bool {
  let arr = [42];
  return max_subarray_sum(&arr) == 42;
}

fn test_max_subarray_single_negative() -> Bool {
  let arr = [-42];
  return max_subarray_sum(&arr) == -42;
}

fn test_max_subarray_empty() -> Bool {
  let arr = Vec[Int].new();
  return max_subarray_sum(&arr) == 0;
}

// ============================================================================
// Integration Tests — Cross-Algorithm
// ============================================================================

fn test_sort_then_binary_search() -> Bool {
  let arr = [9, 3, 7, 1, 5];
  let sorted = quicksort(arr);
  let result = binary_search(&sorted, 7);
  return result.is_some() && result.unwrap() >= 0;
}

fn test_unique_on_sorted_output() -> Bool {
  let arr = [3, 1, 3, 2, 1, 4, 2];
  let sorted = merge_sort(arr);
  let dedup = unique(sorted);
  let expected = [1, 2, 3, 4];
  return vec_eq(&dedup, &expected);
}

fn test_is_prime_and_sieve_agree() -> Bool {
  let primes = sieve_of_eratosthenes(50);
  var i = 2;
  while i <= 50 {
    var found = binary_search(&primes, i).is_some();
    if found != is_prime(i) { return false; }
    i = i + 1;
  }
  return true;
}

fn test_gcd_and_lcm_relation() -> Bool {
  var a = 12;
  var b = 18;
  var g = gcd(a, b);
  var l = lcm(a, b);
  return (a / g) * b == l;
}

fn test_factorial_identity() -> Bool {
  return factorial(5) * 6 == factorial(6);
}

// ============================================================================
// Main — Run All Tests
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1; if test_binary_search_found() { passed = passed + 1; }
  total = total + 1; if test_binary_search_not_found() { passed = passed + 1; }
  total = total + 1; if test_binary_search_first() { passed = passed + 1; }
  total = total + 1; if test_binary_search_last() { passed = passed + 1; }
  total = total + 1; if test_binary_search_empty() { passed = passed + 1; }
  total = total + 1; if test_binary_search_single_found() { passed = passed + 1; }
  total = total + 1; if test_binary_search_single_not_found() { passed = passed + 1; }

  total = total + 1; if test_quicksort_empty() { passed = passed + 1; }
  total = total + 1; if test_quicksort_single() { passed = passed + 1; }
  total = total + 1; if test_quicksort_sorted() { passed = passed + 1; }
  total = total + 1; if test_quicksort_reverse() { passed = passed + 1; }
  total = total + 1; if test_quicksort_duplicates() { passed = passed + 1; }
  total = total + 1; if test_quicksort_random() { passed = passed + 1; }
  total = total + 1; if test_quicksort_all_equal() { passed = passed + 1; }

  total = total + 1; if test_merge_sort_empty() { passed = passed + 1; }
  total = total + 1; if test_merge_sort_single() { passed = passed + 1; }
  total = total + 1; if test_merge_sort_sorted() { passed = passed + 1; }
  total = total + 1; if test_merge_sort_reverse() { passed = passed + 1; }
  total = total + 1; if test_merge_sort_duplicates() { passed = passed + 1; }
  total = total + 1; if test_merge_sort_two_elements() { passed = passed + 1; }

  total = total + 1; if test_gcd_basic() { passed = passed + 1; }
  total = total + 1; if test_gcd_coprime() { passed = passed + 1; }
  total = total + 1; if test_gcd_one_divisible() { passed = passed + 1; }
  total = total + 1; if test_gcd_equal() { passed = passed + 1; }
  total = total + 1; if test_gcd_large() { passed = passed + 1; }

  total = total + 1; if test_fibonacci_zero() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_one() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_two() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_three() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_six() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_ten() { passed = passed + 1; }
  total = total + 1; if test_fibonacci_fifteen() { passed = passed + 1; }

  total = total + 1; if test_sieve_primes_up_to_10() { passed = passed + 1; }
  total = total + 1; if test_sieve_primes_up_to_30() { passed = passed + 1; }
  total = total + 1; if test_sieve_min_n() { passed = passed + 1; }
  total = total + 1; if test_sieve_large() { passed = passed + 1; }

  total = total + 1; if test_is_prime_zero() { passed = passed + 1; }
  total = total + 1; if test_is_prime_one() { passed = passed + 1; }
  total = total + 1; if test_is_prime_two() { passed = passed + 1; }
  total = total + 1; if test_is_prime_small_primes() { passed = passed + 1; }
  total = total + 1; if test_is_prime_composites() { passed = passed + 1; }
  total = total + 1; if test_is_prime_large_prime() { passed = passed + 1; }
  total = total + 1; if test_is_prime_large_composite() { passed = passed + 1; }
  total = total + 1; if test_is_prime_perfect_square() { passed = passed + 1; }

  total = total + 1; if test_factorial_zero() { passed = passed + 1; }
  total = total + 1; if test_factorial_one() { passed = passed + 1; }
  total = total + 1; if test_factorial_five() { passed = passed + 1; }
  total = total + 1; if test_factorial_ten() { passed = passed + 1; }
  total = total + 1; if test_factorial_twenty() { passed = passed + 1; }

  total = total + 1; if test_lcm_basic() { passed = passed + 1; }
  total = total + 1; if test_lcm_coprime() { passed = passed + 1; }
  total = total + 1; if test_lcm_one_divisible() { passed = passed + 1; }
  total = total + 1; if test_lcm_equal() { passed = passed + 1; }
  total = total + 1; if test_lcm_one() { passed = passed + 1; }

  total = total + 1; if test_power_exp_zero() { passed = passed + 1; }
  total = total + 1; if test_power_exp_one() { passed = passed + 1; }
  total = total + 1; if test_power_positive() { passed = passed + 1; }
  total = total + 1; if test_power_large() { passed = passed + 1; }
  total = total + 1; if test_power_base_one() { passed = passed + 1; }
  total = total + 1; if test_power_base_zero() { passed = passed + 1; }

  total = total + 1; if test_is_sorted_empty() { passed = passed + 1; }
  total = total + 1; if test_is_sorted_single() { passed = passed + 1; }
  total = total + 1; if test_is_sorted_true() { passed = passed + 1; }
  total = total + 1; if test_is_sorted_false() { passed = passed + 1; }
  total = total + 1; if test_is_sorted_with_duplicates() { passed = passed + 1; }
  total = total + 1; if test_is_sorted_descending() { passed = passed + 1; }

  total = total + 1; if test_reverse_empty() { passed = passed + 1; }
  total = total + 1; if test_reverse_single() { passed = passed + 1; }
  total = total + 1; if test_reverse_multiple() { passed = passed + 1; }
  total = total + 1; if test_reverse_two() { passed = passed + 1; }

  total = total + 1; if test_concat_both_nonempty() { passed = passed + 1; }
  total = total + 1; if test_concat_first_empty() { passed = passed + 1; }
  total = total + 1; if test_concat_second_empty() { passed = passed + 1; }
  total = total + 1; if test_concat_both_empty() { passed = passed + 1; }

  total = total + 1; if test_unique_empty() { passed = passed + 1; }
  total = total + 1; if test_unique_single() { passed = passed + 1; }
  total = total + 1; if test_unique_no_duplicates() { passed = passed + 1; }
  total = total + 1; if test_unique_with_duplicates() { passed = passed + 1; }
  total = total + 1; if test_unique_all_same() { passed = passed + 1; }

  total = total + 1; if test_max_subarray_all_positive() { passed = passed + 1; }
  total = total + 1; if test_max_subarray_all_negative() { passed = passed + 1; }
  total = total + 1; if test_max_subarray_mixed() { passed = passed + 1; }
  total = total + 1; if test_max_subarray_single_positive() { passed = passed + 1; }
  total = total + 1; if test_max_subarray_single_negative() { passed = passed + 1; }
  total = total + 1; if test_max_subarray_empty() { passed = passed + 1; }

  total = total + 1; if test_sort_then_binary_search() { passed = passed + 1; }
  total = total + 1; if test_unique_on_sorted_output() { passed = passed + 1; }
  total = total + 1; if test_is_prime_and_sieve_agree() { passed = passed + 1; }
  total = total + 1; if test_gcd_and_lcm_relation() { passed = passed + 1; }
  total = total + 1; if test_factorial_identity() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
