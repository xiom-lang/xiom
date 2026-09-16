// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y04: enum payload + method with contracts + match extraction + module + impl + generic + diff
type Vec2 = { x: Int; y: Int; }
enum ShapeKind { Rect, Square, Circle }
fn area_rect(v: Vec2) -> Int
  requires: v.x >= 0
  requires: v.y >= 0
  ensures: result >= 0
{ return v.x * v.y; }
fn area_sq(side: Int) -> Int
  requires: side >= 0
  ensures: result >= 0
{ return side * side; }
interface Measurable { fn size(self) -> Int; }
impl Measurable for Vec2 {
  fn size(self) -> Int { return self.x * self.y; }
}
module mathlib {
  pub fn area_rect_m(v: Vec2) -> Int { return area_rect(v); }
  pub fn area_sq_m(side: Int) -> Int { return area_sq(side); }
  pub fn via_impl(v: Vec2) -> Int { return v.size(); }
}
use mathlib.area_rect_m;
use mathlib.area_sq_m;
use mathlib.via_impl;
enum CalcVia { Rect, Sq, Impl }
fn calc(via: CalcVia, v: Vec2, side: Int) -> Int {
  match via { Rect => area_rect_m(v), Sq => area_sq_m(side), Impl => via_impl(v), }
}
fn main() -> Int {
  var v = Vec2{ x: 8; y: 8; };
  var r1 = calc(CalcVia.Rect, v, 0);
  var r2 = calc(CalcVia.Sq, v, 8);
  var r3 = calc(CalcVia.Impl, v, 0);
  if r1 == r2 && r2 == r3 && r1 == 64 { return 0; }
  return 1;
}
