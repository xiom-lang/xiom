module m21_type_edge_001
pub fn run() -> Int {
    var a: Int = 42;
    var b: UInt = 100;
    if a == 42 { return 0; }
    return 1;
  }
use m21_type_edge_001.run;
fn main() -> Int { return run(); }
