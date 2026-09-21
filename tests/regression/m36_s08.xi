// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S08: Codegen -- register allocation simulation (linear scan style)
type RegState = { allocated: Bool; spill: Bool; temp: Bool; }
fn init_reg(alloc: Bool, sp: Bool, tmp: Bool) -> RegState {
  return RegState{ allocated: alloc; spill: sp; temp: tmp; };
}
fn alloc_reg(available: Int) -> Bool {
  return available > 0;
}
fn free_reg(count: Int, used: Int) -> Int {
  return count - used;
}
fn spill_reg(r: RegState) -> RegState {
  return RegState{ allocated: true; spill: true; temp: r.temp; };
}
fn count_free_r1(r0: Bool, r1: Bool) -> Int {
  var f: Int = 0;
  if !r0 { f = f + 1; }
  if !r1 { f = f + 1; }
  return f;
}
fn count_free_r3(r0: Bool, r1: Bool, r2: Bool) -> Int {
  var f: Int = count_free_r1(r0, r1);
  if !r2 { f = f + 1; }
  return f;
}
fn alloc_success(free: Int, need: Int) -> Bool {
  return free >= need;
}
fn main() -> Int {
  var r0 = init_reg(false, false, false);
  if r0.allocated || r0.spill { return 1; }
  var avail: Int = 8;
  if !alloc_reg(avail) { return 2; }
  var left = free_reg(avail, 3);
  if left != 5 { return 3; }
  var spilled = spill_reg(r0);
  if !spilled.allocated || !spilled.spill { return 4; }
  if alloc_reg(0) { return 5; }
  var f2 = count_free_r1(false, false);
  if f2 != 2 { return 6; }
  var f3 = count_free_r3(false, true, false);
  if f3 != 2 { return 7; }
  if !alloc_success(4, 2) { return 8; }
  if alloc_success(1, 3) { return 9; }
  return 0;
}
