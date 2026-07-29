module m21_result_option_040
pub fn run() -> Int {
    var opt: Option[Vec[Int]] = Some([1, 2, 3]);
    match opt {
      Some(vec) => if vec.len() == 3 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_040.run;
fn main() -> Int { return run(); }
