module test_transfer
fn transfer(balance: Int, amount: Int) -> Int
  requires: amount > 0
  requires: balance >= amount
  ensures: result == balance - amount
  ensures: result >= 0
{ return balance - amount; }
