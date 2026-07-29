module m21_while_003
pub fn run() -> Int {
    var i = 0;
    var sum = 0;
    while i < 10 {
      i = i + 1;
      if i == 5 { continue; }
      sum = sum + i;
    }
    if sum == 50 { return 0; }
    return 1;
  }
use m21_while_003.run;
fn main() -> Int { return run(); }
