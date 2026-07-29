module m21_while_011
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3, 4, 5];
    var sum = 0;
    var i = 0;
    while i < v.len() {
      sum = sum + v[i];
      i = i + 1;
    }
    if sum == 15 { return 0; }
    return 1;
  }
use m21_while_011.run;
fn main() -> Int { return run(); }
