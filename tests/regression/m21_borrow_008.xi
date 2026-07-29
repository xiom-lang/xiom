module m21_borrow_008
fn double(r: &Int) -> Int {
    return *r * 2;
  }

  pub fn run() -> Int {
    var x = 21;
    var v = double(&x);
    if v == 42 { return 0; }
    return 1;
  }
use m21_borrow_008.run;
fn main() -> Int { return run(); }
