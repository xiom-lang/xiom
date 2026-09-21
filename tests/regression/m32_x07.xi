// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X07: Combinatorial + Differential -- manual eq vs derive Eq with struct+enum
type Vec2 = { x: Int; y: Int; } derive[Eq]
fn manual_eq(a: Vec2, b: Vec2) -> Bool {
  return a.x == b.x && a.y == b.y;
}
fn derived_eq(a: Vec2, b: Vec2) -> Bool {
  return a == b;
}
enum EqMethod { Manual, Derived }
fn check(m: EqMethod, a: Vec2, b: Vec2) -> Bool {
  match m { Manual => manual_eq(a, b), Derived => derived_eq(a, b), }
}
fn main() -> Int {
  var v1 = Vec2{ x: 10; y: 20; };
  var v2 = Vec2{ x: 10; y: 20; };
  var v3 = Vec2{ x: 30; y: 40; };
  var r1 = check(EqMethod.Manual, v1, v2) && check(EqMethod.Derived, v1, v2);
  var r2 = !check(EqMethod.Manual, v1, v3) && !check(EqMethod.Derived, v1, v3);
  if r1 && r2 { return 0; }
  return 1;
}
