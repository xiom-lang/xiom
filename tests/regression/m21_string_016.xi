module m21_string_016
pub fn run() -> Int {
    var s = "abcdefghijklmnopqrstuvwxyz0123456789";
    if s.len() == 36 { return 0; }
    return 1;
  }
use m21_string_016.run;
fn main() -> Int { return run(); }
