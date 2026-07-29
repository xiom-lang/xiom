module m21_while_006
pub fn run() -> Int {
    var sum = 0;
    var i = 1;
    while i <= 10 {
      sum = sum + i;
      i = i + 1;
    }
    if sum == 55 { return 0; }
    return 1;
  }
use m21_while_006.run;
fn main() -> Int { return run(); }
