module m21_contract_003
fn factorial(n: Int) -> Int
    requires: n >= 0
    ensures: result >= 1
  {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
  }

  pub fn run() -> Int {
    var f = factorial(5);
    if f == 120 { return 0; }
    return 1;
  }
use m21_contract_003.run;
fn main() -> Int { return run(); }
