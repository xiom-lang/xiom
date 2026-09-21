// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S17: Optimization -- constant folding for arithmetic expressions
type ConstExpr = { op: Int; left_val: Int; right_val: Int; is_const: Bool; }
fn make_const_expr(op: Int, left: Int, right: Int, cnst: Bool) -> ConstExpr {
  return ConstExpr{ op: op; left_val: left; right_val: right; is_const: cnst; };
}
fn is_constant(e: ConstExpr) -> Bool {
  return e.is_const;
}
fn const_fold_add(e: ConstExpr) -> Int {
  if !e.is_const { return 0; }
  return e.left_val + e.right_val;
}
fn const_fold_sub(e: ConstExpr) -> Int {
  if !e.is_const { return 0; }
  return e.left_val - e.right_val;
}
fn const_fold_mul(e: ConstExpr) -> Int {
  if !e.is_const { return 0; }
  return e.left_val * e.right_val;
}
fn const_fold_div(e: ConstExpr) -> Int {
  if !e.is_const || e.right_val == 0 { return 0; }
  return e.left_val / e.right_val;
}
fn identity_fold(e: ConstExpr) -> Int {
  if e.op == 0 { return 0; }
  if e.op == 1 && e.right_val == 0 { return e.left_val; }
  if e.op == 3 && e.right_val == 1 { return e.left_val; }
  return -1;
}
fn main() -> Int {
  var e1 = make_const_expr(1, 10, 20, true);
  var e2 = make_const_expr(2, 30, 10, true);
  var e3 = make_const_expr(3, 5, 6, true);
  var e4 = make_const_expr(4, 100, 4, true);
  var e5 = make_const_expr(1, 50, 50, false);
  if !is_constant(e1) { return 1; }
  if is_constant(e5) { return 2; }
  if const_fold_add(e1) != 30 { return 3; }
  if const_fold_sub(e2) != 20 { return 4; }
  if const_fold_mul(e3) != 30 { return 5; }
  if const_fold_div(e4) != 25 { return 6; }
  if const_fold_add(e5) != 0 { return 7; }
  var id1 = make_const_expr(1, 42, 0, false);
  if identity_fold(id1) != 42 { return 8; }
  var id2 = make_const_expr(3, 7, 1, false);
  if identity_fold(id2) != 7 { return 9; }
  return 0;
}
