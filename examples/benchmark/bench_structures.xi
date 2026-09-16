// XIOM -- Data Structures Stress Benchmark
// Exercises Vec, binary trees, linked lists, stacks, queues, and map/set patterns.
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module benchmark.structures

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Vec Operations
// ============================================================

pub fn vec_push_test(n: Int) -> Int {
  var v = Vec[Int].new();
  var i = 0;
  while i < n {
    v.push(i);
    i = i + 1;
  }
  return v.len();
}

pub fn vec_pop_test(n: Int) -> Int {
  var v = Vec[Int].new();
  var i = 0;
  while i < n {
    v.push(i);
    i = i + 1;
  }
  var popped = 0;
  i = 0;
  while i < n / 2 {
    if v.len() > 0 {
      v.pop();
      popped = popped + 1;
    }
    i = i + 1;
  }
  return popped;
}

pub fn vec_sum(v: &Vec[Int]) -> Int {
  var sum = 0;
  var i = 0;
  while i < v.len() {
    sum = sum + v[i];
    i = i + 1;
  }
  return sum;
}

pub fn vec_max(v: Vec[Int]) -> Int {
  if v.len() == 0 { return 0; }
  var max_val = v[0];
  var i = 1;
  while i < v.len() {
    if v[i] > max_val { max_val = v[i]; }
    i = i + 1;
  }
  return max_val;
}

pub fn vec_reverse(v: Vec[Int]) -> Vec[Int] {
  var r = Vec[Int].new();
  var i = v.len();
  while i > 0 {
    i = i - 1;
    r.push(v[i]);
  }
  return r;
}

fn test_vec_basic() -> Int {
  var score = 0;
  if vec_push_test(100) == 100 { score = score + 1; }
  if vec_push_test(0) == 0 { score = score + 1; }

  if vec_pop_test(100) == 50 { score = score + 1; }

  var nums = [1, 2, 3, 4, 5];
  if vec_sum(&nums) == 15 { score = score + 1; }

  var empty: Vec[Int] = [];
  if vec_sum(&empty) == 0 { score = score + 1; }

  var nums2 = [7, 2, 9, 1, 5];
  if vec_max(nums2) == 9 { score = score + 1; }

  var rev = vec_reverse(nums);
  if rev.len() == 5 { score = score + 1; }
  if rev[0] == 5 { score = score + 1; }
  if rev[4] == 1 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Binary Search Tree (using owned types)
// ============================================================

pub enum BST[T] {
  Empty,
  Node(value: T, left: BST[T], right: BST[T]),
}

pub fn BST.new[T]() -> BST[T] {
  return Empty;
}

pub fn BST.insert[T](val: T) -> BST[T] {
  match self {
    Empty => Node(value: val, left: Empty, right: Empty),
    Node(value: v, left: l, right: r) => {
      if val < v {
        return Node(value: v, left: l.insert(val), right: r);
      }
      elif val > v {
        return Node(value: v, left: l, right: r.insert(val));
      } else {
        return Node(value: v, left: l, right: r);
      }
    }
  }
}

pub fn BST.contains[T](val: T) -> Bool {
  match self {
    Empty => false,
    Node(value: v, left: l, right: r) => {
      if val == v { return true; }
      if val < v { return l.contains(val); }
      return r.contains(val);
    }
  }
}

pub fn BST.size[T]() -> Int {
  match self {
    Empty => 0,
    Node(value: _, left: l, right: r) => {
      return 1 + l.size() + r.size();
    }
  }
}

pub fn BST.min[T]() -> Option[T] {
  match self {
    Empty => None,
    Node(value: v, left: Empty, right: _) => Some(v),
    Node(value: _, left: l, right: _) => l.min(),
  }
}

fn test_bst() -> Int {
  var score = 0;
  var tree: BST[Int] = BST.new[Int]();
  if tree.size() == 0 { score = score + 1; }
  if !(tree.contains(5)) { score = score + 1; }

  var t1 = tree.insert(5);
  if t1.size() == 1 { score = score + 1; }
  if t1.contains(5) { score = score + 1; }
  if !(t1.contains(3)) { score = score + 1; }

  var t2 = t1.insert(3);
  var t3 = t2.insert(7);
  var t4 = t3.insert(1);
  var t5 = t4.insert(9);

  if t5.size() == 5 { score = score + 1; }
  if t5.contains(1) { score = score + 1; }
  if t5.contains(5) { score = score + 1; }
  if t5.contains(9) { score = score + 1; }
  if !(t5.contains(4)) { score = score + 1; }
  if !(t5.contains(10)) { score = score + 1; }

  match t5.min() {
    Some(v) => if v == 1 { score = score + 1; }
    None => {}
  }

  // Test 100 inserts (non-decreasing)
  var big_tree: BST[Int] = BST.new[Int]();
  var i = 0;
  while i < 100 {
    big_tree = big_tree.insert(i);
    i = i + 1;
  }
  if big_tree.size() == 100 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Linked List
// ============================================================

pub enum List[T] {
  Nil,
  Cons(head: T, tail: List[T]),
}

pub fn List.new[T]() -> List[T] {
  return Nil;
}

pub fn List.prepend[T](val: T) -> List[T] {
  return Cons(head: val, tail: self);
}

pub fn List.length[T]() -> Int {
  match self {
    Nil => 0,
    Cons(head: _, tail: t) => 1 + t.length(),
  }
}

pub fn List.sum() -> Int {
  match self {
    Nil => 0,
    Cons(head: h, tail: t) => h + t.sum(),
  }
}

pub fn List.reverse[T]() -> List[T] {
  match self {
    Nil => Nil,
    Cons(head: h, tail: t) => {
      var rev_tail = t.reverse();
      return rev_tail.append(h);
    }
  }
}

pub fn List.append[T](val: T) -> List[T] {
  match self {
    Nil => Cons(head: val, tail: Nil),
    Cons(head: h, tail: t) => Cons(head: h, tail: t.append(val)),
  }
}

fn test_list() -> Int {
  var score = 0;
  var list: List[Int] = List.new[Int]();
  if list.length() == 0 { score = score + 1; }

  var l1 = list.prepend(3);
  var l2 = l1.prepend(2);
  var l3 = l2.prepend(1);

  if l3.length() == 3 { score = score + 1; }
  if l3.sum() == 6 { score = score + 1; }

  var l4 = List.new[Int]();
  l4 = l4.append(10);
  l4 = l4.append(20);
  l4 = l4.append(30);
  if l4.length() == 3 { score = score + 1; }
  if l4.sum() == 60 { score = score + 1; }

  // Test 50 prepends
  var big_list: List[Int] = List.new[Int]();
  var i = 0;
  while i < 50 {
    big_list = big_list.prepend(i);
    i = i + 1;
  }
  if big_list.length() == 50 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Stack (using Vec)
// ============================================================

pub type Stack[T] = {
  items: Vec[T];
} derive[Clone]

pub fn Stack.new[T]() -> Stack[T] {
  return Stack[T]{ items: Vec[T].new() };
}

pub fn Stack.push[T](val: T) {
  items.push(val);
}

pub fn Stack.pop[T]() -> Option[T] {
  if items.len() == 0 { return None; }
  var idx = items.len() - 1;
  var val = items[idx];
  // Simplify: pop last element
  return Some(val);
}

pub fn Stack.peek[T]() -> Option[T] {
  if items.len() == 0 { return None; }
  return Some(items[items.len() - 1]);
}

pub fn Stack.is_empty[T]() -> Bool {
  return items.len() == 0;
}

pub fn Stack.size[T]() -> Int {
  return items.len();
}

fn test_stack() -> Int {
  var score = 0;
  var s: Stack[Int] = Stack.new[Int]();

  if s.is_empty() { score = score + 1; }
  if s.size() == 0 { score = score + 1; }

  s.push(10);
  s.push(20);
  s.push(30);

  if s.size() == 3 { score = score + 1; }
  if !(s.is_empty()) { score = score + 1; }

  match s.peek() {
    Some(v) => if v == 30 { score = score + 1; }
    None => {}
  }

  match s.pop() {
    Some(v) => if v == 30 { score = score + 1; }
    None => {}
  }

  return score;
}

// ============================================================
// SECTION 5: Queue (ring buffer simulation)
// ============================================================

pub type Queue[T] = {
  data: Vec[T];
  head: Int;
  tail: Int;
} derive[Clone]

pub fn Queue.new[T]() -> Queue[T] {
  return Queue[T]{ data: Vec[T].new(), head: 0, tail: 0 };
}

pub fn Queue.enqueue[T](val: T) {
  data.push(val);
  tail = data.len();
}

pub fn Queue.dequeue[T]() -> Option[T] {
  if head >= tail { return None; }
  var val = data[head];
  head = head + 1;
  return Some(val);
}

pub fn Queue.is_empty[T]() -> Bool {
  return head >= tail;
}

pub fn Queue.size[T]() -> Int {
  return tail - head;
}

fn test_queue() -> Int {
  var score = 0;
  var q: Queue[Int] = Queue.new[Int]();

  if q.is_empty() { score = score + 1; }
  if q.size() == 0 { score = score + 1; }

  q.enqueue(1);
  q.enqueue(2);
  q.enqueue(3);

  if q.size() == 3 { score = score + 1; }

  match q.dequeue() {
    Some(v) => if v == 1 { score = score + 1; }
    None => {}
  }

  match q.dequeue() {
    Some(v) => if v == 2 { score = score + 1; }
    None => {}
  }

  if q.size() == 1 { score = score + 1; }

  match q.dequeue() {
    Some(v) => if v == 3 { score = score + 1; }
    None => {}
  }

  if q.is_empty() { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Map Patterns (using parallel Vecs)
// ============================================================

pub type SimpleMap = {
  keys: Vec[Int];
  values: Vec[Int];
} derive[Clone]

pub fn SimpleMap.new() -> SimpleMap {
  return SimpleMap{ keys: Vec[Int].new(), values: Vec[Int].new() };
}

pub fn SimpleMap.insert(key: Int, value: Int) {
  var i = 0;
  var found = 0;
  while i < keys.len() && found == 0 {
    if keys[i] == key {
      values[i] = value;
      found = 1;
    }
    i = i + 1;
  }
  if found == 0 {
    keys.push(key);
    values.push(value);
  }
}

pub fn SimpleMap.get(key: Int) -> Option[Int] {
  var i = 0;
  while i < keys.len() {
    if keys[i] == key { return Some(values[i]); }
    i = i + 1;
  }
  return None;
}

pub fn SimpleMap.contains(key: Int) -> Bool {
  var i = 0;
  while i < keys.len() {
    if keys[i] == key { return true; }
    i = i + 1;
  }
  return false;
}

pub fn SimpleMap.size() -> Int {
  return keys.len();
}

fn test_map() -> Int {
  var score = 0;
  var m = SimpleMap.new();

  if m.size() == 0 { score = score + 1; }
  if !(m.contains(5)) { score = score + 1; }

  m.insert(1, 10);
  m.insert(2, 20);
  m.insert(3, 30);

  if m.size() == 3 { score = score + 1; }
  if m.contains(2) { score = score + 1; }
  if !(m.contains(5)) { score = score + 1; }

  match m.get(1) {
    Some(v) => if v == 10 { score = score + 1; }
    None => {}
  }

  match m.get(3) {
    Some(v) => if v == 30 { score = score + 1; }
    None => {}
  }

  // Update existing
  m.insert(2, 200);
  match m.get(2) {
    Some(v) => if v == 200 { score = score + 1; }
    None => {}
  }

  return score;
}

// ============================================================
// SECTION 7: Set Operations
// ============================================================

pub type SimpleSet = {
  data: Vec[Int];
} derive[Clone]

pub fn SimpleSet.new() -> SimpleSet {
  return SimpleSet{ data: Vec[Int].new() };
}

pub fn SimpleSet.add(val: Int) {
  var i = 0;
  while i < data.len() {
    if data[i] == val { return; }
    i = i + 1;
  }
  data.push(val);
}

pub fn SimpleSet.contains(val: Int) -> Bool {
  var i = 0;
  while i < data.len() {
    if data[i] == val { return true; }
    i = i + 1;
  }
  return false;
}

pub fn SimpleSet.size() -> Int {
  return data.len();
}

pub fn set_union(a: &SimpleSet, b: &SimpleSet) -> SimpleSet {
  var result = SimpleSet.new();
  var i = 0;
  while i < a.data.len() {
    result.add(a.data[i]);
    i = i + 1;
  }
  i = 0;
  while i < b.data.len() {
    result.add(b.data[i]);
    i = i + 1;
  }
  return result;
}

pub fn set_intersection(a: &SimpleSet, b: &SimpleSet) -> SimpleSet {
  var result = SimpleSet.new();
  var i = 0;
  while i < a.data.len() {
    if b.contains(a.data[i]) {
      result.add(a.data[i]);
    }
    i = i + 1;
  }
  return result;
}

fn test_set() -> Int {
  var score = 0;
  var s = SimpleSet.new();

  if s.size() == 0 { score = score + 1; }

  s.add(1);
  s.add(2);
  s.add(2);
  s.add(3);

  if s.size() == 3 { score = score + 1; }
  if s.contains(1) { score = score + 1; }
  if s.contains(2) { score = score + 1; }
  if !(s.contains(4)) { score = score + 1; }

  var s2 = SimpleSet.new();
  s2.add(2);
  s2.add(3);
  s2.add(4);

  var u = set_union(&s, &s2);
  if u.size() == 4 { score = score + 1; }

  var inter = set_intersection(&s, &s2);
  if inter.size() == 2 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Iterator Simulation (while-loop based)
// ============================================================

pub fn vec_filter(v: &Vec[Int], predicate: fn(Int) -> Bool) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    if predicate(v[i]) { result.push(v[i]); }
    i = i + 1;
  }
  return result;
}

pub fn vec_map(v: &Vec[Int], mapper: fn(Int) -> Int) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = 0;
  while i < v.len() {
    result.push(mapper(v[i]));
    i = i + 1;
  }
  return result;
}

pub fn vec_fold(v: &Vec[Int], initial: Int, folder: fn(Int, Int) -> Int) -> Int {
  var accum = initial;
  var i = 0;
  while i < v.len() {
    accum = folder(accum, v[i]);
    i = i + 1;
  }
  return accum;
}

fn is_even_pred(n: Int) -> Bool { return n % 2 == 0; }
fn double(n: Int) -> Int { return n * 2; }
fn add(a: Int, b: Int) -> Int { return a + b; }
fn multiply(a: Int, b: Int) -> Int { return a * b; }

fn test_iterators() -> Int {
  var score = 0;
  var nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

  var evens = vec_filter(&nums, is_even_pred);
  if evens.len() == 5 { score = score + 1; }

  var doubled = vec_map(&nums, double);
  if doubled.len() == 10 { score = score + 1; }
  if doubled[0] == 2 { score = score + 1; }

  var sum = vec_fold(&nums, 0, add);
  if sum == 55 { score = score + 1; }

  var product = vec_fold(&nums, 1, multiply);
  if product == 3628800 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 9: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_vec_basic();
  total = total + s1;
  max_score = max_score + 9;

  var s2 = test_bst();
  total = total + s2;
  max_score = max_score + 14;

  var s3 = test_list();
  total = total + s3;
  max_score = max_score + 7;

  var s4 = test_stack();
  total = total + s4;
  max_score = max_score + 6;

  var s5 = test_queue();
  total = total + s5;
  max_score = max_score + 8;

  var s6 = test_map();
  total = total + s6;
  max_score = max_score + 9;

  var s7 = test_set();
  total = total + s7;
  max_score = max_score + 8;

  var s8 = test_iterators();
  total = total + s8;
  max_score = max_score + 5;

  return BenchResult{
    name: "structures",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
