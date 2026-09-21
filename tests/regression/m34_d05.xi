// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D05: List traversal -- multiple recursive traversals on linked list
type Node = { value: Int; next: *Node; }

fn list_sum(n: *Node, acc: Int) -> Int {
  if n == (unsafe { 0 as *Node }) { return acc; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  return list_sum(nx, acc + val);
}

fn list_check(n: *Node, target: Int) -> Bool {
  if n == (unsafe { 0 as *Node }) { return target == 0; }
  var val: Int;
  var nx: *Node;
  unsafe { val = (*n).value; }
  unsafe { nx = (*n).next; }
  return list_check(nx, target - val);
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if list_sum(empty, 0) != 0 { return 1; }
  if !list_check(empty, 0) { return 2; }
  return 0;
}
