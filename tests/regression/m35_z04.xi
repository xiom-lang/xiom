// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z04: invariant+method+generic+match+while+if+contract+compound_assign+derive
type PosVal = { x: Int; invariant: x > 0; } derive[Eq]
enum Op { Inc, Dec, Set(v: Int) }
fn PosVal.add(self, other: PosVal) -> PosVal
  requires: self.x + other.x > 0
  ensures: result.x > 0
{ return PosVal{ x: self.x + other.x; }; }
fn adjust[T](p: PosVal, op: Op) -> PosVal
  requires: p.x > 0
  ensures: result.x > 0
{
  var c = p;
  match op { Inc => { c.x += 1; } Dec => { c.x -= 1; } Set(v) => { c.x = v; } }
  if c.x <= 0 { c.x = 1; }
  return c;
}
module guard {
  pub fn safe_adjust(p: PosVal, op: Op) -> PosVal { return adjust(p, op); }
  pub fn is_positive(p: PosVal) -> Bool { return p.x > 0; }
}
use guard.safe_adjust;
use guard.is_positive;
fn main() -> Int {
  var p1 = PosVal{ x: 5; };
  var p2 = PosVal{ x: 3; };
  var p3 = p1.add(p2);
  var cnt = 0;
  var r = safe_adjust(p3, Op.Inc);
  while cnt < 3 { r = safe_adjust(r, Op.Dec); cnt += 1; }
  r = safe_adjust(r, Op.Set(10));
  r = safe_adjust(r, Op.Inc);
  if is_positive(r) && r.x == 11 { return 0; }
  return 1;
}
