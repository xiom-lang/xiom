module m21_string_012
fn make_label() -> Str {
    return "label";
  }

  pub fn run() -> Int {
    var s = make_label();
    if s.len() == 5 { return 0; }
    return 1;
  }
use m21_string_012.run;
fn main() -> Int { return run(); }
