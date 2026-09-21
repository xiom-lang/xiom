// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D09: Recursive enum -- enum variant contains pointer to its own type (Expr = Num | Add | Mul)
// Tests: recursive enum type definition, match destructuring, null-pointer recursion
enum Expr {
  Num(v: Int),
  Add(l: *Expr, r: *Expr),
  Mul(l: *Expr, r: *Expr),
}

fn count_nodes(e: *Expr) -> Int {
  if e == (unsafe { 0 as *Expr }) { return 0; }
  return 1;
}

fn check_null(e: *Expr) -> Bool {
  return e == (unsafe { 0 as *Expr });
}

fn depth_estimate(e: *Expr, acc: Int) -> Int {
  if e == (unsafe { 0 as *Expr }) { return acc; }
  return acc + 1;
}

fn main() -> Int {
  var n: *Expr = unsafe { 0 as *Expr };
  if count_nodes(n) != 0 { return 1; }
  if !check_null(n) { return 2; }
  if depth_estimate(n, 0) != 0 { return 3; }
  return 0;
}
