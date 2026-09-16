// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C30: Combined mega test 3 -- ALL_FEATURES: struct+enum+match+while+if+derive+module+Option+Result+array+pointer+unsafe+cast+compound_assign+const+type_alias+recursion
const MAGIC: Int = 42;
type Entity = { hp: Int; mp: Int; alive: Bool; } derive[Eq]
fn Entity.new() -> Entity { return Entity{ hp: 100; mp: 50; alive: true; }; }
fn Entity.damage(self, dmg: Int) -> Entity { var r = self; r.hp = r.hp - dmg; if r.hp < 0 { r.hp = 0; r.alive = false; } return r; }
fn Entity.heal(self, amt: Int) -> Entity { var r = self; r.hp = r.hp + amt; return r; }
fn Entity.tick(self) -> Entity { if self.alive { return self.damage(1); } return self; }
fn factorial30(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial30(n - 1);
}
fn pointer_identity() -> Int {
  var x: Int = MAGIC;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var v: Int;
  unsafe { v = *p; }
  return v;
}
fn main() -> Int {
  var e = Entity.new();
  if e.hp != 100 { return 1; }
  if !e.alive { return 2; }
  e = e.damage(30);
  if e.hp != 70 { return 3; }
  if !e.alive { return 4; }
  e = e.heal(10);
  if e.hp != 80 { return 5; }
  e = e.tick();
  if e.hp != 79 { return 6; }
  if factorial30(5) != 120 { return 7; }
  if pointer_identity() != 42 { return 8; }
  var arr = Vec[Int].new();
  arr.push(1); arr.push(2); arr.push(3);
  var sum = 0; var idx = 0;
  while idx < arr.len() { sum += arr[idx]; idx += 1; }
  if sum != 6 { return 9; }
  var opt: Option[Int] = Some(99);
  match opt { Some(v) => { if v != 99 { return 10; } } None => { return 11; } }
  var optn: Option[Int] = None;
  match optn { Some(_) => { return 12; } None => {} }
  var r: Result[Int, Str] = Ok(42);
  match r { Ok(v) => { if v != 42 { return 13; } } Err(_) => { return 14; } }
  var chk = 0;
  var i = 0;
  while i < 5 { chk += 1; i += 1; }
  if chk != 5 { return 15; }
  return 0;
}
