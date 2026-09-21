// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C17: Every generic instantiation count 1-5 per function -- identity, pair, triple, quad, quintuple with different type args
fn id[T](x: T) -> T { return x; }
fn pair2[T](a: T, b: T) -> T { if a == a { return a; } return b; }
fn triple3[T](a: T, b: T, c: T) -> T { return a; }
fn quad4[T](a: T, b: T, c: T, d: T) -> T { return a; }
fn quintuple5[T](a: T, b: T, c: T, d: T, e: T) -> T { return a; }
fn multi_inst1() -> Int { var a = id(42); return a; }
fn multi_inst2() -> Int { var a = id(10); var b = id(20); return a + b; }
fn multi_inst3() -> Int { var a = id(1); var b = id(2); var c = id(3); return a + b + c; }
fn multi_inst4() -> Int { var a = id(1); var b = id(2); var c = id(3); var d = id(4); return a + b + c + d; }
fn multi_inst5() -> Int { var a = id(1); var b = id(2); var c = id(3); var d = id(4); var e = id(5); return a + b + c + d + e; }
fn main() -> Int {
  if id(42) != 42 { return 1; }
  if id(true) != true { return 2; }
  if id("x") != "x" { return 3; }
  if pair2(10, 20) != 10 { return 4; }
  if pair2(true, false) != true { return 5; }
  if triple3(1, 2, 3) != 1 { return 6; }
  if quad4(1, 2, 3, 4) != 1 { return 7; }
  if quintuple5(1, 2, 3, 4, 5) != 1 { return 8; }
  if multi_inst1() != 42 { return 9; }
  if multi_inst2() != 30 { return 10; }
  if multi_inst3() != 6 { return 11; }
  if multi_inst4() != 10 { return 12; }
  if multi_inst5() != 15 { return 13; }
  return 0;
}
