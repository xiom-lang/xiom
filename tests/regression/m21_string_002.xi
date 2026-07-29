module m21_string_002
pub fn run() -> Int {
    var a = "alpha";
    var b = "alpha";
    var c = "beta";
    if a == b && a != c { return 0; }
    return 1;
  }
use m21_string_002.run;
fn main() -> Int { return run(); }
