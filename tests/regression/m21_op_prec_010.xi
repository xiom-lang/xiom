module m21_op_prec_010
pub fn run() -> Int {
    var x = 5;
    var y = 3;
    var z = 2;
    var r = x + y * z - x / z;
    if r == 9 { return 0; }
    return 1;
  }
use m21_op_prec_010.run;
fn main() -> Int { return run(); }
