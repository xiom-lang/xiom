// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C16: Every function chain length 1-10 -- functions calling functions in chains of increasing depth
fn f1(x: Int) -> Int { return x + 1; }
fn f2(x: Int) -> Int { return f1(x) + 1; }
fn f3(x: Int) -> Int { return f2(x) + 1; }
fn f4(x: Int) -> Int { return f3(x) + 1; }
fn f5(x: Int) -> Int { return f4(x) + 1; }
fn f6(x: Int) -> Int { return f5(x) + 1; }
fn f7(x: Int) -> Int { return f6(x) + 1; }
fn f8(x: Int) -> Int { return f7(x) + 1; }
fn f9(x: Int) -> Int { return f8(x) + 1; }
fn f10(x: Int) -> Int { return f9(x) + 1; }
fn chain_double_recurse(x: Int, depth: Int) -> Int {
  if depth <= 0 { return x; }
  return chain_double_recurse(x * 2, depth - 1);
}
fn mixed_chain(a: Int) -> Int {
  var s = f1(a);
  s = f3(s);
  s = f5(s);
  return s;
}
fn main() -> Int {
  if f1(0) != 1 { return 1; }
  if f2(0) != 2 { return 2; }
  if f3(0) != 3 { return 3; }
  if f4(0) != 4 { return 4; }
  if f5(0) != 5 { return 5; }
  if f6(0) != 6 { return 6; }
  if f7(0) != 7 { return 7; }
  if f8(0) != 8 { return 8; }
  if f9(0) != 9 { return 9; }
  if f10(0) != 10 { return 10; }
  if f10(5) != 15 { return 11; }
  if chain_double_recurse(1, 1) != 2 { return 12; }
  if chain_double_recurse(1, 5) != 32 { return 13; }
  if chain_double_recurse(1, 10) != 1024 { return 14; }
  if mixed_chain(0) != 9 { return 15; }
  return 0;
}
