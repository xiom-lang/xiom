module m21_string_017
pub fn run() -> Int {
    var opt: Option[Str] = Some("present");
    match opt {
      Some(s) => if s == "present" { return 0; },
      None => return 1,
    }
  }
use m21_string_017.run;
fn main() -> Int { return run(); }
