// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D18: Recursive generic types -- Tree[T] and List[T] with type parameters
type TreeNode[T] = { value: T; left: *TreeNode[T]; right: *TreeNode[T]; }
type ListNode[T] = { value: T; next: *ListNode[T]; }

fn tree_check_depth(n: *TreeNode[Int], max: Int) -> Bool {
  if n == (unsafe { 0 as *TreeNode[Int] }) { return true; }
  if max <= 0 { return false; }
  var l: *TreeNode[Int];
  var r: *TreeNode[Int];
  unsafe { l = (*n).left; }
  unsafe { r = (*n).right; }
  return tree_check_depth(l, max - 1) && tree_check_depth(r, max - 1);
}

fn list_count_int(n: *ListNode[Int]) -> Int {
  if n == (unsafe { 0 as *ListNode[Int] }) { return 0; }
  var nx: *ListNode[Int];
  unsafe { nx = (*n).next; }
  return 1 + list_count_int(nx);
}

fn is_null_tree(n: *TreeNode[Int]) -> Bool {
  return n == (unsafe { 0 as *TreeNode[Int] });
}

fn is_null_list(n: *ListNode[Int]) -> Bool {
  return n == (unsafe { 0 as *ListNode[Int] });
}

fn main() -> Int {
  var tn: *TreeNode[Int] = unsafe { 0 as *TreeNode[Int] };
  var ln: *ListNode[Int] = unsafe { 0 as *ListNode[Int] };
  if !is_null_tree(tn) { return 1; }
  if !is_null_list(ln) { return 2; }
  if !tree_check_depth(tn, 0) { return 3; }
  if list_count_int(ln) != 0 { return 4; }
  return 0;
}
