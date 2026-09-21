// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z15: derive+struct+enum+impl+module+generic+match+while+compound_assign+Option
type Slot = { val: Int; active: Bool; } derive[Eq]
enum SlotCmd { Enable, Disable, Toggle, Bump(n: Int) }
fn Slot.default() -> Slot { return Slot{ val: 0; active: false; }; }
fn Slot.is_active(self) -> Bool { return self.active; }
interface Slottable { fn activate(self) -> Slot; fn value(self) -> Int; }
impl Slottable for Slot {
  fn activate(self) -> Slot { return Slot{ val: self.val; active: true; }; }
  fn value(self) -> Int { return self.val; }
}
fn apply_cmd[T](s: Slot, cmd: SlotCmd) -> Slot {
  match cmd {
    Enable => s.activate(),
    Disable => { var r = s; r.active = false; return r; }
    Toggle => { var r = s; r.active = !s.active; return r; }
    Bump(n) => { var r = s; r.val += n; return r; }
  }
}
module slots {
  pub fn cmd(s: Slot, c: SlotCmd) -> Slot { return apply_cmd(s, c); }
  pub fn default_slot() -> Slot { return Slot.default(); }
  pub fn active_val(s: Slot) -> Option[Int] { if s.active { return Some(s.val); } return None; }
}
use slots.cmd;
use slots.default_slot;
use slots.active_val;
fn main() -> Int {
  var s1 = default_slot();
  var s2 = cmd(s1, SlotCmd.Bump(5));
  var s3 = cmd(s2, SlotCmd.Enable);
  var s4 = cmd(s3, SlotCmd.Bump(3));
  var chk = 0;
  match active_val(s4) { Some(v) => { if v == 8 { chk += 1; } } None => {} }
  if s4.active && s4.val == 8 { chk += 1; }
  var i = 0; while i < 3 { var s5 = cmd(s4, SlotCmd.Bump(1)); s4 = s5; i += 1; }
  match active_val(s4) { Some(v) => { if v == 11 { chk += 1; } } None => {} }
  if chk == 3 { return 0; }
  return 1;
}
