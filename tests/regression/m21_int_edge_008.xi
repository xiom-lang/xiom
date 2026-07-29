module m21_int_edge_008
pub fn run() -> Int {
    var a: Int = 0xFF;
    var b: Int = 0x10;
    if a == 255 && b == 16 { return 0; }
    return 1;
  }
use m21_int_edge_008.run;
fn main() -> Int { return run(); }
