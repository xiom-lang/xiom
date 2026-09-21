// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z01: struct+enum+match+closure+compound_assign+module+derive+contract+while+const
type Item = { id: Int; qty: Int; avg: Int; } derive[Eq]
const TAX: Int = 2;
enum Action { Buy, Sell, Hold }
fn apply[T](act: Action, base: Item) -> Item
  requires: base.qty >= 0
  ensures: result.qty >= 0
{
  var cpy = base;
  match act { Buy => { cpy.qty += 1; } Sell => { cpy.qty -= 1; } Hold => {} }
  if cpy.qty < 0 { cpy.qty = 0; }
  return cpy;
}
module trade {
  pub fn exec(a: Action, it: Item) -> Item { return apply(a, it); }
  pub fn tax_val(it: Item) -> Int { return it.avg * TAX; }
  pub fn summary(it: Item) -> Int { var f = |x, y| x * y; return f(it.qty, it.avg); }
}
use trade.exec;
use trade.tax_val;
use trade.summary;
fn main() -> Int {
  var i1 = Item{ id: 1; qty: 3; avg: 10; };
  var r1 = exec(Action.Buy, i1);
  var r2 = exec(Action.Sell, r1);
  var t = tax_val(r2);
  var s = summary(r2);
  var i = 0;
  var acc = 0;
  while i < r2.qty { acc += r2.avg; i += 1; }
  if r2.qty == 3 && t == 20 && s == 30 && acc == 30 { return 0; }
  return 1;
}
