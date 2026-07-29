module m21_result_option_006
pub fn run() -> Int {
    var r: Result[Int16, Int16] = Err(-32768i16);
    match r {
      Ok(_) => return 1,
      Err(e) => if e == -32768i16 { return 0; },
    }
  }
use m21_result_option_006.run;
fn main() -> Int { return run(); }
