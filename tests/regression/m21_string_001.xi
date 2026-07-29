module m21_string_001
pub fn run() -> Int {
    var a = "hello";
    var b = "hello";
    if a == b { return 0; }
    return 1;
  }
use m21_string_001.run;
fn main() -> Int { return run(); }
