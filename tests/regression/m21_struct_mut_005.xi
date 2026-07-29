module m21_struct_mut_005
type Record = { a: Int8; b: Int8; }

  pub fn run() -> Int {
    var r: Record = { a: 1i8; b: 2i8; };
    r.a = 100i8;
    if r.a == 100i8 && r.b == 2i8 { return 0; }
    return 1;
  }
use m21_struct_mut_005.run;
fn main() -> Int { return run(); }
