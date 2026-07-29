module m21_struct_mut_004
type L1 = { val: Int; }
  type L2 = { l1: L1; }
  type L3 = { l2: L2; }

  pub fn run() -> Int {
    var x: L3 = { l2: { l1: { val: 1; }; }; };
    x.l2.l1.val = 99;
    if x.l2.l1.val == 99 { return 0; }
    return 1;
  }
use m21_struct_mut_004.run;
fn main() -> Int { return run(); }
