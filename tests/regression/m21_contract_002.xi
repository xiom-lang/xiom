module m21_contract_002
fn abs(x: Int) -> Int
    requires: x >= -1000000
    ensures: result >= 0
  {
    if x < 0 { return -x; }
    return x;
  }

  pub fn run() -> Int {
    var a = abs(-42);
    if a == 42 { return 0; }
    return 1;
  }
use m21_contract_002.run;
fn main() -> Int { return run(); }
