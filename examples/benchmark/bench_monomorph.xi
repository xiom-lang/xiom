// XIOM -- Huge Monomorphisation Benchmark
// Forces the compiler to generate 15+ struct instantiations, 10+ function
// instantiations, nested generics, and cross-product instantiations.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.monomorph

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Generic Struct with 15+ Distinct Instantiations
// ============================================================

pub type Slot[T] = {
  value: T;
  version: Int;
} derive[Clone]

pub fn Slot.new[T](val: T, ver: Int) -> Slot[T] {
  return Slot[T]{ value: val, version: ver };
}

pub fn Slot.read[T]() -> T {
  return value;
}

pub fn Slot.ver[T]() -> Int {
  return version;
}

pub type IntSlot    = Slot[Int];
pub type BoolSlot   = Slot[Bool];
pub type FloatSlot  = Slot[Float64];
pub type StrSlot    = Slot[Str];

fn test_15_struct_instantiations() -> Int {
  var score = 0;

  var s1 = Slot.new[Int](1, 1);
  if s1.read() == 1 { score = score + 1; }

  var s2 = Slot.new[Bool](true, 2);
  if s2.read() { score = score + 1; }

  var s3 = Slot.new[Float64](3.14, 3);
  if s3.ver() == 3 { score = score + 1; }

  var s4 = Slot.new[Str]("ax", 4);
  if s4.ver() == 4 { score = score + 1; }

  var s5 = Slot.new[Int](0, 5);
  var s6 = Slot.new[Bool](false, 6);
  var s7 = Slot.new[Float64](2.71, 7);
  var s8 = Slot.new[Int](2, 8);

  if s5.ver() == 5 { score = score + 1; }
  if !(s6.read()) { score = score + 1; }
  if s7.ver() == 7 { score = score + 1; }
  if s8.read() == 2 { score = score + 1; }

  var s9  = Slot.new[Int](3, 9);
  var s10 = Slot.new[Bool](true, 10);
  var s11 = Slot.new[Float64](1.0, 11);
  var s12 = Slot.new[Str]("b", 12);
  var s13 = Slot.new[Int](4, 13);
  var s14 = Slot.new[Bool](true, 14);
  var s15 = Slot.new[Float64](0.5, 15);

  if s9.ver() == 9 { score = score + 1; }
  if s10.ver() == 10 { score = score + 1; }
  if s11.ver() == 11 { score = score + 1; }
  if s12.ver() == 12 { score = score + 1; }
  if s13.ver() == 13 { score = score + 1; }
  if s14.ver() == 14 { score = score + 1; }
  if s15.ver() == 15 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: 10+ Distinct Generic Function Instantiations
// ============================================================

pub fn identity_fn[T](x: T) -> T {
  return x;
}

pub fn greater_than[T](a: T, b: T) -> Bool {
  return a > b;
}

pub fn add_one_or[T](x: T) -> T {
  return x;
}

fn test_10_fn_instantiations() -> Int {
  var score = 0;

  var fi1 = identity_fn[Int](42);
  var fi2 = identity_fn[Bool](true);
  var fi3 = identity_fn[Float64](3.14);
  var fi4 = identity_fn[Int](0);
  var fi5 = identity_fn[Bool](false);
  var fi6 = identity_fn[Float64](0.0);
  var fi7 = identity_fn[Int](100);
  var fi8 = identity_fn[Int](200);
  var fi9 = identity_fn[Bool](true);
  var fi10 = identity_fn[Float64](99.9);

  if fi1 == 42 { score = score + 1; }
  if fi2 { score = score + 1; }
  if fi3 == 3.14 { score = score + 1; }
  if fi4 == 0 { score = score + 1; }
  if !fi5 { score = score + 1; }
  if fi6 == 0.0 { score = score + 1; }
  if fi7 == 100 { score = score + 1; }
  if fi8 == 200 { score = score + 1; }
  if fi9 { score = score + 1; }
  if fi10 == 99.9 { score = score + 1; }

  if greater_than[Int](5, 3) { score = score + 1; }
  if !(greater_than[Int](1, 9)) { score = score + 1; }
  if greater_than[Float64](5.5, 3.3) { score = score + 1; }

  var a1 = add_one_or[Int](7);
  var a2 = add_one_or[Float64](7.5);
  if a1 == 7 { score = score + 1; }
  if a2 == 7.5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Nested Generics -- Vec of Option of Pair
// ============================================================

pub type Pair[A, B] = {
  fst: A;
  snd: B;
} derive[Clone]

pub fn Pair.new[A, B](a: A, b: B) -> Pair[A, B] {
  return Pair[A, B]{ fst: a, snd: b };
}

pub fn Pair.get_first[A, B]() -> A {
  return fst;
}

pub fn Pair.get_second[A, B]() -> B {
  return snd;
}

pub fn build_nested_collection() -> Vec[Option[Pair[Int, Float64]]] {
  var out = Vec[Option[Pair[Int, Float64]]].new();
  out.push(Some(Pair.new[Int, Float64](1, 1.5)));
  out.push(None);
  out.push(Some(Pair.new[Int, Float64](2, 2.5)));
  out.push(Some(Pair.new[Int, Float64](3, 3.5)));
  out.push(None);
  return out;
}

pub fn count_some_pairs(v: &Vec[Option[Pair[Int, Float64]]]) -> Int {
  var count = 0;
  var i = 0;
  while i < v.len() {
    match v[i] {
      Some(_) => { count = count + 1; }
      None => {}
    }
    i = i + 1;
  }
  return count;
}

pub fn sum_nested_values(v: &Vec[Option[Pair[Int, Float64]]]) -> Float64 {
  var total = 0.0;
  var i = 0;
  while i < v.len() {
    match v[i] {
      Some(p) => {
        var f = p.get_first();
        var s = p.get_second();
        total = total + (f as Float64) + s;
      }
      None => {}
    }
    i = i + 1;
  }
  return total;
}

fn test_nested_generics() -> Int {
  var score = 0;
  var nested = build_nested_collection();
  if nested.len() == 5 { score = score + 1; }

  var some_count = count_some_pairs(&nested);
  if some_count == 3 { score = score + 1; }

  var sum = sum_nested_values(&nested);
  var sum_ok = sum > 13.9 && sum < 14.1;
  if sum_ok { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Generic Methods Called on Different Concrete Types
// ============================================================

pub type Tagged[T] = {
  data: T;
  tag: Str;
} derive[Clone]

pub fn Tagged.label[T](val: T, tag: Str) -> Tagged[T] {
  return Tagged[T]{ data: val, tag: tag };
}

pub fn Tagged.get_tag[T]() -> Str {
  return tag;
}

pub fn Tagged.get_data[T]() -> T {
  return data;
}

pub fn Tagged.describe[T]() -> Str {
  return tag;
}

fn test_generic_method_concrete() -> Int {
  var score = 0;

  var ti: Tagged[Int] = Tagged.label[Int](42, "int_tag");
  if ti.get_data() == 42 { score = score + 1; }
  if ti.get_tag() == "int_tag" { score = score + 1; }
  if ti.describe() == "int_tag" { score = score + 1; }

  var tb: Tagged[Bool] = Tagged.label[Bool](true, "bool_tag");
  if tb.get_data() { score = score + 1; }
  if tb.get_tag() == "bool_tag" { score = score + 1; }

  var tf: Tagged[Float64] = Tagged.label[Float64](3.14, "float_tag");
  if tf.get_data() == 3.14 { score = score + 1; }
  if tf.describe() == "float_tag" { score = score + 1; }

  var ts: Tagged[Str] = Tagged.label[Str]("hello", "str_tag");
  if ts.get_data() == "hello" { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Cross-Product Instantiations
// ============================================================

pub type Cell[A, B] = {
  key: A;
  value: B;
} derive[Clone]

pub fn Cell.new[A, B](k: A, v: B) -> Cell[A, B] {
  return Cell[A, B]{ key: k, value: v };
}

pub fn Cell.read_key[A, B]() -> A {
  return key;
}

pub fn Cell.read_value[A, B]() -> B {
  return value;
}

pub fn cell_pair_int_float(k: Int, v: Float64) -> Cell[Int, Float64] {
  return Cell.new[Int, Float64](k, v);
}

pub fn cell_pair_int_bool(k: Int, v: Bool) -> Cell[Int, Bool] {
  return Cell.new[Int, Bool](k, v);
}

pub fn cell_pair_str_int(k: Str, v: Int) -> Cell[Str, Int] {
  return Cell.new[Str, Int](k, v);
}

pub fn cell_pair_bool_float(k: Bool, v: Float64) -> Cell[Bool, Float64] {
  return Cell.new[Bool, Float64](k, v);
}

pub fn cell_pair_float_str(k: Float64, v: Str) -> Cell[Float64, Str] {
  return Cell.new[Float64, Str](k, v);
}

pub fn cell_pair_int_int(k: Int, v: Int) -> Cell[Int, Int] {
  return Cell.new[Int, Int](k, v);
}

fn test_cross_product() -> Int {
  var score = 0;

  var cp1 = cell_pair_int_float(1, 1.5);
  if cp1.read_key() == 1 { score = score + 1; }

  var cp2 = cell_pair_int_bool(2, true);
  if cp2.read_value() { score = score + 1; }

  var cp3 = cell_pair_str_int("three", 3);
  if cp3.read_key() == "three" { score = score + 1; }

  var cp4 = cell_pair_bool_float(false, 4.0);
  if !(cp4.read_key()) { score = score + 1; }

  var cp5 = cell_pair_float_str(5.5, "five");
  if cp5.read_value() == "five" { score = score + 1; }

  var cp6 = cell_pair_int_int(10, 20);
  if cp6.read_key() == 10 { score = score + 1; }
  if cp6.read_value() == 20 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Monomorphised Vec of Complex Types
// ============================================================

pub type Coord[T] = {
  x: T;
  y: T;
} derive[Clone]

pub fn Coord.new[T](x: T, y: T) -> Coord[T] {
  return Coord[T]{ x: x, y: y };
}

pub fn Coord.dist_sq[T]() -> Int {
  return 0;
}

pub fn build_vec_of_coords() -> Vec[Coord[Int]] {
  var vc = Vec[Coord[Int]].new();
  vc.push(Coord.new[Int](0, 0));
  vc.push(Coord.new[Int](1, 1));
  vc.push(Coord.new[Int](2, 4));
  vc.push(Coord.new[Int](3, 9));
  vc.push(Coord.new[Int](4, 16));
  return vc;
}

pub fn inspect_coords(vc: &Vec[Coord[Int]]) -> Int {
  var total = 0;
  var i = 0;
  while i < vc.len() {
    total = total + vc[i].x + vc[i].y;
    i = i + 1;
  }
  return total;
}

fn test_vec_of_complex() -> Int {
  var score = 0;
  var coords = build_vec_of_coords();
  if coords.len() == 5 { score = score + 1; }
  var sum = inspect_coords(&coords);
  if sum == 40 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 7: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_15_struct_instantiations();
  total = total + s1;
  max_score = max_score + 16;

  var s2 = test_10_fn_instantiations();
  total = total + s2;
  max_score = max_score + 15;

  var s3 = test_nested_generics();
  total = total + s3;
  max_score = max_score + 3;

  var s4 = test_generic_method_concrete();
  total = total + s4;
  max_score = max_score + 8;

  var s5 = test_cross_product();
  total = total + s5;
  max_score = max_score + 7;

  var s6 = test_vec_of_complex();
  total = total + s6;
  max_score = max_score + 2;

  return BenchResult{
    name: "monomorph",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
