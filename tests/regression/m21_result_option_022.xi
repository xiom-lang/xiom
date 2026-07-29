module m21_result_option_022
pub fn run() -> Int {
    var opt: Option[Int] = None;
    match opt {
      Some(_) => return 1,
      None => return 0,
    }
  }
use m21_result_option_022.run;
fn main() -> Int { return run(); }
