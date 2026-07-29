module m21_type_edge_004
pub fn run() -> Int {
    var a: UInt8 = 255u8;
    if a == 255u8 { return 0; }
    return 1;
  }
use m21_type_edge_004.run;
fn main() -> Int { return run(); }
