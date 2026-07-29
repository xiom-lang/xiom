module m21_contract_005
type BoundedInt = {
    value: Int;
    lo: Int;
    hi: Int;
    invariant: value >= lo;
    invariant: value <= hi;
  }

  pub fn run() -> Int {
    var b: BoundedInt = { value: 50; lo: 0; hi: 100; };
    if b.value == 50 { return 0; }
    return 1;
  }
use m21_contract_005.run;
fn main() -> Int { return run(); }
