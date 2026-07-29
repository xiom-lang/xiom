module m21_while_007
pub fn run() -> Int {
    var i = 10;
    while i > 0 {
      i = i - 1;
    }
    if i == 0 { return 0; }
    return 1;
  }
use m21_while_007.run;
fn main() -> Int { return run(); }
