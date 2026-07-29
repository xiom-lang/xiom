module m21_if_chain_008
pub fn run() -> Int {
    var a = true;
    var b = false;
    if a && !b { return 0; }
    elif b { return 1; }
    else { return 1; }
  }
use m21_if_chain_008.run;
fn main() -> Int { return run(); }
