// XIOM -- Sorting Algorithm Stress Benchmark
// Exercises multiple sorting algorithms with different complexity characteristics.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.sort

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: O(n^2) Sorts
// ============================================================

pub fn bubble_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(v);
  var n = arr.len();
  var i = 0;
  while i < n - 1 {
    var j = 0;
    while j < n - i - 1 {
      if arr[j] > arr[j + 1] {
        var temp = arr[j];
        arr[j] = arr[j + 1];
        arr[j + 1] = temp;
      }
      j = j + 1;
    }
    i = i + 1;
  }
  return arr;
}

pub fn selection_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(v);
  var n = arr.len();
  var i = 0;
  while i < n - 1 {
    var min_idx = i;
    var j = i + 1;
    while j < n {
      if arr[j] < arr[min_idx] { min_idx = j; }
      j = j + 1;
    }
    if min_idx != i {
      var temp = arr[i];
      arr[i] = arr[min_idx];
      arr[min_idx] = temp;
    }
    i = i + 1;
  }
  return arr;
}

pub fn insertion_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(v);
  var n = arr.len();
  var i = 1;
  while i < n {
    var key = arr[i];
    var j = i;
    while j > 0 && arr[j - 1] > key {
      arr[j] = arr[j - 1];
      j = j - 1;
    }
    arr[j] = key;
    i = i + 1;
  }
  return arr;
}

pub fn gnome_sort(v: Vec[Int]) -> Vec[Int] {
  var arr = vec_copy(v);
  var pos = 0;
  var n = arr.len();
  while pos < n {
    if pos == 0 || arr[pos] >= arr[pos - 1] {
      pos = pos + 1;
    } else {
      var temp = arr[pos];
      arr[pos] = arr[pos - 1];
      arr[pos - 1] = temp;
      pos = pos - 1;
    }
  }
  return arr;
}

pub fn vec_copy(v: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    result.push(v[i]);
    i = i + 1;
  }
  return result;
}

pub fn is_sorted(v: &Vec[Int]) -> Bool {
  var i = 1;
  while i < v.len() {
    if v[i - 1] > v[i] { return false; }
    i = i + 1;
  }
  return true;
}

pub fn sorted_equal(a: &Vec[Int], b: &Vec[Int]) -> Bool {
  if a.len() != b.len() { return false; }
  var i = 0;
  while i < a.len() {
    if a[i] != b[i] { return false; }
    i = i + 1;
  }
  return true;
}

fn test_quadratic_sorts() -> Int {
  var score = 0;
  var unsorted = [5, 2, 8, 1, 9, 3, 7, 4, 6];

  var bs = bubble_sort(unsorted);
  if is_sorted(&bs) { score = score + 1; }
  if bs[0] == 1 && bs[bs.len() - 1] == 9 { score = score + 1; }

  var ss = selection_sort(unsorted);
  if is_sorted(&ss) { score = score + 1; }

  var ins = insertion_sort(unsorted);
  if is_sorted(&ins) { score = score + 1; }

  var gn = gnome_sort(unsorted);
  if is_sorted(&gn) { score = score + 1; }

  // All sorts produce the same result
  if sorted_equal(&bs, &ss) { score = score + 1; }
  if sorted_equal(&bs, &ins) { score = score + 1; }
  if sorted_equal(&bs, &gn) { score = score + 1; }

  // Edge cases
  var already = [1, 2, 3, 4, 5];
  var sorted_already = bubble_sort(already);
  if is_sorted(&sorted_already) { score = score + 1; }

  var reverse = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  var sorted_rev = insertion_sort(reverse);
  if is_sorted(&sorted_rev) { score = score + 1; }
  if sorted_rev[0] == 1 { score = score + 1; }

  var single = [42];
  if is_sorted(&bubble_sort(single)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: O(n log n) Sorts (Merge Sort, Quick Sort)
// ============================================================

pub fn merge_sort(v: Vec[Int]) -> Vec[Int] {
  if v.len() <= 1 { return v; }
  var mid = v.len() / 2;
  var left = slice(v, 0, mid);
  var right = slice(v, mid, v.len());
  var sorted_left = merge_sort(left);
  var sorted_right = merge_sort(right);
  return merge(sorted_left, sorted_right);
}

pub fn slice(v: Vec[Int], start: Int, end: Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = start;
  while i < end {
    result.push(v[i]);
    i = i + 1;
  }
  return result;
}

pub fn merge(left: Vec[Int], right: Vec[Int]) -> Vec[Int] {
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

pub fn quick_sort(v: Vec[Int]) -> Vec[Int] {
  if v.len() <= 1 { return v; }
  var pivot = v[v.len() / 2];
  var less = Vec[Int].new();
  var equal = Vec[Int].new();
  var greater = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    if v[i] < pivot {
      less.push(v[i]);
    } elif v[i] > pivot {
      greater.push(v[i]);
    } else {
      equal.push(v[i]);
    }
    i = i + 1;
  }
  var sorted_less = quick_sort(less);
  var sorted_greater = quick_sort(greater);
  return concat_three(sorted_less, equal, sorted_greater);
}

pub fn concat_three(a: Vec[Int], b: Vec[Int], c: Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < a.len() { result.push(a[i]); i = i + 1; }
  i = 0;
  while i < b.len() { result.push(b[i]); i = i + 1; }
  i = 0;
  while i < c.len() { result.push(c[i]); i = i + 1; }
  return result;
}

fn test_nlogn_sorts() -> Int {
  var score = 0;
  var unsorted = [9, 3, 7, 1, 5, 8, 2, 6, 4];

  var ms = merge_sort(unsorted);
  if is_sorted(&ms) { score = score + 1; }

  var qs = quick_sort(unsorted);
  if is_sorted(&qs) { score = score + 1; }

  if sorted_equal(&ms, &qs) { score = score + 1; }

  // Edge cases
  if is_sorted(&merge_sort([1])) { score = score + 1; }
  if is_sorted(&quick_sort([1])) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Counting/Bucket Sort
// ============================================================

pub fn counting_sort(v: Vec[Int], max_val: Int) -> Vec[Int] {
  var counts = Vec[Int].new();
  var i = 0;
  while i <= max_val {
    counts.push(0);
    i = i + 1;
  }
  i = 0;
  while i < v.len() {
    var idx = v[i];
    counts[idx] = counts[idx] + 1;
    i = i + 1;
  }
  var result = Vec[Int].new();
  i = 0;
  while i < counts.len() {
    var j = 0;
    while j < counts[i] {
      result.push(i);
      j = j + 1;
    }
    i = i + 1;
  }
  return result;
}

fn test_counting_sort() -> Int {
  var score = 0;
  var data = [3, 1, 4, 1, 5, 9, 2, 6];
  var sorted = counting_sort(data, 9);
  if is_sorted(&sorted) { score = score + 1; }
  if sorted.len() == data.len() { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 4: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_quadratic_sorts();
  total = total + s1;
  max_score = max_score + 12;

  var s2 = test_nlogn_sorts();
  total = total + s2;
  max_score = max_score + 5;

  var s3 = test_counting_sort();
  total = total + s3;
  max_score = max_score + 2;

  return BenchResult{
    name: "sort",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
