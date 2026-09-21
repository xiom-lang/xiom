// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y17: type alias + generic + enum payload + struct + match + contract + impl + module + diff
type Id = Int;
type Value = Int;
type Slot = { id: Id; value: Value; }
enum SlotKind { Active(s: Slot), Inactive(id: Id) }
fn extract[T](k: SlotKind) -> Int
  requires: true
  ensures: result >= 0
{
  match k {
    Active(s) => s.value,
    Inactive(id) => id,
  }
}
fn slot_val(s: Slot) -> Int { return s.value; }
interface SlotOps { fn val(self) -> Int; }
impl SlotOps for Slot {
  fn val(self) -> Int { return self.value; }
}
module lib {
  pub fn do_extract(k: SlotKind) -> Int { return extract(k); }
  pub fn do_slot_val(s: Slot) -> Int { return slot_val(s); }
  pub fn via_iface(s: Slot) -> Int { return s.val(); }
}
use lib.do_extract;
use lib.do_slot_val;
use lib.via_iface;
enum Route { Extract, Direct, Iface }
fn resolve(r: Route, k: SlotKind, s: Slot) -> Int {
  match r { Extract => do_extract(k), Direct => do_slot_val(s), Iface => via_iface(s), }
}
fn main() -> Int {
  var s = Slot{ id: 10; value: 55; };
  var k = SlotKind.Active(s);
  var r1 = resolve(Route.Extract, k, s);
  var r2 = resolve(Route.Direct, k, s);
  var r3 = resolve(Route.Iface, k, s);
  if r1 == r2 && r2 == r3 && r1 == 55 { return 0; }
  return 1;
}
