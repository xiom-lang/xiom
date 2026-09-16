// XIOM -- Heavy Generics + Comptime Benchmark
// Exercises complex generic types, multi-param generics, generic enums,
// nested generics, and generic methods on generic structs.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.generics_hard

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Generic Box[T] with Methods
// ============================================================

pub type Box[T] = {
  item: T;
  sealed: Bool;
} derive[Clone]

pub fn Box.pack[T](val: T) -> Box[T] {
  return Box[T]{ item: val, sealed: false };
}

pub fn Box.unpack[T]() -> T {
  return item;
}

pub fn Box.seal[T]() {
  sealed = true;
}

pub fn Box.is_sealed[T]() -> Bool {
  return sealed;
}

pub fn Box.replace[T](val: T) {
  item = val;
}

fn test_generic_box() -> Int {
  var score = 0;
  var bi: Box[Int] = Box.pack[Int](42);
  if bi.unpack() == 42 { score = score + 1; }
  if !(bi.is_sealed()) { score = score + 1; }

  bi.seal();
  if bi.is_sealed() { score = score + 1; }

  bi.replace(99);
  if bi.unpack() == 99 { score = score + 1; }

  var bs: Box[Str] = Box.pack[Str]("xiom");
  if bs.unpack() == "xiom" { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Generic Function with 2 Type Params + Constraint
// ============================================================

pub fn choose_first[T, U](a: T, b: U) -> T {
  var _discard = b;
  return a;
}

pub fn choose_second[T, U](a: T, b: U) -> U {
  var _discard2 = a;
  return b;
}

pub fn combine_eq[T: Eq, U: Eq](a1: T, a2: T, b: U) -> Bool {
  if a1 == a2 { return true; }
  var _use_b = b;
  return false;
}

pub fn matches_any[T: Eq](needle: T, a: T, b: T, c: T) -> Bool {
  if needle == a { return true; }
  if needle == b { return true; }
  if needle == c { return true; }
  return false;
}

fn test_multi_param_constrained() -> Int {
  var score = 0;
  var cf1 = choose_first[Int, Bool](10, true);
  if cf1 == 10 { score = score + 1; }
  var cf2 = choose_second[Int, Bool](10, false);
  if !cf2 { score = score + 1; }

  if combine_eq[Int, Str](1, 1, "ok") { score = score + 1; }
  if !(combine_eq[Int, Str](1, 2, "ok")) { score = score + 1; }

  if matches_any[Int](5, 1, 5, 9) { score = score + 1; }
  if !(matches_any[Int](5, 1, 2, 3)) { score = score + 1; }
  if matches_any[Int](0, 0, 0, 0) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Nested Generic -- Vec[Pair[Int, Float64]]
// ============================================================

pub type Pair[A, B] = {
  first: A;
  second: B;
} derive[Clone]

pub fn Pair.new[A, B](a: A, b: B) -> Pair[A, B] {
  return Pair[A, B]{ first: a, second: b };
}

pub fn Pair.read_first[A, B]() -> A {
  return first;
}

pub fn Pair.read_second[A, B]() -> B {
  return second;
}

pub fn collect_pairs() -> Vec[Pair[Int, Float64]] {
  var result = Vec[Pair[Int, Float64]].new();
  result.push(Pair.new[Int, Float64](1, 1.1));
  result.push(Pair.new[Int, Float64](2, 2.2));
  result.push(Pair.new[Int, Float64](3, 3.3));
  result.push(Pair.new[Int, Float64](4, 4.4));
  result.push(Pair.new[Int, Float64](5, 5.5));
  return result;
}

pub fn sum_pairs(v: &Vec[Pair[Int, Float64]]) -> Float64 {
  var total = 0.0;
  var i = 0;
  while i < v.len() {
    var fst = v[i].first;
    var snd = v[i].second;
    total = total + (fst as Float64) + snd;
    i = i + 1;
  }
  return total;
}

fn test_nested_generic_collection() -> Int {
  var score = 0;
  var pairs = collect_pairs();
  if pairs.len() == 5 { score = score + 1; }

  var p_first = pairs[0].read_first();
  if p_first == 1 { score = score + 1; }

  var p_second = pairs[1].read_second();
  var p_snd_ok = p_second > 2.1 && p_second < 2.3;
  if p_snd_ok { score = score + 1; }

  var sp = sum_pairs(&pairs);
  var sp_ok = sp > 31.4 && sp < 31.6;
  if sp_ok { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Generic Enum with Data Variants
// ============================================================

pub enum Container[T] {
  Empty,
  Single(value: T),
  Duo(first: T, second: T),
  Many(items: Vec[T]),
}

pub fn Container.from_single[T](val: T) -> Container[T] {
  return Single(value: val);
}

pub fn Container.from_duo[T](a: T, b: T) -> Container[T] {
  return Duo(first: a, second: b);
}

pub fn Container.from_many[T](vals: Vec[T]) -> Container[T] {
  return Many(items: vals);
}

pub fn Container.count[T]() -> Int {
  match self {
    Empty => 0,
    Single(value: _) => 1,
    Duo(first: _, second: _) => 2,
    Many(items: _) => 0,
  }
}

pub fn Container.get_first[T]() -> Option[T] {
  match self {
    Empty => None,
    Single(value: v) => Some(v),
    Duo(first: f, second: _) => Some(f),
    Many(items: xs) => {
      if xs.len() > 0 { return Some(xs[0]); }
      return None;
    }
  }
}

fn test_generic_enum() -> Int {
  var score = 0;
  var c_empty: Container[Int] = Empty;
  if c_empty.count() == 0 { score = score + 1; }
  match c_empty.get_first() {
    Some(_) => {}
    None => { score = score + 1; }
  }

  var c_one: Container[Int] = Container.from_single[Int](42);
  if c_one.count() == 1 { score = score + 1; }
  match c_one.get_first() {
    Some(v) => if v == 42 { score = score + 1; }
    None => {}
  }

  var c_duo: Container[Int] = Container.from_duo[Int](10, 20);
  if c_duo.count() == 2 { score = score + 1; }
  match c_duo.get_first() {
    Some(v) => if v == 10 { score = score + 1; }
    None => {}
  }

  var bag = Vec[Int].new();
  bag.push(100);
  bag.push(200);
  var c_many: Container[Int] = Container.from_many[Int](bag);
  match c_many.get_first() {
    Some(v) => if v == 100 { score = score + 1; }
    None => {}
  }

  return score;
}

// ============================================================
// SECTION 5: Generic Method on Generic Struct
// ============================================================

pub type GraphNode[T] = {
  val: T;
  edges: Vec[Int];
} derive[Clone]

pub fn GraphNode.new[T](data: T) -> GraphNode[T] {
  return GraphNode[T]{ val: data, edges: Vec[Int].new() };
}

pub fn GraphNode.connect[T](target: Int) {
  edges.push(target);
}

pub fn GraphNode.degree[T]() -> Int {
  return edges.len();
}

pub fn GraphNode.has_edge[T](target: Int) -> Bool {
  var i = 0;
  while i < edges.len() {
    if edges[i] == target { return true; }
    i = i + 1;
  }
  return false;
}

pub fn GraphNode.map[U](f: fn(T) -> U) -> GraphNode[U] {
  return GraphNode[U]{ val: f(val), edges: Vec[Int].new() };
}

pub fn double_int(x: Int) -> Int { return x * 2; }
pub fn bool_from_int(x: Int) -> Bool { return x > 0; }

fn test_generic_method() -> Int {
  var score = 0;
  var gn: GraphNode[Int] = GraphNode.new[Int](5);
  if gn.val == 5 { score = score + 1; }
  if gn.degree() == 0 { score = score + 1; }

  gn.connect(1);
  gn.connect(2);
  gn.connect(3);
  if gn.degree() == 3 { score = score + 1; }
  if gn.has_edge(2) { score = score + 1; }
  if !(gn.has_edge(9)) { score = score + 1; }

  var gn2 = gn.map[Int](double_int);
  if gn2.val == 10 { score = score + 1; }

  var gn3 = gn.map[Bool](bool_from_int);
  if gn3.val { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Three Type Parameters (3-param Generics)
// ============================================================

pub type Triple[A, B, C] = {
  a: A;
  b: B;
  c: C;
} derive[Clone]

pub fn Triple.new[A, B, C](a: A, b: B, c: C) -> Triple[A, B, C] {
  return Triple[A, B, C]{ a: a, b: b, c: c };
}

pub fn Triple.project_a[A, B, C]() -> A {
  return a;
}

pub fn Triple.project_b[A, B, C]() -> B {
  return b;
}

pub fn Triple.project_c[A, B, C]() -> C {
  return c;
}

pub fn zip_triple[A, B, C](a: A, b: B, c: C) -> Triple[A, B, C] {
  return Triple.new[A, B, C](a, b, c);
}

pub fn unpack_triple[A, B, C](t: Triple[A, B, C]) -> (A, B, C) {
  return (t.a, t.b, t.c);
}

fn test_three_param_generics() -> Int {
  var score = 0;
  var tr: Triple[Int, Bool, Str] = Triple.new[Int, Bool, Str](42, true, "three");
  if tr.project_a() == 42 { score = score + 1; }
  if tr.project_b() { score = score + 1; }
  if tr.project_c() == "three" { score = score + 1; }

  var tr2 = zip_triple[Float64, Int, Bool](3.14, 10, false);
  if tr2.a == 3.14 { score = score + 1; }
  if tr2.b == 10 { score = score + 1; }
  if !tr2.c { score = score + 1; }

  var tr3: Triple[Int, Int, Int] = Triple.new[Int, Int, Int](1, 2, 3);
  var (x, y, z) = unpack_triple[Int, Int, Int](tr3);
  if x == 1 { score = score + 1; }
  if y == 2 { score = score + 1; }
  if z == 3 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Generic Tagged Union via Enum + Vec nesting
// ============================================================

pub enum TaggedData[T] {
  Missing,
  Numeral(val: Int),
  Wrapped(inner: T),
  Labelled(label: Str, payload: T),
}

pub fn TaggedData.is_present[T]() -> Bool {
  match self {
    Missing => false,
    Numeral(val: _) => true,
    Wrapped(inner: _) => true,
    Labelled(label: _, payload: _) => true,
  }
}

pub fn TaggedData.description[T]() -> Str {
  match self {
    Missing => "missing",
    Numeral(val: _) => "numeral",
    Wrapped(inner: _) => "wrapped",
    Labelled(label: _, payload: _) => "labelled",
  }
}

fn test_tagged_generic_enum() -> Int {
  var score = 0;
  var td1: TaggedData[Int] = Wrapped(inner: 99);
  if td1.is_present() { score = score + 1; }
  if td1.description() == "wrapped" { score = score + 1; }

  var td2: TaggedData[Str] = Labelled(label: "key", payload: "val");
  if td2.is_present() { score = score + 1; }
  if td2.description() == "labelled" { score = score + 1; }

  var td3: TaggedData[Bool] = Missing;
  if !(td3.is_present()) { score = score + 1; }
  if td3.description() == "missing" { score = score + 1; }

  var td4: TaggedData[Int] = Numeral(val: 77);
  if td4.is_present() { score = score + 1; }
  if td4.description() == "numeral" { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_generic_box();
  total = total + s1;
  max_score = max_score + 5;

  var s2 = test_multi_param_constrained();
  total = total + s2;
  max_score = max_score + 7;

  var s3 = test_nested_generic_collection();
  total = total + s3;
  max_score = max_score + 4;

  var s4 = test_generic_enum();
  total = total + s4;
  max_score = max_score + 7;

  var s5 = test_generic_method();
  total = total + s5;
  max_score = max_score + 7;

  var s6 = test_three_param_generics();
  total = total + s6;
  max_score = max_score + 9;

  var s7 = test_tagged_generic_enum();
  total = total + s7;
  max_score = max_score + 8;

  return BenchResult{
    name: "generics_hard",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
