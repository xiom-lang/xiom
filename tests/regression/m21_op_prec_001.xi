module m21_op_prec_001
pub fn run() -> Int {
    var r = 2 + 3 * 4;
    if r == 14 { return 0; }
    return 1;
  }
use m21_op_prec_001.run;
fn main() -> Int { return run(); }
