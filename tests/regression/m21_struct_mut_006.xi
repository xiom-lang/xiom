module m21_struct_mut_006
type Record = Pair{ a: Int16; b: Int16; }

  pub fn run() -> Int {
    var r: Record = Pair{ a: 1i16; b: 2i16; };
    r.b = 32767i16;
    if r.a == 1i16 && r.b == 32767i16 { return 0; }
    return 1;
  }
use m21_struct_mut_006.run;
fn main() -> Int { return run(); }
