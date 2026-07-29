module m21_string_013
pub fn run() -> Int {
    var a = "one";
    var b = "two";
    var c = a;
    a = "three";
    if c == "one" && a == "three" && b == "two" { return 0; }
    return 1;
  }
use m21_string_013.run;
fn main() -> Int { return run(); }
