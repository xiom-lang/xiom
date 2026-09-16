// XIOM -- Deep Ownership Chains Benchmark
// Exercises ownership transfer, clone-before-move, conditional moves,
// struct-embedded Vec ownership, and interleaved borrow patterns.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.ownership

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Owned Data Types
// ============================================================

pub type OwnedData = {
  values: Vec[Int];
  label: Int;
} derive[Clone]

pub fn OwnedData.new(label: Int) -> OwnedData {
  return OwnedData{ values: Vec[Int].new(), label: label };
}

pub fn OwnedData.push(val: Int) {
  values.push(val);
}

pub fn OwnedData.count() -> Int {
  return values.len();
}

pub fn OwnedData.sum() -> Int {
  var total = 0;
  var i = 0;
  while i < values.len() {
    total = total + values[i];
    i = i + 1;
  }
  return total;
}

pub fn OwnedData.item(idx: Int) -> Int {
  return values[idx];
}

// ============================================================
// SECTION 2: Ownership Transfer Chain (3-4 functions)
// ============================================================

pub fn stage_a(d: OwnedData) -> OwnedData {
  var result = d;
  result.push(10);
  result.push(20);
  return result;
}

pub fn stage_b(d: OwnedData) -> OwnedData {
  var result = d;
  result.push(30);
  return result;
}

pub fn stage_c(d: OwnedData) -> OwnedData {
  var result = d;
  result.push(40);
  return result;
}

pub fn stage_final(d: OwnedData) -> Int {
  return d.sum();
}

fn test_ownership_chain() -> Int {
  var score = 0;
  var d1 = OwnedData.new(1);
  d1.push(5);
  var d2 = stage_a(d1);
  var d3 = stage_b(d2);
  var d4 = stage_c(d3);
  var result = stage_final(d4);
  if result == 105 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 3: Clone-Before-Move Patterns
// ============================================================

pub fn clone_then_move(d: OwnedData) -> (OwnedData, OwnedData) {
  var backup = d.clone();
  var processed = stage_a(d);
  return (backup, processed);
}

pub fn multi_clone_route(d: OwnedData) -> Int {
  var a = d.clone();
  var b = a.clone();
  var c = b.clone();
  var consumed = stage_c(c);
  return consumed.sum() + d.sum();
}

fn test_clone_before_move() -> Int {
  var score = 0;
  var orig = OwnedData.new(10);
  orig.push(1);
  orig.push(2);
  var (bk, moved) = clone_then_move(orig);
  if bk.count() == 2 { score = score + 1; }
  if bk.sum() == 3 { score = score + 1; }
  if moved.count() == 4 { score = score + 1; }
  if moved.sum() == 33 { score = score + 1; }

  var src = OwnedData.new(20);
  src.push(3);
  src.push(4);
  var multi = multi_clone_route(src);
  if multi == 144 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Interleaved Read Borrows During Ownership Chain
// ============================================================

pub fn inspect_data(d: &OwnedData) -> Int {
  return d.count() * 100 + d.sum();
}

pub fn chain_with_inspect(d: OwnedData) -> Int {
  var stage1 = stage_a(d);
  var info1 = inspect_data(&stage1);
  var stage2 = stage_b(stage1);
  var info2 = inspect_data(&stage2);
  var stage3 = stage_c(stage2);
  var info3 = inspect_data(&stage3);
  return info1 + info2 + info3 + stage_final(stage3);
}

fn test_interleaved_borrows() -> Int {
  var score = 0;
  var d = OwnedData.new(30);
  d.push(1);
  var result = chain_with_inspect(d);
  if result == 3334 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 5: Move-in-Conditional Patterns
// ============================================================

pub fn choose_path(flag: Bool, d1: OwnedData, d2: OwnedData) -> OwnedData {
  if flag {
    return stage_b(d1);
  } else {
    return stage_c(d2);
  }
}

pub fn conditional_move(flag: Bool, d: OwnedData) -> OwnedData {
  if flag {
    var processed = stage_a(d);
    return processed;
  } else {
    var cloned = d.clone();
    return stage_b(cloned);
  }
}

fn test_move_conditional() -> Int {
  var score = 0;
  var a = OwnedData.new(40);
  a.push(5);
  var b = OwnedData.new(50);
  b.push(8);
  var path_true = choose_path(true, a, b);
  if path_true.item(0) == 5 { score = score + 1; }
  if path_true.item(1) == 10 { score = score + 1; }
  if path_true.item(2) == 20 { score = score + 1; }
  if path_true.item(3) == 30 { score = score + 1; }

  var c = OwnedData.new(60);
  c.push(3);
  var cond1 = conditional_move(true, c);
  if cond1.item(0) == 3 { score = score + 1; }
  if cond1.item(1) == 10 { score = score + 1; }
  if cond1.item(2) == 20 { score = score + 1; }

  var d = OwnedData.new(70);
  d.push(7);
  var cond2 = conditional_move(false, d);
  if cond2.item(0) == 7 { score = score + 1; }
  if cond2.item(1) == 30 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Struct-with-Vec Through Function Chain
// ============================================================

pub type Payload = {
  items: Vec[Int];
  version: Int;
  flags: Int;
} derive[Clone]

pub fn Payload.new(ver: Int) -> Payload {
  return Payload{ items: Vec[Int].new(), version: ver, flags: 0 };
}

pub fn Payload.attach(val: Int) {
  items.push(val);
}

pub fn payload_xform_1(p: Payload) -> Payload {
  var out = p;
  out.attach(100);
  return out;
}

pub fn payload_xform_2(p: Payload) -> Payload {
  var out = p;
  out.attach(200);
  return out;
}

pub fn payload_xform_3(p: Payload) -> Payload {
  var out = p;
  out.attach(300);
  return out;
}

pub fn payload_inspect(p: &Payload) -> Int {
  var s = 0;
  var i = 0;
  while i < p.items.len() {
    s = s + p.items[i];
    i = i + 1;
  }
  return s;
}

fn test_payload_chain() -> Int {
  var score = 0;
  var pl = Payload.new(1);
  pl.attach(1);
  pl.attach(2);
  var pl2 = payload_xform_1(pl);
  var mid = payload_inspect(&pl2);
  var pl3 = payload_xform_2(pl2);
  var pl4 = payload_xform_3(pl3);
  if mid == 103 { score = score + 1; }
  if payload_inspect(&pl4) == 603 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 7: Deep Ownership Clone-Then-Consume
// ============================================================

pub type Token = {
  content: Vec[Int];
  code: Int;
} derive[Clone]

pub fn Token.new(code: Int) -> Token {
  return Token{ content: Vec[Int].new(), code: code };
}

pub fn Token.emit(val: Int) {
  content.push(val);
}

pub fn token_preprocess(t: Token) -> (Token, Token) {
  var clone_safe = t.clone();
  clone_safe.emit(500);
  var consumed = token_finalize(t);
  return (clone_safe, consumed);
}

pub fn token_finalize(t: Token) -> Token {
  var out = t;
  out.code = out.code * 10;
  return out;
}

pub fn token_consume(t: Token) -> Int {
  var s = 0;
  var i = 0;
  while i < t.content.len() {
    s = s + t.content[i];
    i = i + 1;
  }
  return s + t.code;
}

fn test_token_ownership() -> Int {
  var score = 0;
  var tk = Token.new(7);
  tk.emit(11);
  tk.emit(13);
  var (safe, consumed) = token_preprocess(tk);
  if safe.code == 7 { score = score + 1; }
  if token_consume(safe) == 507 { score = score + 1; }
  if consumed.code == 70 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 8: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_ownership_chain();
  total = total + s1;
  max_score = max_score + 1;

  var s2 = test_clone_before_move();
  total = total + s2;
  max_score = max_score + 6;

  var s3 = test_interleaved_borrows();
  total = total + s3;
  max_score = max_score + 1;

  var s4 = test_move_conditional();
  total = total + s4;
  max_score = max_score + 10;

  var s5 = test_payload_chain();
  total = total + s5;
  max_score = max_score + 2;

  var s6 = test_token_ownership();
  total = total + s6;
  max_score = max_score + 3;

  return BenchResult{
    name: "ownership",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
