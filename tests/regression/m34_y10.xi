// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y10: while + if + compound assign + Option + match + derive + module + impl
type Counter = { val: Int; delta: Int; } derive[Eq]
enum Step { Increment, Decrement, Double, Half }
fn step(s: Step, v: Int) -> Option[Int] {
  match s {
    Increment => Some(v + 1),
    Decrement => { if v > 0 { return Some(v - 1); } else { return None; } }
    Double => Some(v * 2),
    Half => Some(v / 2),
  }
}
fn run_loop[T](c: Counter, n: Int) -> Int
  requires: c.val >= 0
  ensures: result >= 0
{
  var v = c.val;
  var i = 0;
  while i < n { v = v + c.delta; i = i + 1; }
  return v;
}
interface CounterOps { fn add_delta(self) -> Int; }
impl CounterOps for Counter {
  fn add_delta(self) -> Int { return self.val + self.delta; }
}
module stepper {
  pub fn do_step(s: Step, v: Int) -> Option[Int] { return step(s, v); }
  pub fn do_loop(c: Counter, n: Int) -> Int { return run_loop(c, n); }
  pub fn via_iface(c: Counter) -> Int { return c.add_delta(); }
}
use stepper.do_step;
use stepper.do_loop;
use stepper.via_iface;
fn main() -> Int {
  match do_step(Step.Increment, 10) {
    Some(v) => { if v != 11 { return 1; } }
    None => { return 2; }
  }
  match do_step(Step.Decrement, 0) {
    Some(_) => { return 3; }
    None => {}
  }
  var c1 = Counter{ val: 5; delta: 3; };
  var r = do_loop(c1, 4);
  if r != 17 { return 4; }
  var c2 = Counter{ val: 5; delta: 3; };
  var r2 = via_iface(c2);
  if r2 != 8 { return 5; }
  return 0;
}
