// m150 (item 3, exact arity): the flip must reject real arity mismatches
// without false positives on two shapes the flip surfaced:
//  - receiver-sugar calls to a receiver-style generic method whose first
//    param is a REF-qualified generic receiver (`fn Box.get[T](b: &Box[T])`):
//    `b.get()` is a valid 0-explicit-arg call (regression: smoke_core_box);
//  - generic bound methods (`T: Ord`) resolve through the interface
//    declaration, not a same-leaf singleton capture (regression: xiom.cmp's
//    `min`/`max`/`clamp` and xiom.collections).
module m150_exact_arity;

use xiom.core;

fn less[T: Ord](a: T, b: T) -> Bool {
  return a.compare(b) < 0;
}

fn main() -> Int {
  let b = Box[Int].new(42);
  let val = b.get();
  if *val != 42 { return 1; }
  if !less(1, 2) { return 2; }
  if less(2, 1) { return 3; }
  return 0;
}
