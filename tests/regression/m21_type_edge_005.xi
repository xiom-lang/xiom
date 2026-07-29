module m21_type_edge_005
pub fn run() -> Int {
    var a: UInt16 = 65535u16;
    if a == 65535u16 { return 0; }
    return 1;
  }
use m21_type_edge_005.run;
fn main() -> Int { return run(); }
