// XIOM -- Control Flow Stress Benchmark
// Exercises if/elif/else chains, while loops, match, early returns, and state machines.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.control

use benchmark.main.BenchResult;

pub fn classify_number(n: Int) -> Int {
  if n == 0 { return 0; }
  elif n == 1 { return 1; }
  elif n == 2 { return 2; }
  elif n == 3 { return 3; }
  elif n == 4 { return 4; }
  elif n == 5 { return 5; }
  elif n == 6 { return 6; }
  elif n == 7 { return 7; }
  elif n == 8 { return 8; }
  elif n == 9 { return 9; }
  elif n == 10 { return 10; }
  elif n == 11 { return 11; }
  elif n == 12 { return 12; }
  elif n == 13 { return 13; }
  elif n == 14 { return 14; }
  elif n == 15 { return 15; }
  elif n == 16 { return 16; }
  elif n == 17 { return 17; }
  elif n == 18 { return 18; }
  elif n == 19 { return 19; }
  elif n == 20 { return 20; }
  elif n == 21 { return 21; }
  elif n == 22 { return 22; }
  elif n == 23 { return 23; }
  elif n == 24 { return 24; }
  elif n == 25 { return 25; }
  elif n == 26 { return 26; }
  elif n == 27 { return 27; }
  elif n == 28 { return 28; }
  elif n == 29 { return 29; }
  elif n == 30 { return 30; }
  else { return -1; }
}

pub fn classify_triple(a: Int, b: Int, c: Int) -> Int {
  if a == b && b == c { return 3; }
  elif a == b || b == c || a == c { return 2; }
  elif a != b && b != c && a != c { return 0; }
  else { return 1; }
}

fn test_deep_if() -> Int {
  var score = 0;
  if classify_number(0) == 0 { score = score + 1; }
  if classify_number(15) == 15 { score = score + 1; }
  if classify_number(30) == 30 { score = score + 1; }
  if classify_number(31) == -1 { score = score + 1; }
  if classify_number(-1) == -1 { score = score + 1; }
  if classify_triple(1, 1, 1) == 3 { score = score + 1; }
  if classify_triple(1, 1, 2) == 2 { score = score + 1; }
  if classify_triple(1, 2, 3) == 0 { score = score + 1; }
  return score;
}

pub fn fizzbuzz(n: Int) -> Int {
  if n % 15 == 0 { return 0; }
  elif n % 3 == 0 { return 1; }
  elif n % 5 == 0 { return 2; }
  else { return 3; }
}

pub fn fizzbuzz_multi(n: Int, a: Int, b: Int) -> Int {
  var mod_a = n % a;
  var mod_b = n % b;
  if mod_a == 0 && mod_b == 0 { return 0; }
  elif mod_a == 0 { return 1; }
  elif mod_b == 0 { return 2; }
  else { return 3; }
}

pub fn fizzbuzz_range(lo: Int, hi: Int) -> Int {
  var count_fizzbuzz = 0;
  var n = lo;
  while n <= hi {
    if fizzbuzz(n) == 0 { count_fizzbuzz = count_fizzbuzz + 1; }
    n = n + 1;
  }
  return count_fizzbuzz;
}

fn test_fizzbuzz() -> Int {
  var score = 0;
  if fizzbuzz(15) == 0 { score = score + 1; }
  if fizzbuzz(3) == 1 { score = score + 1; }
  if fizzbuzz(5) == 2 { score = score + 1; }
  if fizzbuzz(7) == 3 { score = score + 1; }
  if fizzbuzz(30) == 0 { score = score + 1; }
  if fizzbuzz_multi(10, 2, 5) == 0 { score = score + 1; }
  if fizzbuzz_multi(4, 2, 5) == 1 { score = score + 1; }
  if fizzbuzz_multi(5, 3, 5) == 2 { score = score + 1; }
  if fizzbuzz_multi(7, 3, 5) == 3 { score = score + 1; }
  if fizzbuzz_range(1, 15) == 1 { score = score + 1; }
  if fizzbuzz_range(1, 30) == 2 { score = score + 1; }
  return score;
}

pub fn match_number(n: Int) -> Int {
  match n {
    0 => 10, 1 => 20, 2 => 30, 3 => 40, 4 => 50,
    5 => 60, 6 => 70, 7 => 80, 8 => 90, 9 => 100,
    _ => 0,
  }
}

pub fn match_sign(n: Int) -> Int {
  match n { x if x > 0 => 1, x if x < 0 => -1, _ => 0 }
}

fn test_match() -> Int {
  var score = 0;
  if match_number(0) == 10 { score = score + 1; }
  if match_number(5) == 60 { score = score + 1; }
  if match_number(9) == 100 { score = score + 1; }
  if match_number(10) == 0 { score = score + 1; }
  if match_number(-1) == 0 { score = score + 1; }
  if match_sign(5) == 1 { score = score + 1; }
  if match_sign(-3) == -1 { score = score + 1; }
  if match_sign(0) == 0 { score = score + 1; }
  return score;
}

pub fn sum_while(limit: Int) -> Int {
  var total = 0; var i = 0;
  while i <= limit { total = total + i; i = i + 1; }
  return total;
}

pub fn countdown(n: Int) -> Int {
  var x = n; var steps = 0;
  while x > 0 { x = x - 1; steps = steps + 1; }
  return steps;
}

pub fn factorial_while(n: Int) -> Int {
  if n <= 0 { return 1; }
  var result = 1; var i = n;
  while i > 0 { result = result * i; i = i - 1; }
  return result;
}

pub fn while_nested(depth: Int, width: Int) -> Int {
  var total = 0; var i = 0;
  while i < depth {
    var j = 0;
    while j < width { total = total + 1; j = j + 1; }
    i = i + 1;
  }
  return total;
}

pub fn while_with_break_sim(n: Int) -> Int {
  var found = -1; var i = 0; var done = 0;
  while done == 0 {
    if i * i > n { done = 1; }
    else { if i * i == n { found = i; done = 1; } }
    i = i + 1;
  }
  return found;
}

fn test_while() -> Int {
  var score = 0;
  if sum_while(0) == 0 { score = score + 1; }
  if sum_while(10) == 55 { score = score + 1; }
  if sum_while(100) == 5050 { score = score + 1; }
  if countdown(5) == 5 { score = score + 1; }
  if countdown(0) == 0 { score = score + 1; }
  if factorial_while(5) == 120 { score = score + 1; }
  if factorial_while(0) == 1 { score = score + 1; }
  if factorial_while(7) == 5040 { score = score + 1; }
  if while_nested(3, 4) == 12 { score = score + 1; }
  if while_nested(1, 1) == 1 { score = score + 1; }
  if while_nested(10, 10) == 100 { score = score + 1; }
  if while_with_break_sim(25) == 5 { score = score + 1; }
  if while_with_break_sim(9) == 3 { score = score + 1; }
  if while_with_break_sim(26) == -1 { score = score + 1; }
  return score;
}

pub fn all_three(a: Bool, b: Bool, c: Bool) -> Bool { return a && b && c; }
pub fn any_three(a: Bool, b: Bool, c: Bool) -> Bool { return a || b || c; }
pub fn xor(a: Bool, b: Bool) -> Bool { return (a || b) && !(a && b); }

pub fn majority(a: Bool, b: Bool, c: Bool) -> Bool {
  var count = 0;
  if a { count = count + 1; }
  if b { count = count + 1; }
  if c { count = count + 1; }
  return count >= 2;
}

pub fn bool_to_int(b: Bool) -> Int { if b { return 1; } return 0; }

fn test_bool() -> Int {
  var score = 0;
  if all_three(true, true, true) { score = score + 1; }
  if !(all_three(true, true, false)) { score = score + 1; }
  if !(all_three(false, false, false)) { score = score + 1; }
  if any_three(true, false, false) { score = score + 1; }
  if !(any_three(false, false, false)) { score = score + 1; }
  if xor(true, false) { score = score + 1; }
  if xor(false, true) { score = score + 1; }
  if !(xor(true, true)) { score = score + 1; }
  if !(xor(false, false)) { score = score + 1; }
  if majority(true, true, true) { score = score + 1; }
  if majority(true, true, false) { score = score + 1; }
  if !(majority(true, false, false)) { score = score + 1; }
  if !(majority(false, false, false)) { score = score + 1; }
  if bool_to_int(true) == 1 { score = score + 1; }
  if bool_to_int(false) == 0 { score = score + 1; }
  return score;
}

pub type TrafficLight = { state: Int; } derive[Clone]

pub fn TrafficLight.new() -> TrafficLight { return TrafficLight{ state: 0 }; }
pub fn TrafficLight.next() -> TrafficLight {
  match state { 0 => TrafficLight{ state: 1 }, 1 => TrafficLight{ state: 2 }, 2 => TrafficLight{ state: 0 }, _ => TrafficLight{ state: 0 } }
}
pub fn TrafficLight.is_green() -> Bool { return state == 0; }
pub fn TrafficLight.is_yellow() -> Bool { return state == 1; }
pub fn TrafficLight.is_red() -> Bool { return state == 2; }

fn test_state_machine() -> Int {
  var score = 0;
  var light = TrafficLight.new();
  if light.is_green() { score = score + 1; }
  var light2 = light.next();
  if light2.is_yellow() { score = score + 1; }
  var light3 = light2.next();
  if light3.is_red() { score = score + 1; }
  var light4 = light3.next();
  if light4.is_green() { score = score + 1; }
  var cycle = TrafficLight.new();
  var c1 = cycle.next(); var c2 = c1.next(); var c3 = c2.next();
  if c3.is_green() { score = score + 1; }
  return score;
}

pub fn triangle_type(a: Int, b: Int, c: Int) -> Int {
  if a <= 0 || b <= 0 || c <= 0 { return 0; }
  if a + b <= c || b + c <= a || a + c <= b { return 0; }
  if a == b && b == c { return 3; }
  if a == b || b == c || a == c { return 2; }
  return 1;
}

pub fn leap_year(year: Int) -> Bool {
  if year % 400 == 0 { return true; }
  if year % 100 == 0 { return false; }
  if year % 4 == 0 { return true; }
  return false;
}

pub fn days_in_month(month: Int, year: Int) -> Int {
  match month {
    1 => 31, 2 => if leap_year(year) { 29; } else { 28; },
    3 => 31, 4 => 30, 5 => 31, 6 => 30, 7 => 31, 8 => 31,
    9 => 30, 10 => 31, 11 => 30, 12 => 31, _ => 0,
  }
}

pub fn day_of_week(day: Int, month: Int, year: Int) -> Int {
  var y = year; var m = month;
  if m < 3 { m = m + 12; y = y - 1; }
  var k = y % 100; var j = y / 100;
  return (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
}

fn test_complex_conditions() -> Int {
  var score = 0;
  if triangle_type(1, 1, 1) == 3 { score = score + 1; }
  if triangle_type(2, 2, 3) == 2 { score = score + 1; }
  if triangle_type(3, 4, 5) == 1 { score = score + 1; }
  if triangle_type(0, 1, 2) == 0 { score = score + 1; }
  if triangle_type(1, 1, 3) == 0 { score = score + 1; }
  if leap_year(2000) { score = score + 1; }
  if leap_year(2020) { score = score + 1; }
  if !(leap_year(1900)) { score = score + 1; }
  if !(leap_year(2023)) { score = score + 1; }
  if leap_year(2024) { score = score + 1; }
  if days_in_month(1, 2023) == 31 { score = score + 1; }
  if days_in_month(2, 2024) == 29 { score = score + 1; }
  if days_in_month(2, 2023) == 28 { score = score + 1; }
  if days_in_month(4, 2023) == 30 { score = score + 1; }
  return score;
}

pub fn collatz_steps(n: Int) -> Int {
  var steps = 0; var x = n;
  while x > 1 {
    if x % 2 == 0 { x = x / 2; } else { x = 3 * x + 1; }
    steps = steps + 1;
  }
  return steps;
}

pub fn collatz_max(n: Int) -> Int {
  var max_val = n; var x = n;
  while x > 1 {
    if x % 2 == 0 { x = x / 2; } else { x = 3 * x + 1; }
    if x > max_val { max_val = x; }
  }
  return max_val;
}

fn test_collatz() -> Int {
  var score = 0;
  if collatz_steps(1) == 0 { score = score + 1; }
  if collatz_steps(6) == 8 { score = score + 1; }
  if collatz_max(1) == 1 { score = score + 1; }
  if collatz_max(6) == 16 { score = score + 1; }
  return score;
}

pub fn run_all() -> BenchResult {
  var total = 0; var max_score = 0;
  var s1 = test_deep_if(); total = total + s1; max_score = max_score + 8;
  var s2 = test_fizzbuzz(); total = total + s2; max_score = max_score + 11;
  var s3 = test_match(); total = total + s3; max_score = max_score + 8;
  var s4 = test_while(); total = total + s4; max_score = max_score + 14;
  var s5 = test_bool(); total = total + s5; max_score = max_score + 15;
  var s6 = test_state_machine(); total = total + s6; max_score = max_score + 5;
  var s7 = test_complex_conditions(); total = total + s7; max_score = max_score + 14;
  var s8 = test_collatz(); total = total + s8; max_score = max_score + 4;
  return BenchResult{ name: "control", score: total, max_score: max_score, passed: total == max_score, elapsed_ms: 0 };
}