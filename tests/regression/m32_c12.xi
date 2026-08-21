// M32-C12: Contract fail -- contracts trap on violation (safe usage shown)
fn withdraw(balance: Int, amount: Int) -> Int
  requires: amount > 0
  requires: balance >= amount
  ensures: result == balance - amount
{
  return balance - amount;
}
fn main() -> Int {
  var r: Int = withdraw(500, 75);
  if r == 425 { return 0; }
  return 1;
}
