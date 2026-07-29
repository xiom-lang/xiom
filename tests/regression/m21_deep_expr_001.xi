module m21_deep_expr_001
pub fn run() -> Int {
    var a = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10;
    if a == 55 { return 0; }
    return 1;
  }
use m21_deep_expr_001.run;
fn main() -> Int { return run(); }
