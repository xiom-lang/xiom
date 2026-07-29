module m21_borrow_014
pub fn run() -> Int {
    var val = 3;
    var r = &val;
    match *r {
      1 => return 1,
      2 => return 1,
      3 => return 0,
      _ => return 1,
    }
  }
use m21_borrow_014.run;
fn main() -> Int { return run(); }
