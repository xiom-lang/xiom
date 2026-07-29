module m21_result_option_037
pub fn run() -> Int {
    var opt: Option[Int] = Some(50);
    match opt {
      Some(v) if v > 25 => return 0,
      Some(_) => return 1,
      None => return 1,
    }
  }
use m21_result_option_037.run;
fn main() -> Int { return run(); }
