module m21_if_chain_005
pub fn run() -> Int {
    var x: Int8 = 100i8;
    if x == 50i8 { return 1; }
    elif x == 75i8 { return 1; }
    elif x == 100i8 { return 0; }
    else { return 1; }
  }
use m21_if_chain_005.run;
fn main() -> Int { return run(); }
