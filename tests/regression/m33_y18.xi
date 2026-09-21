// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y18: generic constraint enum + struct derive + match + multi-contract + impl + module + diff
type Range = { lo: Int; hi: Int; } derive[Eq]
enum BoundCheck { InRange, Below, Above }
fn classify[T](r: Range, val: Int) -> BoundCheck
  requires: r.lo <= r.hi
{
  if val < r.lo { return BoundCheck.Below; }
  elif val > r.hi { return BoundCheck.Above; }
  return BoundCheck.InRange;
}
fn check_to_int(b: BoundCheck) -> Int {
  match b { InRange => 1, Below => 0, Above => 2, }
}
interface RangeOps { fn width(self) -> Int; }
impl RangeOps for Range {
  fn width(self) -> Int { return self.hi - self.lo + 1; }
}
module check {
  pub fn do_classify(r: Range, v: Int) -> BoundCheck { return classify(r, v); }
  pub fn via_width(r: Range) -> Int { return r.width(); }
  pub fn do_check_int(b: BoundCheck) -> Int { return check_to_int(b); }
}
use check.do_classify;
use check.via_width;
use check.do_check_int;
enum Strategy { Classify, DirectCount }
fn test_range(s: Strategy, r: Range, val: Int) -> Int {
  match s {
    Classify => do_check_int(do_classify(r, val)),
    DirectCount => if val >= r.lo && val <= r.hi { 1 } else { 0 },
  }
}
fn main() -> Int {
  var r = Range{ lo: 10; hi: 20; };
  var r1 = test_range(Strategy.Classify, r, 15);
  var r2 = test_range(Strategy.DirectCount, r, 15);
  var r3 = test_range(Strategy.Classify, r, 5);
  var r4 = test_range(Strategy.DirectCount, r, 5);
  if r1 == r2 && r3 == r4 && r1 == 1 && r3 == 0 { return 0; }
  return 1;
}
