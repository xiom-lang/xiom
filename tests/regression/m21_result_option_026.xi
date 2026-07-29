module m21_result_option_026
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(1);
    if r.is_ok() { return 0; }
    return 1;
  }
use m21_result_option_026.run;
fn main() -> Int { return run(); }
