module m21_op_prec_002
pub fn run() -> Int {
    var r = 10 - 3 - 2;
    if r == 5 { return 0; }
    return 1;
  }
use m21_op_prec_002.run;
fn main() -> Int { return run(); }
