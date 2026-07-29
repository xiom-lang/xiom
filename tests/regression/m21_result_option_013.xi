module m21_result_option_013
type User = { name: Str; id: Int; }

  pub fn run() -> Int {
    var opt: Option[User] = None;
    match opt {
      Some(_) => return 1,
      None => return 0,
    }
  }
use m21_result_option_013.run;
fn main() -> Int { return run(); }
