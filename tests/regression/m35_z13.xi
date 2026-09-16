// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z13: contract+invariant+generic+match+struct+module+derive+compound_assign+closure
type Bounded = { val: Int; lo: Int; hi: Int; invariant: lo <= hi; invariant: val >= lo; invariant: val <= hi; } derive[Eq]
enum Adjust { Up, Down, Set(v: Int) }
fn clamp[T](b: Bounded, a: Adjust) -> Bounded
  requires: a != Adjust.Set(-1)
  ensures: result.val >= result.lo
  ensures: result.val <= result.hi
{
  var r = b;
  match a { Up => { r.val += 1; if r.val > r.hi { r.val = r.hi; } } Down => { r.val -= 1; if r.val < r.lo { r.val = r.lo; } } Set(v) => { r.val = v; } }
  return r;
}
module bounds {
  pub fn adjust(b: Bounded, a: Adjust) -> Bounded { return clamp(b, a); }
  pub fn in_range(b: Bounded) -> Bool { return b.val >= b.lo && b.val <= b.hi; }
  pub fn mid(b: Bounded) -> Int { return (b.lo + b.hi) / 2; }
}
use bounds.adjust;
use bounds.in_range;
use bounds.mid;
fn main() -> Int {
  var b = Bounded{ val: 5; lo: 0; hi: 10; };
  var b1 = adjust(b, Adjust.Up);
  var b2 = adjust(b1, Adjust.Down);
  var b3 = adjust(b2, Adjust.Set(7));
  var chk = 0;
  if in_range(b1) && b1.val == 6 { chk += 1; }
  if in_range(b2) && b2.val == 5 { chk += 1; }
  if in_range(b3) && b3.val == 7 { chk += 1; }
  var f = |x| x + mid(b3);
  if f(0) == 5 { chk += 1; }
  if chk == 4 { return 0; }
  return 1;
}
