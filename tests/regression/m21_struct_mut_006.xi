module m21_struct_mut_006
type Record = { a: Int16; b: Int16; }

fn main() -> Int {
    var r: Record = { a: 1 as Int16; b: 2 as Int16; };
    r.b = 32767 as Int16;
    if r.a == 1 as Int16 && r.b == 32767 as Int16 { return 0; }
    return 1;
  }
use m21_struct_mut_006.run;
fn main() -> Int { return run(); }
