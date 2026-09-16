// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X20: Dead code paths -- conditional branches that are unreachable
fn always_true() -> Bool { return true; }
fn always_false() -> Bool { return false; }
fn classify(n: Int) -> Int {
  if n > 0 { return 1; }
  if n < 0 { return -1; }
  return 0;
}
fn dead_branch_test(x: Int) -> Int {
  if true { return 42; }
  return 0;
}
fn const_dead(flag: Bool) -> Int {
  if flag { return 10; }
  return 20;
}
fn unreachable_else(x: Int) -> Int {
  if x >= 0 {
    if x == 0 { return 0; }
    return 1;
  }
  return -1;
}
fn main() -> Int {
  if dead_branch_test(100) != 42 { return 1; }
  if const_dead(true) != 10 { return 2; }
  if const_dead(false) != 20 { return 3; }
  if classify(5) != 1 { return 4; }
  if classify(0) != 0 { return 5; }
  if classify(-3) != -1 { return 6; }
  if unreachable_else(0) != 0 { return 7; }
  if unreachable_else(7) != 1 { return 8; }
  if unreachable_else(-5) != -1 { return 9; }
  return 0;
}
