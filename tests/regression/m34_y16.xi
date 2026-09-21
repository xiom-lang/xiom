// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y16: while-break + Option + match + generic + compound assign + module
type Counter = { val: Int; limit: Int; }
enum Tick { Up, Down, Buzz }
fn wrap_opt(x: Int) -> Option[Int] { if x >= 0 { return Some(x); } else { return None; } }
fn tick[T](c: Counter, kind: Tick) -> Option[Int]
  requires: c.val >= 0
  requires: c.limit > 0
{
  var v = c.val;
  var n = 0;
  while n < c.limit {
    match kind {
      Up => { v = v + 1; }
      Down => {
        if v > 0 { v = v - 1; }
        else { break; }
      }
      Buzz => {
        v = v * 2;
        if v > 1000 { return None; }
      }
    }
    n = n + 1;
  }
  return wrap_opt(v);
}
module tick_mod {
  pub fn run_tick(c: Counter, k: Tick) -> Option[Int] { return tick(c, k); }
  pub fn raw_val(c: Counter) -> Int { return c.val; }
}
use tick_mod.run_tick;
use tick_mod.raw_val;
fn main() -> Int {
  var c1 = Counter{ val: 5; limit: 3; };
  var c2 = Counter{ val: 2; limit: 4; };
  match run_tick(c1, Tick.Up) {
    Some(v) => { if v != 8 { return 1; } }
    None => { return 2; }
  }
  match run_tick(c2, Tick.Down) {
    Some(v) => { if v != 0 { return 3; } }
    None => { return 4; }
  }
  return 0;
}
