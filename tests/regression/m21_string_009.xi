module m21_string_009
pub fn run() -> Int {
    var v: Vec[Str] = ["a", "b", "c"];
    v.push("d");
    if v.len() == 4 && v[3] == "d" { return 0; }
    return 1;
  }
use m21_string_009.run;
fn main() -> Int { return run(); }
