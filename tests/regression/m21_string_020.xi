module m21_string_020
pub fn run() -> Int {
    var a = "abc";
    var b = "def";
    var c = a.concat(b);
    if c.len() == 6 && c == "abcdef" { return 0; }
    return 1;
  }
use m21_string_020.run;
fn main() -> Int { return run(); }
