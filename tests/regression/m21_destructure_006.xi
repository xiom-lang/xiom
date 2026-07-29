module m21_destructure_006
pub fn run() -> Int {
    var a = 1;
    var b = 2;
    var c = 3;
    var d = 4;
    var t1 = (a, b);
    var t2 = (c, d);
    if t1.0 + t1.1 + t2.0 + t2.1 == 10 { return 0; }
    return 1;
  }
use m21_destructure_006.run;
fn main() -> Int { return run(); }
