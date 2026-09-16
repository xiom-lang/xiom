// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S18: Optimization -- dead code elimination analysis
type DefUse = { id: Int; defined: Bool; used: Bool; reachable: Bool; }
fn make_def_use(id: Int, def: Bool, used: Bool, reach: Bool) -> DefUse {
  return DefUse{ id: id; defined: def; used: used; reachable: reach; };
}
fn is_dead_code(du: DefUse) -> Bool {
  return du.defined && !du.used;
}
fn is_unreachable(du: DefUse) -> Bool {
  return !du.reachable;
}
fn can_eliminate(du: DefUse) -> Bool {
  return is_dead_code(du) || is_unreachable(du);
}
fn mark_used(du: DefUse) -> DefUse {
  return DefUse{ id: du.id; defined: du.defined; used: true; reachable: du.reachable; };
}
fn mark_unreachable(du: DefUse) -> DefUse {
  return DefUse{ id: du.id; defined: du.defined; used: du.used; reachable: false; };
}
fn eliminate_count(a: Bool, b: Bool, c: Bool, d: Bool) -> Int {
  var cnt: Int = 0;
  if a { cnt = cnt + 1; }
  if b { cnt = cnt + 1; }
  if c { cnt = cnt + 1; }
  if d { cnt = cnt + 1; }
  return cnt;
}
fn main() -> Int {
  var live = make_def_use(1, true, true, true);
  var dead = make_def_use(2, true, false, true);
  var unreach = make_def_use(3, false, false, false);
  var unused = make_def_use(4, false, true, true);
  if can_eliminate(live) { return 1; }
  if !can_eliminate(dead) { return 2; }
  if !can_eliminate(unreach) { return 3; }
  if can_eliminate(unused) { return 4; }
  var marked = mark_used(dead);
  if !can_eliminate(dead) { return 5; }
  if can_eliminate(marked) { return 6; }
  var unreach2 = mark_unreachable(live);
  if !can_eliminate(unreach2) { return 7; }
  var c = eliminate_count(can_eliminate(live), can_eliminate(dead), can_eliminate(unreach), can_eliminate(unused));
  if c != 2 { return 8; }
  return 0;
}
