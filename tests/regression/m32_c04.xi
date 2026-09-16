// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C04: Multiple requires -- transfer with three preconditions
fn transfer(amount: Int, balance: Int, limit: Int) -> Int
  requires: amount > 0
  requires: balance >= amount
  requires: limit >= amount
{
  return balance - amount;
}
fn main() -> Int {
  var r: Int = transfer(30, 200, 100);
  if r == 170 { return 0; }
  return 1;
}
