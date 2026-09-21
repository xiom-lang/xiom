// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D18: BST traversal -- inorder, preorder, postorder sum/sequence verification
type BNode = { value: Int; left: *BNode; right: *BNode; }

fn inorder_sum(n: *BNode) -> Int {
  if n == unsafe { 0 as *BNode } { return 0; }
  var v: Int;
  var l: *BNode;
  var r: *BNode;
  unsafe { v = (*n).value; l = (*n).left; r = (*n).right; }
  return inorder_sum(l) + v + inorder_sum(r);
}

fn preorder_sum(n: *BNode) -> Int {
  if n == unsafe { 0 as *BNode } { return 0; }
  var v: Int;
  var l: *BNode;
  var r: *BNode;
  unsafe { v = (*n).value; l = (*n).left; r = (*n).right; }
  return v + preorder_sum(l) + preorder_sum(r);
}

fn postorder_sum(n: *BNode) -> Int {
  if n == unsafe { 0 as *BNode } { return 0; }
  var v: Int;
  var l: *BNode;
  var r: *BNode;
  unsafe { v = (*n).value; l = (*n).left; r = (*n).right; }
  return postorder_sum(l) + postorder_sum(r) + v;
}

fn main() -> Int {
  var empty: *BNode = unsafe { 0 as *BNode };
  if inorder_sum(empty) != 0 { return 1; }
  if preorder_sum(empty) != 0 { return 2; }
  if postorder_sum(empty) != 0 { return 3; }
  return 0;
}
