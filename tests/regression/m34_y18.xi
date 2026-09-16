// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y18: match + contract + Option + compound assign + impl + module
type Slot = { val: Int; flag: Bool; }
enum Action { Increment, Decrement, Toggle, Nop }
fn inc(x: Int) -> Int { var r = x; r = r + 1; return r; }
fn dec(x: Int) -> Int { var r = x; if r > 0 { r = r - 1; } return r; }
fn togg(x: Int, f: Bool) -> Int { if f { return x * 2; } else { return x / 2; } }
fn apply[T](s: Slot, act: Action) -> Int
  requires: s.val >= 0
{
  var v = s.val;
  match act {
    Increment => inc(v),
    Decrement => dec(v),
    Toggle => togg(v, s.flag),
    Nop => v,
  }
}
module slot_mod {
  pub fn do_apply(s: Slot, a: Action) -> Int { return apply(s, a); }
  pub fn raw_val(s: Slot) -> Int { return s.val; }
}
use slot_mod.do_apply;
use slot_mod.raw_val;
interface Actionable { fn act(self, a: Action) -> Int; }
impl Actionable for Slot {
  fn act(self, a: Action) -> Int { return apply(self, a); }
}
fn main() -> Int {
  var s1 = Slot{ val: 10; flag: true; };
  var s1b = Slot{ val: 10; flag: true; };
  var s2 = Slot{ val: 8; flag: false; };
  var r1 = do_apply(s1, Action.Increment);
  if r1 != 11 { return 1; }
  var r2 = do_apply(s1b, Action.Toggle);
  if r2 != 20 { return 2; }
  var r3 = s2.act(Action.Toggle);
  if r3 != 4 { return 3; }
  return 0;
}
