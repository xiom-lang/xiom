// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z08: closure+enum+match+compound_assign+while+contract+derive+module+cast
type Container = { val: Int; tag: Int; } derive[Eq]
enum Mode { Normal, Boost(m: Int), Drain }
fn process[T](c: Container, m: Mode) -> Int
  requires: c.val >= 0
  ensures: result >= 0
{
  var v = c.val;
  match m { Normal => { v += c.tag; } Boost(mult) => { v *= mult; } Drain => { v = 0; } }
  var v2 = v + 1;
  var v3 = v2 * 2;
  if v3 > 100 { v3 = 100; }
  return v3;
}
module nest {
  pub fn run(c: Container, m: Mode) -> Int { return process(c, m); }
  pub fn wrap(c: Container) -> Int { var f = |x| x + c.tag; return f(c.val); }
}
use nest.run;
use nest.wrap;
fn main() -> Int {
  var c = Container{ val: 10; tag: 5; };
  var r1 = run(c, Mode.Normal);
  var r2 = run(c, Mode.Boost(3));
  var r3 = wrap(c);
  var i = 0;
  var accum = 0;
  while i < 3 { accum += r1; i += 1; }
  if r1 == 32 && r2 == 62 && r3 == 15 && accum == 96 { return 0; }
  return 1;
}
