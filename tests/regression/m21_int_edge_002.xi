module m21_int_edge_002
pub fn run() -> Int {
    var a: Int8 = -128i8;
    if a == -128i8 { return 0; }
    return 1;
  }
use m21_int_edge_002.run;
fn main() -> Int { return run(); }
