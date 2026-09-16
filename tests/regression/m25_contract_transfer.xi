// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M25: Multiple requires -- transfer with balance check
fn transfer(amount: Int, balance: Int) -> Int
  requires: amount > 0
  requires: balance >= amount
{
  return balance - amount;
}
fn main() -> Int {
  var r: Int = transfer(10, 100);
  if r == 90 { return 0; }
  return 1;
}
