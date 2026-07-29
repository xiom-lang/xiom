module m21_if_chain_001
pub fn run() -> Int {
    var x = 10;
    if x > 5 { return 0; }
    else { return 1; }
  }
use m21_if_chain_001.run;
fn main() -> Int { return run(); }
