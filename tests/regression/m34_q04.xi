// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q04: Contract + method -- impl methods with contracts calling each other
type Counter = { val: Int; }

fn Counter.inc(self, by: Int) -> Int
  requires: self.val >= 0
  ensures: result > self.val
{
  return self.val + by;
}
fn Counter.double_inc(self, by: Int) -> Int
  requires: self.val >= 0
  ensures: result > self.val + by
{
  var first = self.inc(by);
  return first + by;
}
fn Counter.triple(self, by: Int) -> Int
  requires: self.val >= 0
  ensures: result > self.val
{
  return self.double_inc(by) + by;
}
fn main() -> Int {
  var c = Counter{ val: 10; };
  var r1: Int = c.inc(5);
  var c2 = Counter{ val: r1; };
  var r2: Int = c2.double_inc(3);
  var r3: Int = c.triple(2);
  if r1 == 15 && r2 == 21 && r3 == 16 { return 0; }
  return 1;
}
