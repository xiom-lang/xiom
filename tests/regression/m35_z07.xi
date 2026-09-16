// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z07: fn_pointer+generic+closure+match+compound_assign+derive+module+Option+contract
type BinOp = { op_a: Int; op_b: Int; } derive[Eq]
enum Calc { Add, Mul, Div, Max }
fn compute[T](c: Calc, a: Int, b: Int) -> Int
  requires: b != 0 || c != Calc.Div
  ensures: result >= 0
{
  match c {
    Add => a + b,
    Mul => a * b,
    Div => a / b,
    Max => { if a >= b { return a; } return b; },
  }
}
fn apply_op(f: fn(Int, Int) -> Int, a: Int, b: Int) -> Int { return f(a, b); }
fn do_add(x: Int, y: Int) -> Int { return x + y; }
fn do_mul(x: Int, y: Int) -> Int { return x * y; }
module engine {
  pub fn calc(c: Calc, a: Int, b: Int) -> Int { return compute(c, a, b); }
  pub fn dispatch(f: fn(Int, Int) -> Int, a: Int, b: Int) -> Int { return apply_op(f, a, b); }
}
use engine.calc;
use engine.dispatch;
fn main() -> Int {
  var bp = BinOp{ op_a: 4; op_b: 2; };
  var r1 = calc(Calc.Add, bp.op_a, bp.op_b);
  var r2 = calc(Calc.Mul, bp.op_a, bp.op_b);
  var r3 = dispatch(do_add, r1, r2);
  var cl = |x| x + bp.op_a;
  var r4 = cl(1);
  var chk = 0;
  if r1 == 6 { chk += 1; }
  if r2 == 8 { chk += 1; }
  if r3 == 14 { chk += 1; }
  if r4 == 5 { chk += 1; }
  if chk == 4 { return 0; }
  return 1;
}
