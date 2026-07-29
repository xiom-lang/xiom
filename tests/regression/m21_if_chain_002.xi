module m21_if_chain_002
pub fn run() -> Int {
    var x = 15;
    if x == 10 { return 1; }
    elif x == 15 { return 0; }
    else { return 2; }
  }
use m21_if_chain_002.run;
fn main() -> Int { return run(); }
