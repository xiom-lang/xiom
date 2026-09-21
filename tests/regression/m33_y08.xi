// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y08: generic pair struct + enum dispatch + match + contract + impl method + module + diff
type Pair = { first: Int; second: Int; } derive[Eq]
enum TransformKind { Swap, AddFirst, MulBoth }
fn transform[T](p: Pair, k: TransformKind) -> Int
  requires: p.first >= 0
  requires: p.second >= 0
  ensures: result >= 0
{
  match k {
    Swap => p.second * 1000 + p.first,
    AddFirst => p.first + p.second,
    MulBoth => p.first * p.second,
  }
}
fn add_direct(p: Pair) -> Int { return p.first + p.second; }
interface Computable { fn compute(self) -> Int; }
impl Computable for Pair {
  fn compute(self) -> Int { return self.first + self.second; }
}
module pairlib {
  pub fn do_transform(p: Pair, k: TransformKind) -> Int { return transform(p, k); }
  pub fn do_add(p: Pair) -> Int { return add_direct(p); }
  pub fn via_trait(p: Pair) -> Int { return p.compute(); }
}
use pairlib.do_transform;
use pairlib.do_add;
use pairlib.via_trait;
enum CallMode { Trans, Direct, Trait }
fn call(mode: CallMode, p: Pair, k: TransformKind) -> Int {
  match mode { Trans => do_transform(p, k), Direct => do_add(p), Trait => via_trait(p), }
}
fn main() -> Int {
  var p = Pair{ first: 4; second: 9; };
  var r1 = call(CallMode.Trans, p, TransformKind.AddFirst);
  var r2 = call(CallMode.Direct, p, TransformKind.AddFirst);
  var r3 = call(CallMode.Trait, p, TransformKind.AddFirst);
  if r1 == r2 && r2 == r3 && r1 == 13 { return 0; }
  return 1;
}
