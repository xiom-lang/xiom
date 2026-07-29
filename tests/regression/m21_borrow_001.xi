module m21_borrow_001
pub fn run() -> Int {
    var x = 42;
    var r = &x;
    if *r == 42 { return 0; }
    return 1;
  }
use m21_borrow_001.run;
fn main() -> Int { return run(); }
