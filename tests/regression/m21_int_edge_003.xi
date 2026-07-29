module m21_int_edge_003
pub fn run() -> Int {
    var a: UInt8 = 0u8;
    if a == 0u8 { return 0; }
    return 1;
  }
use m21_int_edge_003.run;
fn main() -> Int { return run(); }
