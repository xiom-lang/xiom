// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z23: fn_pointer+generic+struct+enum+contract+module+match+derive+compound_assign+const
type Pair = { a: Int; b: Int; } derive[Eq]
const FACTOR: Int = 3;
enum Merge { Sum, Prod, Diff, Avg }
fn merge_pair[T](p: Pair, m: Merge) -> Int
  requires: p.a >= 0
  requires: p.b >= 0
  ensures: result >= 0
{
  match m { Sum => p.a + p.b, Prod => p.a * p.b, Diff => { if p.a >= p.b { return p.a - p.b; } return p.b - p.a; }, Avg => (p.a + p.b) / 2 }
}
fn apply_binop(f: fn(Int, Int) -> Int, x: Int, y: Int) -> Int { return f(x, y); }
fn do_sum(x: Int, y: Int) -> Int { return x + y; }
fn do_max(x: Int, y: Int) -> Int { if x >= y { return x; } return y; }
module math_ops {
  pub fn merge(p: Pair, m: Merge) -> Int { return merge_pair(p, m); }
  pub fn binop(f: fn(Int, Int) -> Int, x: Int, y: Int) -> Int { return apply_binop(f, x, y); }
  pub fn factor() -> Int { return FACTOR; }
}
use math_ops.merge;
use math_ops.binop;
use math_ops.factor;
fn main() -> Int {
  var p = Pair{ a: 10; b: 4; };
  var s = merge(p, Merge.Sum);
  var d = merge(p, Merge.Diff);
  var bp = binop(do_sum, p.a, p.b);
  var fp = factor();
  var total = s + d;
  total *= fp;
  if bp == 14 && s == 14 && d == 6 && total == 60 { return 0; }
  return 1;
}
