// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Regression test: recursive enum with this-based methods
// The JSON ecosystem test uses JsonValue which has variants containing
// Vec[JsonValue] (self-referential). This tests whether recursive type
// definitions cause issues with this-based method dispatch.
module tests.ecosystem.test_recursive_enum_this

pub enum Node {
  Leaf(val: Int),
  Branch(left: Vec[Node], right: Vec[Node]),
}

fn Node.is_leaf() -> Bool {
  match this {
    Leaf(_) => { return true; }
    _ => { return false; }
  }
}

fn Node.is_branch() -> Bool {
  match this {
    Branch(_, _) => { return true; }
    _ => { return false; }
  }
}

fn Node.get_leaf_val() -> Int {
  match this {
    Leaf(val) => { return val; }
    _ => { return 0; }
  }
}

fn main() -> Int {
  var passed = 0; var total = 0;

  // Test 1: Leaf variant
  total = total + 1;
  var leaf = Node.Leaf(42);
  if Node.is_leaf(&leaf) { passed = passed + 1; }

  // Test 2: Leaf value
  total = total + 1;
  var leaf2 = Node.Leaf(99);
  if Node.get_leaf_val(&leaf2) == 99 { passed = passed + 1; }

  // Test 3: Leaf is not branch
  total = total + 1;
  var leaf3 = Node.Leaf(1);
  if !Node.is_branch(&leaf3) { passed = passed + 1; }

  // Test 4: Multiple leafs
  total = total + 1;
  var a = Node.Leaf(10);
  var b = Node.Leaf(20);
  if Node.is_leaf(&a) && Node.is_leaf(&b) { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
