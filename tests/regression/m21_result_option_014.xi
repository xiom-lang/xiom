module m21_result_option_014
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(42);
    if r.is_ok() && r.unwrap() == 42 { return 0; }
    return 1;
  }
use m21_result_option_014.run;
fn main() -> Int { return run(); }
