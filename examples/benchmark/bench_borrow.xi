// XIOM -- Borrow Checker Torture Benchmark
// Exercises lexical borrows, multiple simultaneous reads, borrow-in-loop,
// nested scope borrows, clone-to-extend-lifetime, and method borrows.
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module benchmark.borrow

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Types Under Test
// ============================================================

pub type Subject = {
  id: Int;
  x: Int;
  y: Int;
  z: Int;
} derive[Clone]

pub fn Subject.new(id: Int, x: Int, y: Int, z: Int) -> Subject {
  return Subject{ id: id, x: x, y: y, z: z };
}

pub fn Subject.magnitude() -> Int {
  return x * x + y * y + z * z;
}

pub fn Subject.sum_components() -> Int {
  return x + y + z;
}

pub fn Subject.read_x() -> Int {
  return x;
}

pub fn Subject.read_y() -> Int {
  return y;
}

pub fn Subject.read_z() -> Int {
  return z;
}

// ============================================================
// SECTION 2: Multiple Simultaneous Read Borrows
// ============================================================

pub fn borrow_three(a: &Subject, b: &Subject, c: &Subject) -> Int {
  return a.sum_components() + b.sum_components() + c.sum_components();
}

pub fn borrow_four(a: &Subject, b: &Subject, c: &Subject, d: &Subject) -> Int {
  return borrow_three(a, b, c) + d.sum_components();
}

pub fn multi_read_on_same_object() -> Int {
  var obj = Subject.new(1, 3, 4, 5);
  var mag = obj.magnitude();
  var sx = obj.read_x();
  var sy = obj.read_y();
  var sz = obj.read_z();
  var sc = obj.sum_components();
  var id_check = obj.id;
  return mag + sx + sy + sz + sc + id_check;
}

fn test_multi_read_borrows() -> Int {
  var score = 0;
  var a = Subject.new(1, 1, 2, 3);
  var b = Subject.new(2, 4, 5, 6);
  var c = Subject.new(3, 7, 8, 9);
  var d = Subject.new(4, 10, 11, 12);

  if borrow_three(&a, &b, &c) == 45 { score = score + 1; }
  if borrow_four(&a, &b, &c, &d) == 78 { score = score + 1; }

  var mro = multi_read_on_same_object();
  if mro == 72 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Move After All Read Borrows Expire
// ============================================================

pub fn move_after_borrows_expire() -> Int {
  var obj = Subject.new(10, 5, 6, 7);
  var r1 = obj.sum_components();
  var r2 = obj.magnitude();
  var r3 = read_two_fields(&obj);
  var owned = consume_subject(obj);
  return r1 + r2 + r3 + owned;
}

pub fn read_two_fields(s: &Subject) -> Int {
  return s.x + s.y;
}

pub fn consume_subject(s: Subject) -> Int {
  return s.id * 1000 + s.sum_components();
}

pub fn borrow_then_move_with_temp() -> Int {
  var obj = Subject.new(20, 1, 1, 1);
  var info = obj.sum_components();
  return info + consume_subject(obj);
}

fn test_move_after_borrows() -> Int {
  var score = 0;
  var r1 = move_after_borrows_expire();
  if r1 == 10148 { score = score + 1; }
  var r2 = borrow_then_move_with_temp();
  if r2 == 20003 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 4: Borrow in Loop Body -- Expires Each Iteration
// ============================================================

pub type Element = {
  value: Int;
  tag: Int;
} derive[Clone]

pub fn Element.new(v: Int, t: Int) -> Element {
  return Element{ value: v, tag: t };
}

pub fn borrow_loop_accumulate(items: &Vec[Int]) -> Int {
  var total = 0;
  var i = 0;
  while i < items.len() {
    total = total + items[i];
    i = i + 1;
  }
  return total;
}

pub fn loop_body_creates_short_borrows(data: &Vec[Element]) -> Int {
  var sum_val = 0;
  var sum_tag = 0;
  var i = 0;
  while i < data.len() {
    sum_val = sum_val + data[i].value;
    sum_tag = sum_tag + data[i].tag;
    i = i + 1;
  }
  return sum_val + sum_tag;
}

fn test_borrow_loop() -> Int {
  var score = 0;
  var nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  var acc = borrow_loop_accumulate(&nums);
  if acc == 55 { score = score + 1; }

  var elems = Vec[Element].new();
  elems.push(Element.new(10, 1));
  elems.push(Element.new(20, 2));
  elems.push(Element.new(30, 3));
  var lb = loop_body_creates_short_borrows(&elems);
  if lb == 66 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Nested Scope Borrows
// ============================================================

pub type Block = {
  data: Vec[Int];
  sum_cache: Int;
} derive[Clone]

pub fn Block.new() -> Block {
  return Block{ data: Vec[Int].new(), sum_cache: 0 };
}

pub fn Block.add(val: Int) {
  data.push(val);
  sum_cache = 0;
}

pub fn Block.len() -> Int {
  return data.len();
}

pub fn nested_scope_borrows(outer: &Block, inner_val: Int) -> Int {
  var result = outer.len();
  if result > 0 {
    result = result * inner_val;
  } else {
    result = inner_val;
  }
  return result;
}

pub fn inner_borrow_outer_move(b: Block) -> Int {
  var len_cache = b.len();
  var computed = nested_scope_borrows(&b, len_cache);
  return consume_block(b) + computed;
}

pub fn consume_block(b: Block) -> Int {
  return b.len() * 10;
}

fn test_nested_scope_borrows() -> Int {
  var score = 0;
  var blk = Block.new();
  blk.add(1);
  blk.add(2);
  blk.add(3);
  var result = inner_borrow_outer_move(blk);
  if result == 39 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 6: Clone to Extend Lifetime Instead of Borrowing
// ============================================================

pub type Record = {
  fields: Vec[Int];
  kind: Int;
} derive[Clone]

pub fn Record.new(kind: Int) -> Record {
  return Record{ fields: Vec[Int].new(), kind: kind };
}

pub fn Record.store(val: Int) {
  fields.push(val);
}

pub fn Record.summary(r: &Record) -> Int {
  var s = 0;
  var i = 0;
  while i < r.fields.len() {
    s = s + r.fields[i];
    i = i + 1;
  }
  return s + r.kind;
}

pub fn clone_to_extend_lifetime(r: Record) -> Int {
  var snapshot = r.clone();
  var view = snapshot.clone();
  var s1 = Record.summary(&snapshot);
  var s2 = Record.summary(&view);
  var moved_out = consume_record(r);
  return s1 + s2 + moved_out;
}

pub fn consume_record(r: Record) -> Int {
  var s = 0;
  var i = 0;
  while i < r.fields.len() {
    s = s + r.fields[i];
    i = i + 1;
  }
  return s;
}

fn test_clone_to_extend() -> Int {
  var score = 0;
  var rec = Record.new(5);
  rec.store(10);
  rec.store(20);
  var result = clone_to_extend_lifetime(rec);
  if result == 115 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 7: Method Borrows on Types
// ============================================================

pub type Metrics = {
  a: Int;
  b: Int;
  c: Int;
  d: Int;
} derive[Clone]

pub fn Metrics.new(a: Int, b: Int, c: Int, d: Int) -> Metrics {
  return Metrics{ a: a, b: b, c: c, d: d };
}

pub fn Metrics.read_all() -> Int {
  return a + b + c + d;
}

pub fn Metrics.read_ab() -> Int {
  return a + b;
}

pub fn Metrics.read_cd() -> Int {
  return c + d;
}

pub fn Metrics.read_ac() -> Int {
  return a + c;
}

pub fn method_borrow_chain(m: &Metrics) -> Int {
  var ab = m.read_ab();
  var cd = m.read_cd();
  var ac = m.read_ac();
  return ab + cd + ac;
}

pub fn method_borrow_vs_field_access(m: &Metrics) -> Int {
  var from_method = m.read_all();
  var from_field = m.a + m.b + m.c + m.d;
  return from_method + from_field;
}

fn test_method_borrows() -> Int {
  var score = 0;
  var met = Metrics.new(1, 2, 3, 4);
  var ch = method_borrow_chain(&met);
  if ch == 18 { score = score + 1; }
  var vs = method_borrow_vs_field_access(&met);
  if vs == 20 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 8: Borrowed Vec Contents
// ============================================================

pub type Bag = {
  contents: Vec[Int];
} derive[Clone]

pub fn Bag.new() -> Bag {
  return Bag{ contents: Vec[Int].new() };
}

pub fn Bag.insert(val: Int) {
  contents.push(val);
}

pub fn Bag.peek_all(b: &Bag) -> Int {
  var s = 0;
  var i = 0;
  while i < b.contents.len() {
    s = s + b.contents[i];
    i = i + 1;
  }
  return s;
}

pub fn Bag.seq_access(b: &Bag) -> Int {
  var first = 0;
  if b.contents.len() > 0 { first = b.contents[0]; }
  var last = 0;
  if b.contents.len() > 0 {
    last = b.contents[b.contents.len() - 1];
  }
  return first + last;
}

fn test_vec_borrow_patterns() -> Int {
  var score = 0;
  var bag = Bag.new();
  bag.insert(5);
  bag.insert(10);
  bag.insert(15);
  bag.insert(20);
  var pa = Bag.peek_all(&bag);
  if pa == 50 { score = score + 1; }
  var sa = Bag.seq_access(&bag);
  if sa == 25 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 9: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_multi_read_borrows();
  total = total + s1;
  max_score = max_score + 3;

  var s2 = test_move_after_borrows();
  total = total + s2;
  max_score = max_score + 2;

  var s3 = test_borrow_loop();
  total = total + s3;
  max_score = max_score + 2;

  var s4 = test_nested_scope_borrows();
  total = total + s4;
  max_score = max_score + 1;

  var s5 = test_clone_to_extend();
  total = total + s5;
  max_score = max_score + 1;

  var s6 = test_method_borrows();
  total = total + s6;
  max_score = max_score + 2;

  var s7 = test_vec_borrow_patterns();
  total = total + s7;
  max_score = max_score + 2;

  return BenchResult{
    name: "borrow",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
