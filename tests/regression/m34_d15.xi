// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D15: List filter -- traverse linked list and count nodes matching predicates
type Node = { value: Int; next: *Node; }

fn count_positive(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  var inc: Int = 0;
  if val > 0 { inc = 1; }
  return inc + count_positive(nx);
}

fn count_negative(n: *Node) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  var inc: Int = 0;
  if val < 0 { inc = 1; }
  return inc + count_negative(nx);
}

fn count_divisible_by(n: *Node, d: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  if d == 0 { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  var inc: Int = 0;
  if val % d == 0 { inc = 1; }
  return inc + count_divisible_by(nx, d);
}

fn sum_greater_than(n: *Node, threshold: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  var add: Int = 0;
  if val > threshold { add = val; }
  return add + sum_greater_than(nx, threshold);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if count_positive(empty) != 0 { return 1; }
  if count_negative(empty) != 0 { return 2; }
  if count_divisible_by(empty, 3) != 0 { return 3; }
  if sum_greater_than(empty, 0) != 0 { return 4; }
  return 0;
}
