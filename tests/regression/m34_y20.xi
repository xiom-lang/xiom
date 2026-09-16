// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y20: all-in-one: struct+enum+generic+contract+match+impl+module+derive+Option+compound_assign+unsafe+while
type Entity = { hp: Int; mp: Int; alive: Bool; } derive[Eq]
enum Spell { Heal(n: Int), Fire(n: Int), Drain(n: Int), Revive }
type Target = { entity: Entity; }
fn heal_hp(hp: Int, n: Int) -> Int { var r = hp; r = r + n; return r; }
fn wrap_entity(hp: Int, mp: Int, a: Bool) -> Entity {
  return Entity{ hp: hp; mp: mp; alive: a; };
}
fn cast[T](t: Target, spell: Spell) -> Result[Entity, Str]
  requires: t.entity.hp >= 0
  requires: t.entity.mp >= 0
{
  var h = t.entity.hp;
  var m = t.entity.mp;
  var alive = t.entity.alive;
  match spell {
    Heal(n) => {
      if !alive { return Err("dead"); }
      h = heal_hp(h, n);
    }
    Fire(n) => {
      if !alive { return Err("dead"); }
      h = h - n;
      m = m - 1;
      if h <= 0 { alive = false; h = 0; }
    }
    Drain(n) => {
      if !alive { return Err("dead"); }
      var i = 0;
      while i < n { h = h - 1; m = m + 1; i = i + 1; if h <= 0 { alive = false; break; } }
    }
    Revive => {
      if alive { h = h + 10; }
      else { alive = true; h = 1; }
    }
  }
  if !alive && h > 0 { h = 0; }
  return Ok(wrap_entity(h, m, alive));
}
interface Caster { fn cast_spell(self, s: Spell) -> Result[Entity, Str]; }
impl Caster for Target {
  fn cast_spell(self, s: Spell) -> Result[Entity, Str] { return cast(self, s); }
}
module battle {
  pub fn do_cast(t: Target, s: Spell) -> Result[Entity, Str] { return cast(t, s); }
  pub fn is_alive(t: Target) -> Bool { return t.entity.alive; }
  pub fn hp_val(t: Target) -> Int { return t.entity.hp; }
}
use battle.do_cast;
use battle.is_alive;
use battle.hp_val;
fn main() -> Int {
  var e1 = Entity{ hp: 10; mp: 5; alive: true; };
  var t1 = Target{ entity: e1; };
  var e2 = Entity{ hp: 10; mp: 5; alive: true; };
  var t2 = Target{ entity: e2; };
  var e3 = Entity{ hp: 10; mp: 5; alive: true; };
  var t3 = Target{ entity: e3; };
  match do_cast(t1, Spell.Heal(5)) {
    Ok(r) => { if r.hp != 15 { return 1; } }
    Err(_) => { return 2; }
  }
  match do_cast(t2, Spell.Fire(3)) {
    Ok(r) => { if r.hp != 7 || r.mp != 4 { return 3; } }
    Err(_) => { return 4; }
  }
  if !is_alive(t3) { return 5; }
  if hp_val(t3) != 10 { return 6; }
  return 0;
}
