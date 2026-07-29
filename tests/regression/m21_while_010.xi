module m21_while_010
pub fn run() -> Int {
    var i = 0;
    while true {
      i = i + 1;
      if i > 100 { break; }
    }
    if i == 101 { return 0; }
    return 1;
  }
use m21_while_010.run;
fn main() -> Int { return run(); }
