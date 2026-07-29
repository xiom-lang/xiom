module m21_if_chain_013
pub fn run() -> Int {
    var x = 5;
    if x > 0 { return 0; }
    return 1;
  }
use m21_if_chain_013.run;
fn main() -> Int { return run(); }
