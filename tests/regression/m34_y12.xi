// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y12: generic + while + if + compound assign + impl + enum + module
type Stats = { min: Int; max: Int; sum: Int; }
enum Aggregate { Min, Max, Avg }
fn min_of(a: Int, b: Int) -> Int { if a < b { return a; } return b; }
fn max_of(a: Int, b: Int) -> Int { if a > b { return a; } return b; }
fn avg_of(a: Int, b: Int, c: Int) -> Int { return (a + b + c) / 3; }
fn aggregate[T](a: Int, b: Int, c: Int, kind: Aggregate) -> Int
  ensures: result >= 0
{
  match kind {
    Min => min_of(min_of(a, b), c),
    Max => max_of(max_of(a, b), c),
    Avg => avg_of(a, b, c),
  }
}
interface Aggregator { fn agg(self, kind: Aggregate) -> Int; }
impl Aggregator for Stats {
  fn agg(self, kind: Aggregate) -> Int {
    match kind { Min => self.min, Max => self.max, Avg => self.sum / 3, }
  }
}
module stats_mod {
  pub fn compute(a: Int, b: Int, c: Int, k: Aggregate) -> Int { return aggregate(a, b, c, k); }
  pub fn via_iface(s: Stats, k: Aggregate) -> Int { return s.agg(k); }
}
use stats_mod.compute;
use stats_mod.via_iface;
fn main() -> Int {
  var s = Stats{ min: 1; max: 9; sum: 12; };
  var r1 = compute(7, 2, 9, Aggregate.Min);
  var r2 = compute(7, 2, 9, Aggregate.Max);
  var r3 = via_iface(s, Aggregate.Avg);
  if r1 == 2 && r2 == 9 && r3 == 4 { return 0; }
  return 1;
}
