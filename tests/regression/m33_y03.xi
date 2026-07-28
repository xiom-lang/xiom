// M33-Y03: module + type + generic fn + enum dispatch + match + contract + impl method + diff
type Account = { id: Int; balance: Int; }
enum TxKind { Deposit, Withdraw, Transfer }
fn apply[T](acct: Account, kind: TxKind, amt: Int) -> Int
  requires: amt > 0
  requires: acct.balance >= 0
  ensures: result >= 0
{
  match kind {
    Deposit => acct.balance + amt,
    Withdraw => if acct.balance >= amt { acct.balance - amt } else { acct.balance },
    Transfer => if acct.balance >= amt * 2 { acct.balance - amt } else { acct.balance },
  }
}
interface BalanceOps { fn check(self, min: Int) -> Bool; fn deposit(self, amt: Int) -> Int; }
impl BalanceOps for Account {
  fn check(self, min: Int) -> Bool { return self.balance >= min; }
  fn deposit(self, amt: Int) -> Int { return self.balance + amt; }
}
module bank {
  pub fn do_apply(acct: Account, kind: TxKind, amt: Int) -> Int { return apply(acct, kind, amt); }
  pub fn do_deposit(acct: Account, amt: Int) -> Int { return acct.deposit(amt); }
}
use bank.do_apply;
use bank.do_deposit;
enum Route { Direct, Impl }
fn route(r: Route, acct: Account, tx: TxKind, amt: Int) -> Int {
  match r { Direct => do_apply(acct, tx, amt), Impl => do_deposit(acct, amt), }
}
fn main() -> Int {
  var a = Account{ id: 1; balance: 50; };
  var r1 = route(Route.Direct, a, TxKind.Deposit, 30);
  var r2 = route(Route.Impl, a, TxKind.Deposit, 30);
  if r1 == r2 && r1 == 80 { return 0; }
  return 1;
}
