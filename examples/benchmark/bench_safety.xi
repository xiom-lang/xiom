// XIOM -- Borrow Safety Stress Benchmark
// Exercises borrow checker with complex patterns, references, and edge cases.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.safety

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Read Borrow Patterns
// ============================================================

pub type Point = {
  x: Int;
  y: Int;
} derive[Clone]

pub fn Point.new(x: Int, y: Int) -> Point {
  return Point{ x: x, y: y };
}

pub fn Point.dist_sq(self, other: &Point) -> Int {
  var dx = self.x - other.x;
  var dy = self.y - other.y;
  return dx * dx + dy * dy;
}

pub fn Point.midpoint(self, other: &Point) -> Point {
  return Point{ x: (self.x + other.x) / 2, y: (self.y + other.y) / 2 };
}

pub fn compute_with_borrows(p1: &Point, p2: &Point, p3: &Point) -> Int {
  var d1 = p1.dist_sq(p2);
  var d2 = p2.dist_sq(p3);
  var d3 = p3.dist_sq(p1);
  return d1 + d2 + d3;
}

fn test_read_borrows() -> Int {
  var score = 0;
  var p1 = Point.new(0, 0);
  var p2 = Point.new(3, 4);
  var p3 = Point.new(6, 8);

  if p1.dist_sq(&p2) == 25 { score = score + 1; }

  var mid = p1.midpoint(&p2);
  if mid.x == 1 { score = score + 1; }
  if mid.y == 2 { score = score + 1; }

  var sum_dist = compute_with_borrows(&p1, &p2, &p3);
  if sum_dist == 75 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Clone to Avoid Move
// ============================================================

pub type Data = {
  id: Int;
  values: Vec[Int];
} derive[Clone]

pub fn Data.new(id: Int) -> Data {
  return Data{ id: id, values: Vec[Int].new() };
}

pub fn Data.add(self, val: Int) {
  self.values.push(val);
}

pub fn Data.sum(self) -> Int {
  var total = 0;
  var i = 0;
  while i < self.values.len() {
    total = total + self.values[i];
    i = i + 1;
  }
  return total;
}

pub fn process_then_check(d: &Data) -> Int {
  var clone1 = d.clone();
  var clone2 = d.clone();
  return clone1.sum() + clone2.sum();
}

fn test_clone_patterns() -> Int {
  var score = 0;
  var d = Data.new(1);
  d.add(10);
  d.add(20);
  d.add(30);

  if d.sum() == 60 { score = score + 1; }

  // Process using clones to avoid move
  var result = process_then_check(&d);
  if result == 120 { score = score + 1; }

  // Original still valid
  if d.sum() == 60 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Multiple Read Borrows
// ============================================================

pub type Account = {
  balance: Int;
  name: Str;
} derive[Clone]

pub fn Account.new(balance: Int, name: Str) -> Account {
  return Account{ balance: balance, name: name };
}

pub fn compare_balances(a: &Account, b: &Account) -> Int {
  if a.balance > b.balance { return 1; }
  if a.balance < b.balance { return -1; }
  return 0;
}

pub fn total_balance(accounts: &Vec[Account]) -> Int {
  var total = 0;
  var i = 0;
  while i < accounts.len() {
    total = total + accounts[i].balance;
    i = i + 1;
  }
  return total;
}

fn test_multi_borrows() -> Int {
  var score = 0;
  var a1 = Account.new(100, "Alice");
  var a2 = Account.new(200, "Bob");
  var a3 = Account.new(150, "Charlie");

  // Multiple read borrows (passing references to compare)
  var cmp = compare_balances(&a1, &a2);
  if cmp == -1 { score = score + 1; }

  var accounts = [a1.clone(), a2.clone(), a3.clone()];
  var total = total_balance(&accounts);
  if total == 450 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Struct Methods with Self Borrow
// ============================================================

pub type Rect = {
  x: Int;
  y: Int;
  w: Int;
  h: Int;
} derive[Clone]

pub fn Rect.new(x: Int, y: Int, w: Int, h: Int) -> Rect {
  return Rect{ x: x, y: y, w: w, h: h };
}

pub fn Rect.area(self) -> Int {
  return self.w * self.h;
}

pub fn Rect.perimeter(self) -> Int {
  return 2 * (self.w + self.h);
}

pub fn Rect.contains_point(self, px: Int, py: Int) -> Bool {
  return px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h;
}

pub fn Rect.overlaps(self, other: &Rect) -> Bool {
  if self.x + self.w <= other.x || other.x + other.w <= self.x { return false; }
  if self.y + self.h <= other.y || other.y + other.h <= self.y { return false; }
  return true;
}

fn test_method_borrows() -> Int {
  var score = 0;
  var r1 = Rect.new(0, 0, 10, 10);
  if r1.area() == 100 { score = score + 1; }
  if r1.perimeter() == 40 { score = score + 1; }
  if r1.contains_point(5, 5) { score = score + 1; }
  if !(r1.contains_point(15, 5)) { score = score + 1; }

  var r2 = Rect.new(5, 5, 10, 10);
  if r1.overlaps(&r2) { score = score + 1; }

  var r3 = Rect.new(20, 20, 5, 5);
  if !(r1.overlaps(&r3)) { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Ownership Transfer Patterns
// ============================================================

pub type Resource = {
  id: Int;
  data: Vec[Int];
} derive[Clone]

pub fn Resource.new(id: Int) -> Resource {
  return Resource{ id: id, data: Vec[Int].new() };
}

pub fn Resource.fill(self, n: Int) {
  var i = 0;
  while i < n {
    self.data.push(i);
    i = i + 1;
  }
}

pub fn Resource.len(self) -> Int {
  return self.data.len();
}

pub fn take_resource(r: Resource) -> Int {
  var size = r.data.len();
  return size;
}

fn test_ownership() -> Int {
  var score = 0;
  var r1 = Resource.new(1);
  r1.fill(5);
  if r1.len() == 5 { score = score + 1; }

  // Clone before transfer
  var r1_clone = r1.clone();
  var consumed = take_resource(r1);
  // r1 is moved, r1_clone still valid
  if consumed == 5 { score = score + 1; }
  if r1_clone.len() == 5 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_read_borrows();
  total = total + s1;
  max_score = max_score + 4;

  var s2 = test_clone_patterns();
  total = total + s2;
  max_score = max_score + 3;

  var s3 = test_multi_borrows();
  total = total + s3;
  max_score = max_score + 2;

  var s4 = test_method_borrows();
  total = total + s4;
  max_score = max_score + 6;

  var s5 = test_ownership();
  total = total + s5;
  max_score = max_score + 3;

  return BenchResult{
    name: "safety",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
