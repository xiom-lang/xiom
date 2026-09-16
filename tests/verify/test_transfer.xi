// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_transfer
fn transfer(balance: Int, amount: Int) -> Int
  requires: amount > 0
  requires: balance >= amount
  ensures: result == balance - amount
  ensures: result >= 0
{ return balance - amount; }
