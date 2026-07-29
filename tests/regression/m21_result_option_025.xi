module m21_result_option_025
pub fn run() -> Int {
    var opt: Option[Int] = None;
    if opt.is_none() { return 0; }
    return 1;
  }
use m21_result_option_025.run;
fn main() -> Int { return run(); }
