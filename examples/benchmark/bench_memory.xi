// XIOM — Memory/Ownership Stress Benchmark
// Exercises moves, clones, borrows, ownership patterns, and allocation pressure.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.memory

use benchmark.main.BenchResult;

// ============================================================
// SECTION 1: Move Semantics
// ============================================================

pub type Data = {
  id: Int;
  value: Int;
} derive[Clone]

pub fn Data.new(id: Int, value: Int) -> Data {
  return Data{ id: id, value: value };
}

pub fn consume_data(d: Data) -> Int {
  return d.id * 1000 + d.value;
}

pub fn borrow_data(d: &Data) -> Int {
  return d.id * 1000 + d.value;
}

fn test_moves() -> Int {
  var score = 0;
  var d1 = Data.new(1, 42);
  if d1.value == 42 { score = score + 1; }

  // Clone before move
  var d1_clone = d1.clone();
  if d1_clone.value == 42 { score = score + 1; }
  if d1_clone.id == 1 { score = score + 1; }

  // Move
  var result = consume_data(d1);
  if result == 1042 { score = score + 1; }
  // d1 is now invalid (moved) - d1_clone still valid
  if d1_clone.value == 42 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 2: Borrow Patterns
// ============================================================

pub type Container = {
  a: Int;
  b: Int;
  c: Int;
} derive[Clone]

pub fn Container.new(a: Int, b: Int, c: Int) -> Container {
  return Container{ a: a, b: b, c: c };
}

pub fn Container.sum() -> Int {
  return a + b + c;
}

pub fn Container.product() -> Int {
  return a * b * c;
}

pub fn Container.max_field() -> Int {
  var m = a;
  if b > m { m = b; }
  if c > m { m = c; }
  return m;
}

pub fn read_borrow_two(c1: &Container, c2: &Container) -> Int {
  return c1.sum() + c2.sum();
}

fn test_borrows() -> Int {
  var score = 0;
  var c1 = Container.new(1, 2, 3);
  var c2 = Container.new(4, 5, 6);

  if c1.sum() == 6 { score = score + 1; }
  if c1.product() == 6 { score = score + 1; }
  if c1.max_field() == 3 { score = score + 1; }

  if c2.sum() == 15 { score = score + 1; }

  // Multiple read borrows simultaneously
  var combined = read_borrow_two(&c1, &c2);
  if combined == 21 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 3: Clone Heavy Paths
// ============================================================

pub type BigData = {
  f0: Int; f1: Int; f2: Int; f3: Int; f4: Int;
  f5: Int; f6: Int; f7: Int; f8: Int; f9: Int;
} derive[Clone]

pub fn BigData.new(base: Int) -> BigData {
  return BigData{
    f0: base, f1: base + 1, f2: base + 2, f3: base + 3, f4: base + 4,
    f5: base + 5, f6: base + 6, f7: base + 7, f8: base + 8, f9: base + 9,
  };
}

pub fn BigData.sum() -> Int {
  return f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8 + f9;
}

pub fn clone_chain(original: &BigData, depth: Int) -> BigData {
  var current = original.clone();
  var i = 0;
  while i < depth {
    current = current.clone();
    i = i + 1;
  }
  return current;
}

fn test_clone_heavy() -> Int {
  var score = 0;
  var bd = BigData.new(0);
  if bd.f0 == 0 { score = score + 1; }
  if bd.f9 == 9 { score = score + 1; }
  if bd.sum() == 45 { score = score + 1; }

  var cloned = clone_chain(&bd, 5);
  if cloned.f0 == 0 { score = score + 1; }
  if cloned.sum() == 45 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 4: Allocation Stress — Vec Push/Pop
// ============================================================

pub fn alloc_stress_vec(count: Int) -> Int {
  var v = Vec[Int].new();
  var i = 0;
  while i < count {
    v.push(i);
    i = i + 1;
  }
  var total = 0;
  i = 0;
  while i < v.len() {
    total = total + v[i];
    i = i + 1;
  }
  return total;
}

pub fn alloc_stress_nested(count: Int) -> Int {
  var outer = Vec[Vec[Int]].new();
  var i = 0;
  while i < count {
    var inner = Vec[Int].new();
    var j = 0;
    while j < i + 1 {
      inner.push(j);
      j = j + 1;
    }
    outer.push(inner);
    i = i + 1;
  }
  var total = 0;
  i = 0;
  while i < outer.len() {
    total = total + outer[i].len();
    i = i + 1;
  }
  return total;
}

fn test_allocation() -> Int {
  var score = 0;
  var sum1 = alloc_stress_vec(50);
  if sum1 == 1225 { score = score + 1; }

  var sum2 = alloc_stress_nested(10);
  if sum2 == 55 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 5: Move Chains
// ============================================================

pub fn create_and_pass() -> Int {
  var d = Data.new(5, 10);
  return consume_data(d);
}

pub fn chain_of_three(a: Data, b: Data, c: Data) -> Int {
  return a.value + b.value + c.value;
}

fn test_move_chains() -> Int {
  var score = 0;
  if create_and_pass() == 5010 { score = score + 1; }

  var result = chain_of_three(
    Data.new(1, 10),
    Data.new(2, 20),
    Data.new(3, 30),
  );
  if result == 60 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 6: Struct Ownership Patterns
// ============================================================

pub type OwnedColl = {
  items: Vec[Data];
} derive[Clone]

pub fn OwnedColl.new() -> OwnedColl {
  return OwnedColl{ items: Vec[Data].new() };
}

pub fn OwnedColl.add(val: Int) {
  items.push(Data.new(items.len(), val));
}

pub fn OwnedColl.sum() -> Int {
  var total = 0;
  var i = 0;
  while i < items.len() {
    total = total + items[i].value;
    i = i + 1;
  }
  return total;
}

pub fn OwnedColl.count() -> Int {
  return items.len();
}

fn test_struct_ownership() -> Int {
  var score = 0;
  var coll = OwnedColl.new();
  if coll.count() == 0 { score = score + 1; }

  coll.add(10);
  coll.add(20);
  coll.add(30);

  if coll.count() == 3 { score = score + 1; }
  if coll.sum() == 60 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 7: Recursive Ownership Walks
// ============================================================

pub type Node = {
  value: Int;
  children: Vec[Node];
} derive[Clone]

pub fn Node.new(val: Int) -> Node {
  return Node{ value: val, children: Vec[Node].new() };
}

pub fn Node.add_child(child: Node) {
  children.push(child);
}

pub fn Node.sum_tree() -> Int {
  var total = value;
  var i = 0;
  while i < children.len() {
    total = total + children[i].sum_tree();
    i = i + 1;
  }
  return total;
}

pub fn Node.count_nodes() -> Int {
  var count = 1;
  var i = 0;
  while i < children.len() {
    count = count + children[i].count_nodes();
    i = i + 1;
  }
  return count;
}

fn test_tree_ownership() -> Int {
  var score = 0;
  var root = Node.new(1);

  var child1 = Node.new(2);
  var child2 = Node.new(3);

  var grandchild = Node.new(4);
  child1.add_child(grandchild);

  root.add_child(child1);
  root.add_child(child2);

  if root.sum_tree() == 10 { score = score + 1; }
  if root.count_nodes() == 4 { score = score + 1; }

  return score;
}

// ============================================================
// SECTION 8: Aggregate Runner
// ============================================================

pub fn run_all() -> BenchResult {
  var total = 0;
  var max_score = 0;

  var s1 = test_moves();
  total = total + s1;
  max_score = max_score + 5;

  var s2 = test_borrows();
  total = total + s2;
  max_score = max_score + 6;

  var s3 = test_clone_heavy();
  total = total + s3;
  max_score = max_score + 5;

  var s4 = test_allocation();
  total = total + s4;
  max_score = max_score + 2;

  var s5 = test_move_chains();
  total = total + s5;
  max_score = max_score + 2;

  var s6 = test_struct_ownership();
  total = total + s6;
  max_score = max_score + 4;

  var s7 = test_tree_ownership();
  total = total + s7;
  max_score = max_score + 2;

  return BenchResult{
    name: "memory",
    score: total,
    max_score: max_score,
    passed: total == max_score,
    elapsed_ms: 0,
  };
}
