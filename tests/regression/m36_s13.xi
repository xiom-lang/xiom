// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S13: AST manipulation -- fold (reduce) over an expression tree
type FoldNode = { value: Int; kind: Int; }
type FoldResult = { total: Int; max_val: Int; min_val: Int; count: Int; }
fn make_fold_node(v: Int, k: Int) -> FoldNode {
  return FoldNode{ value: v; kind: k; };
}
fn init_fold_result() -> FoldResult {
  return FoldResult{ total: 0; max_val: -9999; min_val: 9999; count: 0; };
}
fn fold_accumulate(res: FoldResult, node: FoldNode) -> FoldResult {
  var new_total = res.total + node.value;
  var new_count = res.count + 1;
  var new_max = res.max_val;
  var new_min = res.min_val;
  if node.value > res.max_val { new_max = node.value; }
  if node.value < res.min_val { new_min = node.value; }
  return FoldResult{ total: new_total; max_val: new_max; min_val: new_min; count: new_count; };
}
fn fold_average(res: FoldResult) -> Int {
  if res.count == 0 { return 0; }
  return res.total / res.count;
}
fn fold_range(res: FoldResult) -> Int {
  return res.max_val - res.min_val;
}
fn main() -> Int {
  var r = init_fold_result();
  var n1 = make_fold_node(10, 1);
  var n2 = make_fold_node(20, 1);
  var n3 = make_fold_node(5, 1);
  var n4 = make_fold_node(25, 1);
  var n5 = make_fold_node(15, 1);
  r = fold_accumulate(r, n1);
  r = fold_accumulate(r, n2);
  r = fold_accumulate(r, n3);
  r = fold_accumulate(r, n4);
  r = fold_accumulate(r, n5);
  if r.count != 5 { return 1; }
  if r.total != 75 { return 2; }
  if r.max_val != 25 { return 3; }
  if r.min_val != 5 { return 4; }
  if fold_average(r) != 15 { return 5; }
  if fold_range(r) != 20 { return 6; }
  return 0;
}
