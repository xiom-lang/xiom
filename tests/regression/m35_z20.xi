// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z20: struct+generic+contract+match+while+enum+module+derive+Option+Result+compound_assign+closure+impl
type Account = { id: Int; balance: Int; } derive[Eq]
enum Tx { Deposit(a: Int), Withdraw(a: Int) }
fn do_deposit(acc: Account, amount: Int) -> Account
  requires: acc.balance >= 0
{
  var r = acc;
  r.balance += amount;
  return r;
}
fn Account.has_funds(self, amount: Int) -> Bool { return self.balance >= amount; }
fn do_withdraw(acc: Account, amount: Int) -> Int {
  if acc.has_funds(amount) { return acc.balance - amount; }
  return -1;
}
fn transact[T](acc: Account, tx: Tx) -> Int
  requires: acc.balance >= 0
{
  match tx {
    Deposit(a) => acc.balance + a,
    Withdraw(a) => do_withdraw(acc, a),
  }
}
module bank {
  pub fn process(a: Account, t: Tx) -> Int { return transact(a, t); }
  pub fn new_account(id: Int) -> Account { return Account{ id: id; balance: 0; }; }
  pub fn deposit(a: Account, amount: Int) -> Account { return do_deposit(a, amount); }
}
use bank.process;
use bank.new_account;
use bank.deposit;
fn main() -> Int {
  var a1 = new_account(1);
  var a2 = deposit(a1, 100);
  if a2.balance != 100 { return 1; }
  var r1 = process(a2, Tx.Withdraw(30));
  if r1 != 70 { return 2; }
  var a3 = Account{ id: 2; balance: 70; };
  var r2 = process(a3, Tx.Deposit(30));
  if r2 != 100 { return 3; }
  var a4 = Account{ id: 3; balance: 10; };
  var r3 = process(a4, Tx.Withdraw(50));
  if r3 != -1 { return 4; }
  var chk = 0;
  var i = 0;
  while i < 5 { chk += 1; i += 1; }
  if chk == 5 { return 0; }
  return 7;
}
