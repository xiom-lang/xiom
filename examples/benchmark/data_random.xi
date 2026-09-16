// XIOM -- Random Data Module
// Large static array of pseudo-random numbers for heavy processing benchmarks.
// Generates 1000 values via a known seed for deterministic verification.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.data_random

// ============================================================
// LCG-based PRNG (deterministic)
// ============================================================
pub fn lcg(seed: Int, count: Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var state = seed;
  var i = 0;
  while i < count {
    state = (state * 1103515245 + 12345) % 2147483648;
    result.push(state % 1000000);
    i = i + 1;
  }
  return result;
}

// ============================================================
// Generate dataset 1: 500 random values (seed=42)
// ============================================================
pub fn dataset_500() -> Vec[Int] {
  return lcg(42, 500);
}

// ============================================================
// Generate dataset 2: 500 more random values (seed=137)
// ============================================================
pub fn dataset_500b() -> Vec[Int] {
  return lcg(137, 500);
}

// ============================================================
// Concat both datasets into a 1000-element array
// ============================================================
pub fn dataset_1000() -> Vec[Int] {
  var d1 = dataset_500();
  var d2 = dataset_500b();
  var result = Vec[Int].new();
  var i = 0;
  while i < d1.len() {
    result.push(d1[i]);
    i = i + 1;
  }
  i = 0;
  while i < d2.len() {
    result.push(d2[i]);
    i = i + 1;
  }
  return result;
}

// ============================================================
// Basic statistics on the dataset
// ============================================================
pub fn dataset_min(data: &Vec[Int]) -> Int {
  if data.len() == 0 { return 0; }
  var m = data[0];
  var i = 1;
  while i < data.len() {
    if data[i] < m { m = data[i]; }
    i = i + 1;
  }
  return m;
}

pub fn dataset_max(data: &Vec[Int]) -> Int {
  if data.len() == 0 { return 0; }
  var m = data[0];
  var i = 1;
  while i < data.len() {
    if data[i] > m { m = data[i]; }
    i = i + 1;
  }
  return m;
}

pub fn dataset_sum(data: &Vec[Int]) -> Int {
  var sum = 0;
  var i = 0;
  while i < data.len() {
    sum = sum + data[i];
    i = i + 1;
  }
  return sum;
}

pub fn dataset_mean(data: &Vec[Int]) -> Int {
  if data.len() == 0 { return 0; }
  return dataset_sum(data) / data.len();
}

pub fn dataset_count_in_range(data: &Vec[Int], lo: Int, hi: Int) -> Int {
  var count = 0;
  var i = 0;
  while i < data.len() {
    if data[i] >= lo && data[i] <= hi { count = count + 1; }
    i = i + 1;
  }
  return count;
}

pub fn dataset_has_duplicates(data: &Vec[Int]) -> Bool {
  var i = 0;
  while i < data.len() {
    var j = i + 1;
    while j < data.len() {
      if data[i] == data[j] { return true; }
      j = j + 1;
    }
    i = i + 1;
  }
  return false;
}

// ============================================================
// Shuffle-like permutation generator
// ============================================================
pub fn identity_permutation(n: Int) -> Vec[Int] {
  var p = Vec[Int].new();
  var i = 0;
  while i < n {
    p.push(i);
    i = i + 1;
  }
  return p;
}

pub fn reverse_permutation(n: Int) -> Vec[Int] {
  var p = Vec[Int].new();
  var i = n;
  while i > 0 {
    i = i - 1;
    p.push(i);
  }
  return p;
}

// ============================================================
// Fixed sine wave data (integer approximation)
// ============================================================
pub fn sine_wave(amplitude: Int, samples: Int) -> Vec[Int] {
  var wave = Vec[Int].new();
  var i = 0;
  while i < samples {
    // Approximate sin(2*pi*i/samples) using integer arithmetic
    var phase = (i * 360) / samples;
    var val = 0;
    if phase < 90 {
      val = (phase * amplitude) / 90;
    } elif phase < 180 {
      val = ((180 - phase) * amplitude) / 90;
    } elif phase < 270 {
      val = -((phase - 180) * amplitude) / 90;
    } else {
      val = -((360 - phase) * amplitude) / 90;
    }
    wave.push(val);
    i = i + 1;
  }
  return wave;
}

// ============================================================
// Cumulative sum array
// ============================================================
pub fn prefix_sum(data: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var running = 0;
  var i = 0;
  while i < data.len() {
    running = running + data[i];
    result.push(running);
    i = i + 1;
  }
  return result;
}

// ============================================================
// Histogram bins helper
// ============================================================
pub fn histogram(data: &Vec[Int], bins: Int, bin_width: Int) -> Vec[Int] {
  var counts = Vec[Int].new();
  var i = 0;
  while i < bins {
    counts.push(0);
    i = i + 1;
  }
  i = 0;
  while i < data.len() {
    var bin = data[i] / bin_width;
    if bin >= 0 && bin < bins {
      counts[bin] = counts[bin] + 1;
    }
    i = i + 1;
  }
  return counts;
}
