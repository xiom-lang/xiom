module m21_result_option_015
pub fn run() -> Int {
    var r: Result[Int, Int] = Err(-1);
    var v = r.unwrap_or(99);
    if v == 99 { return 0; }
    return 1;
  }
use m21_result_option_015.run;
fn main() -> Int { return run(); }
