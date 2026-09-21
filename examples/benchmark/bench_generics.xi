// XIOM -- Generics Stress Benchmark
// Exercises generic functions, generic types, type constraints, and multiple type parameters.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.generics

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Basic Generic Functions
// ============================================================

pub fn identity[T](x: T) -> T {
  return x;
}

pub fn pair[T, U](a: T, b: U) -> (T, U) {
  return (a, b);
}

pub fn swap[T, U](a: T, b: U) -> (U, T) {
  return (b, a);
}

pub fn triple[T](a: T, b: T, c: T) -> (T, T, T) {
  return (a, b, c);
}

fn test_basic_generics() -> Int {
  var score = 0;
  if identity(42) == 42 { score = score + 1; }
  if identity(true) { score = score + 1; }

  var p1 = pair(1, "hello");
  var p2 = pair(true, 3.14);

  var (i_val, s_val) = pair(42, "test");
  if i_val == 42 { score = score + 1; }

  if p1.0 == 1 { score = score + 1; }

  var (b_val, f_val) = swap(3.14, true);
  if b_val { score = score + 1; }

  var t = triple(10, 20, 30);
  if t.0 == 10 { score = score + 1; }
  if t.2 == 30 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Generic Structs
// ============================================================

pub type Box[T] = {
  value: T;
} derive[Clone]

pub type Pair[A, B] = {
  first: A;
  second: B;
} derive[Clone]

pub type Wrapper[T] = {
  data: T;
  tag: Int;
} derive[Clone]

pub fn Box.new[T](val: T) -> Box[T] {
  return Box[T]{ value: val };
}

pub fn Box.unwrap[T]() -> T {
  return value;
}

pub fn Pair.new[A, B](a: A, b: B) -> Pair[A, B] {
  return Pair[A, B]{ first: a, second: b };
}

pub fn Wrapper.new[T](data: T, tag: Int) -> Wrapper[T] {
  return Wrapper[T]{ data: data, tag: tag };
}

fn test_generic_structs() -> Int {
  var score = 0;
  var b1 = Box.new[Int](42);
  if b1.value == 42 { score = score + 1; }
  if b1.unwrap() == 42 { score = score + 1; }

  var b2 = Box.new[Bool](true);
  if b2.value { score = score + 1; }

  var p = Pair.new[Int, Str](1, "one");
  if p.first == 1 { score = score + 1; }

  var w = Wrapper.new[Float64](3.14, 1);
  if w.data == 3.14 { score = score + 1; }
  if w.tag == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Generic Functions with Operations
// ============================================================

pub fn max[T](a: T, b: T) -> T {
  if a > b { return a; }
  return b;
}

pub fn min[T](a: T, b: T) -> T {
  if a < b { return a; }
  return b;
}

pub fn abs_diff(a: Int, b: Int) -> Int {
  if a > b { return a - b; }
  return b - a;
}

pub fn is_equal[T: Eq](a: T, b: T) -> Bool {
  return a == b;
}

pub fn is_not_equal[T: Eq](a: T, b: T) -> Bool {
  return a != b;
}

fn test_generic_ops() -> Int {
  var score = 0;
  if max(10, 20) == 20 { score = score + 1; }
  if max(-5, 5) == 5 { score = score + 1; }
  if max(3.14, 2.71) == 3.14 { score = score + 1; }

  if min(10, 20) == 10 { score = score + 1; }
  if min[Float64](5.5, 3.3) == 3.3 { score = score + 1; }

  if abs_diff(10, 3) == 7 { score = score + 1; }
  if abs_diff(3, 10) == 7 { score = score + 1; }
  if abs_diff(5, 5) == 0 { score = score + 1; }

  if is_equal(42, 42) { score = score + 1; }
  if !(is_equal(42, 43)) { score = score + 1; }

  if is_not_equal(1, 2) { score = score + 1; }
  if !(is_not_equal(5, 5)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Option/Result Generic Patterns
// ============================================================

pub fn wrap_some[T](x: T) -> Option[T] {
  return Some(x);
}

pub fn wrap_ok[T, E](x: T) -> Result[T, E] {
  return Ok(x);
}

pub fn wrap_err[T, E](e: E) -> Result[T, E] {
  return Err(e);
}

pub fn default_value[T]() -> Option[T] {
  return None;
}

fn test_opt_result_generic() -> Int {
  var score = 0;
  var opt = wrap_some(99);
  match opt {
    Some(v) => if v == 99 { score = score + 1; }
    None => {}
  }

  var ok_res: Result[Int, Str] = wrap_ok[Int, Str](42);
  match ok_res {
    Ok(v) => if v == 42 { score = score + 1; }
    Err(_) => {}
  }

  var err_res: Result[Int, Str] = wrap_err[Int, Str]("fail");
  match err_res {
    Ok(_) => {}
    Err(e) => if e == "fail" { score = score + 1; }
  }

  return score;
}

// ============================================================
// SECTION 5: Generic Data Structures
// ============================================================

pub type Stack[T] = {
  data: Vec[T];
  top: Int;
} derive[Clone]

pub fn Stack.new[T]() -> Stack[T] {
  return Stack[T]{ data: Vec[T].new(), top: 0 };
}

pub fn Stack.push[T](val: T) {
  data.push(val);
  top = data.len();
}

pub fn Stack.is_empty[T]() -> Bool {
  return top == 0;
}

pub fn Stack.size[T]() -> Int {
  return top;
}

pub type Priority = {
  value: Int;
  order: Int;
} derive[Clone]

pub type PriorityQueue[T] = {
  items: Vec[T];
  priorities: Vec[Int];
} derive[Clone]

pub fn PriorityQueue.new[T]() -> PriorityQueue[T] {
  return PriorityQueue[T]{ items: Vec[T].new(), priorities: Vec[Int].new() };
}

pub fn PriorityQueue.enqueue[T](item: T, prio: Int) {
  items.push(item);
  priorities.push(prio);
}

pub fn PriorityQueue.size[T]() -> Int {
  return items.len();
}

fn test_generic_data_structures() -> Int {
  var score = 0;
  var si: Stack[Int] = Stack.new[Int]();
  if si.is_empty() { score = score + 1; }

  si.push(42);
  if si.size() == 1 { score = score + 1; }
  if !(si.is_empty()) { score = score + 1; }

  var sb: Stack[Bool] = Stack.new[Bool]();
  sb.push(true);
  sb.push(false);
  if sb.size() == 2 { score = score + 1; }

  var pq: PriorityQueue[Int] = PriorityQueue.new[Int]();
  pq.enqueue(1, 5);
  pq.enqueue(2, 3);
  pq.enqueue(3, 7);
  if pq.size() == 3 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Generic Methods
// ============================================================

pub type Counter[T] = {
  value: T;
  count: Int;
} derive[Clone]

pub fn Counter.new[T](initial: T) -> Counter[T] {
  return Counter[T]{ value: initial, count: 0 };
}

pub fn Counter.set[T](new_val: T) {
  value = new_val;
  count = count + 1;
}

pub fn Counter.times_updated[T]() -> Int {
  return count;
}

fn test_generic_methods() -> Int {
  var score = 0;
  var ci: Counter[Int] = Counter.new[Int](0);
  if ci.value == 0 { score = score + 1; }
  if ci.times_updated() == 0 { score = score + 1; }

  ci.set(42);
  if ci.value == 42 { score = score + 1; }
  if ci.times_updated() == 1 { score = score + 1; }

  ci.set(99);
  if ci.times_updated() == 2 { score = score + 1; }

  var cb: Counter[Bool] = Counter.new[Bool](false);
  cb.set(true);
  if cb.value { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Nested Generics
// ============================================================

pub type Nested[T] = {
  a: Option[T];
  b: Option[T];
} derive[Clone]

pub type MultiNested[A, B, C] = {
  first: A;
  second: Vec[B];
  third: Option[C];
} derive[Clone]

pub fn Nested.new[T]() -> Nested[T] {
  return Nested[T]{ a: None, b: None };
}

pub fn Nested.set_a[T](val: T) {
  a = Some(val);
}

pub fn MultiNested.new[A, B, C](f: A) -> MultiNested[A, B, C] {
  return MultiNested[A, B, C]{ first: f, second: Vec[B].new(), third: None };
}

fn test_nested_generics() -> Int {
  var score = 0;
  var n: Nested[Int] = Nested.new[Int]();
  match n.a {
    Some(_) => {}
    None => { score = score + 1; }
  }

  n.set_a(42);
  match n.a {
    Some(v) => if v == 42 { score = score + 1; }
    None => {}
  }

  var mn: MultiNested[Int, Str, Bool] = MultiNested.new[Int, Str, Bool](1);
  if mn.first == 1 { score = score + 1; }

  match mn.third {
    Some(_) => {}
    None => { score = score + 1; }
  }

  return score;
}

// ============================================================
// SECTION 8: Type Parameter Stress
// ============================================================

pub fn compose[A, B, C](f: fn(A) -> B, g: fn(B) -> C, x: A) -> C {
  return g(f(x));
}

fn add_one(x: Int) -> Int { return x + 1; }
fn double_it(x: Int) -> Int { return x * 2; }
fn to_bool(x: Int) -> Bool { return x > 0; }

fn test_type_params() -> Int {
  var score = 0;
  var r1 = compose(add_one, double_it, 5);
  if r1 == 12 { score = score + 1; }

  var r2 = compose(double_it, add_one, 3);
  if r2 == 7 { score = score + 1; }

  var r3 = compose(add_one, to_bool, 0);
  if r3 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 9: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_basic_generics();
  total = total + s1;
  max_score = max_score + 7;

  var s2 = test_generic_structs();
  total = total + s2;
  max_score = max_score + 6;

  var s3 = test_generic_ops();
  total = total + s3;
  max_score = max_score + 12;

  var s4 = test_opt_result_generic();
  total = total + s4;
  max_score = max_score + 3;

  var s5 = test_generic_data_structures();
  total = total + s5;
  max_score = max_score + 6;

  var s6 = test_generic_methods();
  total = total + s6;
  max_score = max_score + 7;

  var s7 = test_nested_generics();
  total = total + s7;
  max_score = max_score + 4;

  var s8 = test_type_params();
  total = total + s8;
  max_score = max_score + 3;

  return BenchResult{
    name: "generics",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
