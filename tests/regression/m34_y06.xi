// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y06: while with nested if containing match + compound assign + module + contract
enum Signal { High, Low, Off }
type Meter = { level: Int; sig: Signal; }
fn meter_loop[T](m: Meter) -> Int
  requires: m.level >= 0
  ensures: result >= 0
{
  var acc = m.level;
  var cycles = 0;
  while cycles < 5 {
    if m.sig == Signal.High {
      acc = acc + 3;
      cycles = cycles + 1;
    }
    else {
      match m.sig {
        Low => { acc = acc - 1; cycles = cycles + 1; }
        Off => { break; }
      }
    }
  }
  acc = acc * 2;
  return acc;
}
module signals {
  pub fn process_signal(m: Meter) -> Int { return meter_loop(m); }
  pub fn raw_level(m: Meter) -> Int { return m.level; }
}
use signals.process_signal;
use signals.raw_level;
fn main() -> Int {
  var m1 = Meter{ level: 10; sig: Signal.High; };
  var m2 = Meter{ level: 5; sig: Signal.Low; };
  var r1 = process_signal(m1);
  var r2 = process_signal(m2);
  if r1 == 50 && r2 == 0 { return 0; }
  return 1;
}
