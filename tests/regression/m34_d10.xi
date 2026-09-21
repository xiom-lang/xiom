// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D10: Deep tree operations -- stress deep recursion with pointer tree (depth 10+)
type Node = { value: Int; left: *Node; right: *Node; }

fn tree_height(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  var hl: Int = tree_height(l);
  var hr: Int = tree_height(r);
  if hl > hr { return hl + 1; }
  return hr + 1;
}

fn tree_leaf_count(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  if l == (unsafe { 0 as *Node }) && r == (unsafe { 0 as *Node }) { return 1; }
  return tree_leaf_count(l) + tree_leaf_count(r);
}

fn tree_sum_all(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var l: *Node;
  var r: *Node;
  unsafe { val = (*n).value; }
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return val + tree_sum_all(l) + tree_sum_all(r);
}

fn tree_check_full(n: *Node, depth: Int) -> Bool {
  if depth == 0 { return n == (unsafe { 0 as *Node }); }
  if n == (unsafe { 0 as *Node }) { return false; }
  var l: *Node;
  var r: *Node;
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return tree_check_full(l, depth - 1) && tree_check_full(r, depth - 1);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if tree_height(empty) != 0 { return 1; }
  if tree_leaf_count(empty) != 0 { return 2; }
  if tree_sum_all(empty) != 0 { return 3; }
  if !tree_check_full(empty, 0) { return 4; }
  return 0;
}
