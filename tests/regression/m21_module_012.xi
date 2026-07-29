module m21_module_012
pub fn square(x: Int) -> Int { return x * x; }

  pub fn sum_of_squares(a: Int, b: Int) -> Int {
    return square(a) + square(b);
  }

  pub fn run() -> Int {
    var s = sum_of_squares(3, 4);
    if s == 25 { return 0; }
    return 1;
  }
use m21_module_012.run;
fn main() -> Int { return run(); }
