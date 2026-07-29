module m21_string_014
fn check(s: Str) -> Bool {
    return s.len() > 0;
  }

  pub fn run() -> Int {
    var name = "test";
    if check(name) { return 0; }
    return 1;
  }
use m21_string_014.run;
fn main() -> Int { return run(); }
