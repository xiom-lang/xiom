// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y01: while with break + generic enum + compound assign + module
enum Status { Active, Paused, Stopped }
type Cell = { val: Int; stat: Status; }
fn process[T](c: Cell, n: Int) -> Int
  requires: n >= 0
  ensures: result >= 0
{
  var i = 0;
  var acc = c.val;
  while i < n {
    if c.stat == Status.Active { acc = acc + i; }
    else { if c.stat == Status.Paused { acc = acc * 2; } else { break; } }
    i = i + 1;
  }
  return acc;
}
module runner {
  pub fn run_cell(c: Cell, n: Int) -> Int { return process(c, n); }
  pub fn direct_val(c: Cell) -> Int { return c.val; }
}
use runner.run_cell;
use runner.direct_val;
interface Valuer { fn value(self) -> Int; }
impl Valuer for Cell {
  fn value(self) -> Int { return self.val; }
}
fn main() -> Int {
  var c = Cell{ val: 5; stat: Status.Active; };
  var r1 = run_cell(c, 3);
  if r1 == 8 { return 0; }
  return 1;
}
