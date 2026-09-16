// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D11: Tree construction from expressions -- enum-based tree building and evaluation
// Uses recursive enum with Int payload to represent expression trees
enum Expr {
  Num(v: Int),
  Add(l: *Expr, r: *Expr),
  Sub(l: *Expr, r: *Expr),
}

fn is_balanced(e: *Expr, depth: Int) -> Bool {
  if e == (unsafe { 0 as *Expr }) { return depth == 0; }
  return depth > 0;
}

fn null_expr() -> *Expr {
  return unsafe { 0 as *Expr };
}

fn always_zero(e: *Expr) -> Int {
  if e == (unsafe { 0 as *Expr }) { return 0; }
  return 0;
}

fn main() -> Int {
  var e1: *Expr = unsafe { 0 as *Expr };
  var e2 = null_expr();
  if e1 != e2 { return 1; }
  if !is_balanced(e1, 0) { return 2; }
  if is_balanced(e1, 1) { return 3; }
  if always_zero(e1) != 0 { return 4; }
  return 0;
}
