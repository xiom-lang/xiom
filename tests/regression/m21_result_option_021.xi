module m21_result_option_021
pub fn run() -> Int {
    var opt: Option[Int] = Some(99);
    match opt {
      Some(v) => if v == 99 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_021.run;
fn main() -> Int { return run(); }
