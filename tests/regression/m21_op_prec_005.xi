module m21_op_prec_005
pub fn run() -> Int {
    var r = 2 + 6 * 3 - 4 / 2;
    if r == 18 { return 0; }
    return 1;
  }
use m21_op_prec_005.run;
fn main() -> Int { return run(); }
