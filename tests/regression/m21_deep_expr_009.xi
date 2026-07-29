module m21_deep_expr_009
type L1 = { v: Int; }
  type L2 = { l1: L1; }
  type L3 = { l2: L2; }
  type L4 = { l3: L3; }
  type L5 = { l4: L4; }
  type L6 = { l5: L5; }
  type L7 = { l6: L6; }
  type L8 = { l7: L7; }

  pub fn run() -> Int {
    var x: L8 = { l7: { l6: { l5: { l4: { l3: { l2: { l1: { v: 42; }; }; }; }; }; }; }; };
    x.l7.l6.l5.l4.l3.l2.l1.v = 99;
    if x.l7.l6.l5.l4.l3.l2.l1.v == 99 { return 0; }
    return 1;
  }
use m21_deep_expr_009.run;
fn main() -> Int { return run(); }
