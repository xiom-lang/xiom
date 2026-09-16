// XIOM -- Interface Dispatch Stress Benchmark
// Exercises interface definitions, interface satisfaction, and generic dispatch.
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module benchmark.interfaces

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Interface Definitions
// ============================================================

pub type Circle = {
  radius: Float64;
} derive[Eq, Clone]

pub type Rectangle = {
  width: Float64;
  height: Float64;
} derive[Eq, Clone]

pub type Triangle = {
  base: Float64;
  height: Float64;
} derive[Eq, Clone]

pub type Square = {
  side: Float64;
} derive[Eq, Clone]

// Interface methods implemented directly on types

pub fn Circle.area() -> Float64 {
  return 3.14159 * radius * radius;
}

pub fn Circle.perimeter() -> Float64 {
  return 2.0 * 3.14159 * radius;
}

pub fn Rectangle.area() -> Float64 {
  return width * height;
}

pub fn Rectangle.perimeter() -> Float64 {
  return 2.0 * (width + height);
}

pub fn Triangle.area() -> Float64 {
  return 0.5 * base * height;
}

pub fn Square.area() -> Float64 {
  return side * side;
}

pub fn Square.perimeter() -> Float64 {
  return 4.0 * side;
}

// ============================================================
// SECTION 2: Generic Dispatch
// ============================================================

pub fn total_area[T](shapes: &Vec[T], area_fn: fn(&T) -> Float64) -> Float64 {
  var total = 0.0;
  var i = 0;
  while i < shapes.len() {
    total = total + area_fn(&shapes[i]);
    i = i + 1;
  }
  return total;
}

pub fn count_larger_than[T](shapes: &Vec[T], threshold: Float64, area_fn: fn(&T) -> Float64) -> Int {
  var count = 0;
  var i = 0;
  while i < shapes.len() {
    if area_fn(&shapes[i]) > threshold {
      count = count + 1;
    }
    i = i + 1;
  }
  return count;
}

fn circle_area_fn(c: &Circle) -> Float64 { return c.area(); }
fn rect_area_fn(r: &Rectangle) -> Float64 { return r.area(); }
fn square_area_fn(s: &Square) -> Float64 { return s.area(); }

fn test_interface_dispatch() -> Int {
  var score = 0;
  var c = Circle{ radius: 2.0 };
  var r = Rectangle{ width: 3.0, height: 4.0 };
  var s = Square{ side: 5.0 };

  // Direct method calls
  var ca = c.area();
  if ca > 12.5 && ca < 12.6 { score = score + 1; }

  if r.area() == 12.0 { score = score + 1; }
  if s.area() == 25.0 { score = score + 1; }

  // Perimeters
  var cp = c.perimeter();
  if cp > 12.5 && cp < 12.6 { score = score + 1; }
  if r.perimeter() == 14.0 { score = score + 1; }
  if s.perimeter() == 20.0 { score = score + 1; }

  // Generic dispatch via function pointer
  var circles = [c.clone(), Circle{ radius: 3.0 }];
  var area = total_area(&circles, circle_area_fn);
  if area > 40.8 && area < 40.9 { score = score + 1; }

  // Count larger than threshold
  var rects = [
    Rectangle{ width: 1.0, height: 1.0 },
    Rectangle{ width: 3.0, height: 4.0 },
    Rectangle{ width: 2.0, height: 2.0 },
  ];
  var big_count = count_larger_than(&rects, 5.0, rect_area_fn);
  if big_count == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Multiple Interface Satisfaction
// ============================================================

pub type Point2D = {
  x: Float64;
  y: Float64;
} derive[Eq, Clone]

pub fn Point2D.distance_sq(self, other: &Point2D) -> Float64 {
  var dx = self.x - other.x;
  var dy = self.y - other.y;
  return dx * dx + dy * dy;
}

pub fn Point2D.magnitude_sq(self) -> Float64 {
  return self.x * self.x + self.y * self.y;
}

pub fn Point2D.add(self, other: &Point2D) -> Float64 {
  return self.x + other.x + self.y + other.y;
}

pub fn distance_between(p1: &Point2D, p2: &Point2D) -> Float64 {
  var dsq = p1.distance_sq(p2);
  // Approximate sqrt
  if dsq == 0.0 { return 0.0; }
  var guess = dsq / 2.0;
  var i = 0;
  while i < 20 {
    guess = (guess + dsq / guess) / 2.0;
    i = i + 1;
  }
  return guess;
}

fn test_point_interface() -> Int {
  var score = 0;
  var p1 = Point2D{ x: 0.0, y: 0.0 };
  var p2 = Point2D{ x: 3.0, y: 4.0 };

  if p1.distance_sq(&p2) == 25.0 { score = score + 1; }
  if p2.magnitude_sq() == 25.0 { score = score + 1; }

  var dist = distance_between(&p1, &p2);
  if dist > 4.9 && dist < 5.1 { score = score + 1; }

  if p2.add(&p1) == 7.0 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Record-Keeping Interface
// ============================================================

pub type Record = {
  id: Int;
  value: Int;
  timestamp: Int;
} derive[Eq, Clone]

pub fn Record.new(id: Int, value: Int) -> Record {
  return Record{ id: id, value: value, timestamp: id * 100 };
}

pub fn Record.update(val: Int) -> Record {
  return Record{ id: id, value: val, timestamp: id * 100 };
}

pub fn Record.is_active() -> Bool {
  return value > 0;
}

pub fn Record.score() -> Int {
  return value;
}

pub fn find_by_id(records: &Vec[Record], target_id: Int) -> Option[Record] {
  var i = 0;
  while i < records.len() {
    if records[i].id == target_id {
      return Some(records[i].clone());
    }
    i = i + 1;
  }
  return None;
}

pub fn sum_scores(records: &Vec[Record]) -> Int {
  var total = 0;
  var i = 0;
  while i < records.len() {
    total = total + records[i].score();
    i = i + 1;
  }
  return total;
}

pub fn count_active(records: &Vec[Record]) -> Int {
  var count = 0;
  var i = 0;
  while i < records.len() {
    if records[i].is_active() { count = count + 1; }
    i = i + 1;
  }
  return count;
}

fn test_record_interface() -> Int {
  var score = 0;
  var recs = Vec[Record].new();
  recs.push(Record.new(1, 10));
  recs.push(Record.new(2, 0));
  recs.push(Record.new(3, 30));

  if sum_scores(&recs) == 40 { score = score + 1; }
  if count_active(&recs) == 2 { score = score + 1; }

  match find_by_id(&recs, 2) {
    Some(r) => if r.value == 0 { score = score + 1; }
    None => {}
  }

  match find_by_id(&recs, 99) {
    Some(_) => {}
    None => { score = score + 1; }
  }

  return score;
}

// ============================================================
// SECTION 5: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_interface_dispatch();
  total = total + s1;
  max_score = max_score + 8;

  var s2 = test_point_interface();
  total = total + s2;
  max_score = max_score + 4;

  var s3 = test_record_interface();
  total = total + s3;
  max_score = max_score + 4;

  return BenchResult{
    name: "interfaces",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
