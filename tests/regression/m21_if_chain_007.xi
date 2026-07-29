module m21_if_chain_007
pub fn run() -> Int {
    var x: Int32 = 50000i32;
    if x == 10000i32 { return 1; }
    elif x == 50000i32 { return 0; }
    else { return 1; }
  }
use m21_if_chain_007.run;
fn main() -> Int { return run(); }
