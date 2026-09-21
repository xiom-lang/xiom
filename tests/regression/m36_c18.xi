// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C18: Every contract clause count 1-5 -- functions with increasing numbers of requires/ensures/invariant clauses
type Safe = { val: Int; invariant: val >= 0; }
type Guarded1 = { x: Int; invariant: x >= 0; }
type Guarded2 = { x: Int; y: Int; invariant: x >= 0; }
type Guarded3 = { x: Int; y: Int; z: Int; invariant: x >= 0; invariant: y >= 0; }
type Guarded4 = { a: Int; b: Int; c: Int; invariant: a >= 0; invariant: b >= 0; invariant: c >= 0; }
type Guarded5 = { a: Int; b: Int; c: Int; invariant: a >= 0; invariant: b >= 0; invariant: c >= 0; }
fn contract1_simple(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
{ return x; }
fn contract2_multi(x: Int, y: Int) -> Int
  requires: x >= 0
  requires: y >= 0
  ensures: result >= 0
  ensures: result >= x + y
{ return x + y; }
fn contract3_triple(x: Int, y: Int, z: Int) -> Int
  requires: x >= 0
  requires: y >= 0
  requires: z >= 0
  ensures: result >= 0
{ return x + y + z; }
fn contract4_quad(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
  ensures: result >= x
  ensures: result <= x * x + x
{ if x == 0 { return 0; } return x + 1; }
fn contract5_quint(x: Int) -> Int
  requires: x >= 0
  requires: x <= 100
  ensures: result >= 0
  ensures: result >= x
  ensures: result <= x * 2
{ if x < 10 { return x * 2; } return x + 5; }
fn main() -> Int {
  if contract1_simple(5) != 5 { return 1; }
  if contract1_simple(0) != 0 { return 2; }
  if contract2_multi(3, 7) != 10 { return 3; }
  if contract2_multi(0, 0) != 0 { return 4; }
  if contract3_triple(1, 2, 3) != 6 { return 5; }
  if contract3_triple(0, 0, 0) != 0 { return 6; }
  if contract4_quad(5) != 6 { return 7; }
  if contract4_quad(0) != 0 { return 8; }
  if contract5_quint(5) != 10 { return 9; }
  if contract5_quint(20) != 25 { return 10; }
  if contract5_quint(0) != 0 { return 11; }
  var g1 = Guarded1{ x: 5; };
  if g1.x != 5 { return 12; }
  var g2 = Guarded2{ x: 1; y: 2; };
  if g2.x + g2.y != 3 { return 13; }
  var g3 = Guarded3{ x: 1; y: 2; z: 3; };
  if g3.x + g3.y + g3.z != 6 { return 14; }
  var g4 = Guarded4{ a: 1; b: 2; c: 3; };
  if g4.a + g4.b + g4.c != 6 { return 15; }
  var g5 = Guarded5{ a: 1; b: 2; c: 3; };
  if g5.a != 1 { return 16; }
  return 0;
}
