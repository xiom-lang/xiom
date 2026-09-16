// XIOM -- Massive Contract Verification Benchmark
// Exercises generic invariants, contract chains, multi-ensures,
// combined requires+ensures, and invariant-rich types.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.contracts_hard

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Generic Type with Multiple Invariants
// ============================================================

pub type BoundedVec[T] = {
  items: Vec[T];
  limit: Int;
  invariant: items.len() <= limit;
  invariant: limit > 0;
  invariant: limit <= 16;
}

pub fn BoundedVec.new[T](max: Int) -> BoundedVec[T]
  requires: max > 0
  requires: max <= 16
{
  return BoundedVec[T]{ items: Vec[T].new(), limit: max };
}

pub fn BoundedVec.insert[T](val: T) -> Bool
  requires: items.len() < limit
  ensures: result == true
{
  items.push(val);
  return true;
}

pub fn BoundedVec.len[T]() -> Int {
  return items.len();
}

pub fn BoundedVec.is_full[T]() -> Bool {
  return items.len() >= limit;
}

fn test_bounded_vec() -> Int {
  var score = 0;
  var bv: BoundedVec[Int] = BoundedVec.new[Int](5);
  if bv.len() == 0 { score = score + 1; }
  if !(bv.is_full()) { score = score + 1; }

  if bv.insert(10) { score = score + 1; }
  if bv.insert(20) { score = score + 1; }
  if bv.insert(30) { score = score + 1; }
  if bv.len() == 3 { score = score + 1; }

  bv.insert(40);
  bv.insert(50);
  if bv.is_full() { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Contract Chain -- 3 Functions
//   fn A with ensures -> fn B with requires -> fn C with requires
// ============================================================

pub fn deposit_and_bonus(base: Int, extra: Int) -> Int
  requires: base >= 0
  requires: extra >= 0
  ensures: result >= base
  ensures: result >= extra
{
  var intermediate = apply_bonus(base, extra);
  return finalize_total(intermediate);
}

pub fn apply_bonus(base: Int, extra: Int) -> Int
  requires: base >= 0
  requires: extra >= 0
{
  var mult = compute_multiplier(base);
  return base + extra * mult;
}

pub fn finalize_total(val: Int) -> Int
  requires: val >= 0
  ensures: result >= val
{
  return val + 1;
}

pub fn compute_multiplier(base: Int) -> Int
  requires: base >= 0
{
  if base >= 100 { return 3; }
  if base >= 50 { return 2; }
  return 1;
}

fn test_contract_chain() -> Int {
  var score = 0;
  var r1 = deposit_and_bonus(0, 10);
  if r1 == 11 { score = score + 1; }
  var r2 = deposit_and_bonus(50, 10);
  if r2 == 21 { score = score + 1; }
  var r3 = deposit_and_bonus(100, 10);
  if r3 == 31 { score = score + 1; }
  var r4 = deposit_and_bonus(200, 20);
  if r4 == 61 { score = score + 1; }
  return score;
}

// ============================================================
// SECTION 3: Multiple Ensures Clauses (3+ on One Function)
// ============================================================

pub type Normalized = {
  value: Int;
  min_val: Int;
  max_val: Int;
  invariant: value >= min_val;
  invariant: value <= max_val;
}

pub fn Normalized.new(v: Int, lo: Int, hi: Int) -> Normalized
  requires: lo <= hi
  ensures: result.value >= lo
  ensures: result.value <= hi
  ensures: result.min_val == lo
  ensures: result.max_val == hi
  ensures: result.min_val <= result.max_val
{
  var clamped = v;
  if clamped < lo { clamped = lo; }
  if clamped > hi { clamped = hi; }
  return Normalized{ value: clamped, min_val: lo, max_val: hi };
}

pub fn Normalized.ratio() -> Int {
  if max_val == min_val { return 100; }
  return (value - min_val) * 100 / (max_val - min_val);
}

pub fn Normalized.extend(new_hi: Int) -> Normalized
  requires: new_hi >= max_val
  ensures: result.min_val == min_val@pre
  ensures: result.max_val == new_hi
  ensures: result.value == value@pre
{
  return Normalized{ value: value, min_val: min_val, max_val: new_hi };
}

fn test_multi_ensures() -> Int {
  var score = 0;
  var n1 = Normalized.new(50, 0, 100);
  if n1.value == 50 { score = score + 1; }
  if n1.min_val == 0 { score = score + 1; }
  if n1.max_val == 100 { score = score + 1; }
  if n1.ratio() == 50 { score = score + 1; }

  var n2 = Normalized.new(-10, 0, 100);
  if n2.value == 0 { score = score + 1; }

  var n3 = Normalized.new(200, 0, 100);
  if n3.value == 100 { score = score + 1; }

  var n4 = n1.extend(200);
  if n4.max_val == 200 { score = score + 1; }
  if n4.value == 50 { score = score + 1; }
  if n4.min_val == 0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Functions with Both Requires AND Ensures
// ============================================================

pub type Fraction = {
  numerator: Int;
  denominator: Int;
  invariant: denominator > 0;
}

pub fn Fraction.new(num: Int, den: Int) -> Fraction
  requires: den > 0
  ensures: result.numerator == num
  ensures: result.denominator == den
{
  return Fraction{ numerator: num, denominator: den };
}

pub fn Fraction.add(self, other: &Fraction) -> Fraction
  requires: self.denominator > 0
  requires: other.denominator > 0
  ensures: result.denominator > 0
  ensures: result.denominator == self.denominator * other.denominator
  ensures: result.numerator == self.numerator * other.denominator + other.numerator * self.denominator
{
  return Fraction{
    numerator: self.numerator * other.denominator + other.numerator * self.denominator,
    denominator: self.denominator * other.denominator,
  };
}

pub fn Fraction.multiply(self, other: &Fraction) -> Fraction
  requires: self.denominator > 0
  requires: other.denominator > 0
  ensures: result.denominator > 0
  ensures: result.denominator == self.denominator * other.denominator
{
  return Fraction{
    numerator: self.numerator * other.numerator,
    denominator: self.denominator * other.denominator,
  };
}

fn test_requires_and_ensures() -> Int {
  var score = 0;
  var f1 = Fraction.new(1, 2);
  if f1.numerator == 1 { score = score + 1; }
  if f1.denominator == 2 { score = score + 1; }

  var f2 = Fraction.new(1, 3);
  var f3 = f1.add(&f2);
  if f3.numerator == 5 { score = score + 1; }
  if f3.denominator == 6 { score = score + 1; }

  var f4 = Fraction.new(1, 4);
  var f5 = Fraction.new(1, 5);
  var f6 = f4.add(&f5);
  if f6.numerator == 9 { score = score + 1; }
  if f6.denominator == 20 { score = score + 1; }

  var f7 = Fraction.new(2, 3);
  var f8 = Fraction.new(3, 4);
  var f9 = f7.multiply(&f8);
  if f9.numerator == 6 { score = score + 1; }
  if f9.denominator == 12 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Invariants on Return Types
// ============================================================

pub type Score = {
  current: Int;
  ceiling: Int;
  invariant: current >= 0;
  invariant: ceiling > 0;
  invariant: current <= ceiling;
}

pub fn Score.new(ceiling: Int) -> Score
  requires: ceiling > 0
{
  return Score{ current: 0, ceiling: ceiling };
}

pub fn Score.increment(amount: Int) -> Score
  requires: amount >= 0
  ensures: result.current >= current@pre
  ensures: result.current <= ceiling
{
  var new_val = current + amount;
  if new_val > ceiling { new_val = ceiling; }
  return Score{ current: new_val, ceiling: ceiling };
}

pub fn Score.decrement(amount: Int) -> Score
  requires: amount >= 0
  ensures: result.current <= current@pre
  ensures: result.current >= 0
{
  var new_val = current - amount;
  if new_val < 0 { new_val = 0; }
  return Score{ current: new_val, ceiling: ceiling };
}

pub fn Score.is_at_max() -> Bool {
  return current >= ceiling;
}

pub fn Score.at_floor() -> Bool {
  return current <= 0;
}

fn test_invariant_return() -> Int {
  var score_count = 0;
  var sc = Score.new(100);
  if sc.current == 0 { score_count = score_count + 1; }
  if sc.ceiling == 100 { score_count = score_count + 1; }
  if sc.at_floor() { score_count = score_count + 1; }

  var sc2 = sc.increment(50);
  if sc2.current == 50 { score_count = score_count + 1; }
  if !(sc2.at_floor()) { score_count = score_count + 1; }
  if !(sc2.is_at_max()) { score_count = score_count + 1; }

  var sc3 = sc2.increment(60);
  if sc3.current == 100 { score_count = score_count + 1; }
  if sc3.is_at_max() { score_count = score_count + 1; }

  var sc4 = sc3.decrement(30);
  if sc4.current == 70 { score_count = score_count + 1; }

  var sc5 = sc4.decrement(100);
  if sc5.current == 0 { score_count = score_count + 1; }
  if sc5.at_floor() { score_count = score_count + 1; }

  return score_count;
}

// ============================================================
// SECTION 6: Generic Contract Type with 2 Invariants
// ============================================================

pub type ValidatedPair[T] = {
  left: T;
  right: T;
  checksum: Int;
  invariant: checksum >= 0;
  invariant: checksum <= 9999;
}

pub fn ValidatedPair.new[T](l: T, r: T, cs: Int) -> ValidatedPair[T]
  requires: cs >= 0
  requires: cs <= 9999
  ensures: result.left == l
  ensures: result.right == r
{
  return ValidatedPair[T]{ left: l, right: r, checksum: cs };
}

pub fn ValidatedPair.matches[T: Eq](l: T, r: T) -> Bool {
  return left == l && right == r;
}

pub fn ValidatedPair.refresh_cs[T]() -> ValidatedPair[T]
  ensures: result.checksum == 0
  ensures: result.left == left@pre
  ensures: result.right == right@pre
{
  return ValidatedPair[T]{ left: left, right: right, checksum: 0 };
}

fn test_validated_pair() -> Int {
  var score = 0;
  var vp: ValidatedPair[Int] = ValidatedPair.new[Int](5, 10, 100);
  if vp.left == 5 { score = score + 1; }
  if vp.right == 10 { score = score + 1; }
  if vp.checksum == 100 { score = score + 1; }
  if vp.matches(5, 10) { score = score + 1; }
  if !(vp.matches(5, 99)) { score = score + 1; }

  var vp2 = vp.refresh_cs();
  if vp2.checksum == 0 { score = score + 1; }
  if vp2.left == 5 { score = score + 1; }
  if vp2.right == 10 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Deep Contract Requirements
// ============================================================

pub type Gauge = {
  reading: Int;
  low_limit: Int;
  high_limit: Int;
  invariant: low_limit <= high_limit;
  invariant: reading >= low_limit;
  invariant: reading <= high_limit;
}

pub fn Gauge.new(rd: Int, lo: Int, hi: Int) -> Gauge
  requires: lo <= hi
  requires: rd >= lo
  requires: rd <= hi
{
  return Gauge{ reading: rd, low_limit: lo, high_limit: hi };
}

pub fn Gauge.adjust(delta: Int) -> Gauge
  requires: reading + delta >= low_limit
  requires: reading + delta <= high_limit
  ensures: result.reading == reading@pre + delta
  ensures: result.low_limit == low_limit@pre
  ensures: result.high_limit == high_limit@pre
{
  return Gauge{ reading: reading + delta, low_limit: low_limit, high_limit: high_limit };
}

pub fn Gauge.reset() -> Gauge
  ensures: result.reading == low_limit
  ensures: result.low_limit == low_limit@pre
  ensures: result.high_limit == high_limit@pre
{
  return Gauge{ reading: low_limit, low_limit: low_limit, high_limit: high_limit };
}

fn test_gauge_contracts() -> Int {
  var score = 0;
  var g = Gauge.new(50, 0, 100);
  if g.reading == 50 { score = score + 1; }
  if g.low_limit == 0 { score = score + 1; }
  if g.high_limit == 100 { score = score + 1; }

  var g2 = g.adjust(20);
  if g2.reading == 70 { score = score + 1; }
  if g2.low_limit == 0 { score = score + 1; }

  var g3 = g2.adjust(-30);
  if g3.reading == 40 { score = score + 1; }

  var g4 = g3.reset();
  if g4.reading == 0 { score = score + 1; }
  if g4.high_limit == 100 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_bounded_vec();
  total = total + s1;
  max_score = max_score + 7;

  var s2 = test_contract_chain();
  total = total + s2;
  max_score = max_score + 4;

  var s3 = test_multi_ensures();
  total = total + s3;
  max_score = max_score + 9;

  var s4 = test_requires_and_ensures();
  total = total + s4;
  max_score = max_score + 9;

  var s5 = test_invariant_return();
  total = total + s5;
  max_score = max_score + 11;

  var s6 = test_validated_pair();
  total = total + s6;
  max_score = max_score + 8;

  var s7 = test_gauge_contracts();
  total = total + s7;
  max_score = max_score + 8;

  return BenchResult{
    name: "contracts_hard",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
