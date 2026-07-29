module m21_result_option_017
pub fn run() -> Int {
    var opt: Option[Int] = None;
    var v = opt.unwrap_or(42);
    if v == 42 { return 0; }
    return 1;
  }
use m21_result_option_017.run;
fn main() -> Int { return run(); }
