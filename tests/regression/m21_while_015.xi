module m21_while_015
pub fn run() -> Int {
    var x = 0;
    var y = 0;
    var z = 0;
    var count = 0;
    while x < 3 {
      x = x + 1;
      y = 0;
      while y < 2 {
        y = y + 1;
        z = 0;
        while z < 2 {
          z = z + 1;
          count = count + 1;
        }
      }
    }
    if count == 12 { return 0; }
    return 1;
  }
use m21_while_015.run;
fn main() -> Int { return run(); }
