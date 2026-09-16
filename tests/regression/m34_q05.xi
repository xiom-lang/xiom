// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q05: Contract + invariant + struct -- struct with invariant, functions with contracts
type Bank = {
  balance: Int;
  invariant: balance >= 0;
}

fn deposit(b: Bank, amount: Int) -> Bank
  requires: amount > 0
  ensures: result.balance == b.balance + amount
{
  return Bank{ balance: b.balance + amount; };
}
fn withdraw(b: Bank, amount: Int) -> Bank
  requires: amount > 0
  requires: b.balance >= amount
  ensures: result.balance >= 0
{
  return Bank{ balance: b.balance - amount; };
}
fn transfer(from: Bank, to: Bank, amt: Int) -> Bank
  requires: amt > 0
  requires: from.balance >= amt
{
  return Bank{ balance: to.balance + amt; };
}
fn main() -> Int {
  var a = Bank{ balance: 100; };
  var b = Bank{ balance: 50; };
  var a2 = deposit(a, 25);
  var a3 = withdraw(a2, 30);
  var b2 = transfer(a2, b, 15);
  if a2.balance == 125 && a3.balance == 95 && b2.balance == 65 { return 0; }
  return 1;
}
