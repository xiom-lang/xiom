// XIOM -- Collections Stress Benchmark
// Exercises Vec, Map, Set operations at scale.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.collections

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Vec Operations at Scale
// ============================================================

pub fn vec_range(start: Int, end: Int) -> Vec[Int] {
  var v = Vec[Int].new();
  var i = start;
  while i < end { v.push(i); i = i + 1; }
  return v;
}

pub fn vec_sum(v: &Vec[Int]) -> Int {
  var sum = 0;
  var i = 0;
  while i < v.len() { sum = sum + v[i]; i = i + 1; }
  return sum;
}

pub fn vec_product(v: &Vec[Int]) -> Int {
  if v.len() == 0 { return 0; }
  var prod = 1;
  var i = 0;
  while i < v.len() { prod = prod * v[i]; i = i + 1; }
  return prod;
}

pub fn vec_copy(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() { result.push(v[i]); i = i + 1; }
  return result;
}

pub fn vec_append(a: &Vec[Int], b: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < a.len() { result.push(a[i]); i = i + 1; }
  i = 0;
  while i < b.len() { result.push(b[i]); i = i + 1; }
  return result;
}

pub fn vec_zip(a: &Vec[Int], b: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var min_len = a.len();
  if b.len() < min_len { min_len = b.len(); }
  var i = 0;
  while i < min_len { result.push(a[i] + b[i]); i = i + 1; }
  return result;
}

fn test_vec_ops() -> Int {
  var score = 0;
  var v = vec_range(1, 11);
  if v.len() == 10 { score = score + 1; }
  if v[0] == 1 { score = score + 1; }
  if v[9] == 10 { score = score + 1; }
  if vec_sum(&v) == 55 { score = score + 1; }

  var v2 = vec_range(1, 6);
  if vec_product(&v2) == 120 { score = score + 1; }

  var v_copy = vec_copy(&v);
  if v_copy.len() == 10 { score = score + 1; }
  if v_copy[0] == 1 { score = score + 1; }

  var a = [1, 2, 3];
  var b = [4, 5, 6];
  var appended = vec_append(&a, &b);
  if appended.len() == 6 { score = score + 1; }
  if appended[0] == 1 { score = score + 1; }
  if appended[5] == 6 { score = score + 1; }

  var zipped = vec_zip(&a, &b);
  if zipped.len() == 3 { score = score + 1; }
  if zipped[0] == 5 { score = score + 1; }
  if zipped[2] == 9 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Sorting Algorithms
// ============================================================

pub fn bubble_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(&v);
  var n = arr.len();
  var i = 0;
  while i < n {
    var j = 0;
    while j < n - i - 1 {
      if arr[j] > arr[j + 1] { var temp = arr[j]; arr[j] = arr[j + 1]; arr[j + 1] = temp; }
      j = j + 1;
    }
    i = i + 1;
  }
  return arr;
}

pub fn insertion_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(&v);
  var i = 1;
  while i < arr.len() {
    var key = arr[i];
    var j = i;
    while j > 0 && arr[j - 1] > key { arr[j] = arr[j - 1]; j = j - 1; }
    arr[j] = key;
    i = i + 1;
  }
  return arr;
}

pub fn is_sorted(v: &Vec[Int]) -> Bool {
  var i = 1;
  while i < v.len() { if v[i - 1] > v[i] { return false; } i = i + 1; }
  return true;
}

fn test_sorting() -> Int {
  var score = 0;
  var unsorted = [5, 2, 8, 1, 9, 3, 7, 4, 6];
  var sorted_bubble = bubble_sort(unsorted);
  if is_sorted(&sorted_bubble) { score = score + 1; }
  if sorted_bubble[0] == 1 { score = score + 1; }
  if sorted_bubble[sorted_bubble.len() - 1] == 9 { score = score + 1; }
  if sorted_bubble.len() == 9 { score = score + 1; }

  var unsorted2 = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  var sorted_insert = insertion_sort(unsorted2);
  if is_sorted(&sorted_insert) { score = score + 1; }
  if sorted_insert[0] == 1 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 3: Searching
// ============================================================

pub fn linear_search(v: &Vec[Int], target: Int) -> Int {
  var i = 0;
  while i < v.len() { if v[i] == target { return i; } i = i + 1; }
  return -1;
}

pub fn binary_search(sorted: &Vec[Int], target: Int) -> Int {
  var lo = 0;
  var hi = sorted.len();
  if hi == 0 { return -1; }
  hi = hi - 1;
  while lo <= hi {
    var mid = (lo + hi) / 2;
    if sorted[mid] == target { return mid; }
    if sorted[mid] < target { lo = mid + 1; } else { hi = mid - 1; }
  }
  return -1;
}

pub fn count_occurrences(v: &Vec[Int], target: Int) -> Int {
  var count = 0;
  var i = 0;
  while i < v.len() { if v[i] == target { count = count + 1; } i = i + 1; }
  return count;
}

fn test_searching() -> Int {
  var score = 0;
  var nums = [3, 7, 2, 9, 1, 5, 8, 4, 6];
  if linear_search(&nums, 7) == 1 { score = score + 1; }
  if linear_search(&nums, 99) == -1 { score = score + 1; }
  var empty: Vec[Int] = [];
  if linear_search(&empty, 5) == -1 { score = score + 1; }
  var sorted = [1, 3, 5, 7, 9, 11, 13, 15];
  if binary_search(&sorted, 7) == 3 { score = score + 1; }
  if binary_search(&sorted, 1) == 0 { score = score + 1; }
  if binary_search(&sorted, 15) == 7 { score = score + 1; }
  if binary_search(&sorted, 8) == -1 { score = score + 1; }
  if binary_search(&empty, 5) == -1 { score = score + 1; }
  var repeats = [1, 2, 2, 3, 2, 4, 2];
  if count_occurrences(&repeats, 2) == 4 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 4: 2D Vector Matrix Operations
// ============================================================

pub fn matrix_new(rows: Int, cols: Int) -> Vec[Vec[Int]] {
  var m = Vec[Vec[Int]].new();
  var i = 0;
  while i < rows {
    var row = Vec[Int].new();
    var j = 0;
    while j < cols { row.push(0); j = j + 1; }
    m.push(row);
    i = i + 1;
  }
  return m;
}

pub fn matrix_fill(mut m: Vec[Vec[Int]], val: Int) -> Vec[Vec[Int]] {
  var i = 0;
  while i < m.len() {
    var j = 0;
    while j < m[i].len() { m[i][j] = val; j = j + 1; }
    i = i + 1;
  }
  return m;
}

pub fn matrix_sum(m: &Vec[Vec[Int]]) -> Int {
  var total = 0;
  var i = 0;
  while i < m.len() {
    var j = 0;
    while j < m[i].len() { total = total + m[i][j]; j = j + 1; }
    i = i + 1;
  }
  return total;
}

fn test_matrix() -> Int {
  var score = 0;
  var m = matrix_new(3, 4);
  if m.len() == 3 { score = score + 1; }
  if m[0].len() == 4 { score = score + 1; }
  var m2 = matrix_fill(m, 5);
  if m2[0][0] == 5 { score = score + 1; }
  if m2[2][3] == 5 { score = score + 1; }
  if matrix_sum(&m2) == 60 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 5: Partition and Group
// ============================================================

pub fn partition(v: &Vec[Int], pred: fn(Int) -> Bool) -> (Vec[Int], Vec[Int]) {
  var yes = Vec[Int].new();
  var no = Vec[Int].new();
  var i = 0;
  while i < v.len() { if pred(v[i]) { yes.push(v[i]); } else { no.push(v[i]); } i = i + 1; }
  return (yes, no);
}

pub fn dedup(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    var j = 0;
    var found = 0;
    while j < result.len() && found == 0 { if result[j] == v[i] { found = 1; } j = j + 1; }
    if found == 0 { result.push(v[i]); }
    i = i + 1;
  }
  return result;
}

pub fn unique_count(v: &Vec[Int]) -> Int { return dedup(v).len(); }

fn is_even(n: Int) -> Bool { return n % 2 == 0; }

fn test_partition() -> Int {
  var score = 0;
  var nums = [1, 2, 3, 4, 5, 6];
  var (evens, odds) = partition(&nums, is_even);
  if evens.len() == 3 { score = score + 1; }
  if odds.len() == 3 { score = score + 1; }
  var with_dupes = [1, 2, 2, 3, 3, 3, 4];
  if unique_count(&with_dupes) == 4 { score = score + 1; }
  var deduped = dedup(&with_dupes);
  if deduped.len() == 4 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_vec_ops(); total = total + s1; max_score = max_score + 13;
  var s2 = test_sorting(); total = total + s2; max_score = max_score + 6;
  var s3 = test_searching(); total = total + s3; max_score = max_score + 9;
  var s4 = test_matrix(); total = total + s4; max_score = max_score + 5;
  var s5 = test_partition(); total = total + s5; max_score = max_score + 4;

  return BenchResult{ name: "collections", score: total, max_score: max_score, passed: total == max_score, elapsed_ms: 0 };
}
