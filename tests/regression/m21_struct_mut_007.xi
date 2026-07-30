module m21_struct_mut_007
type Record = { a: Int32; b: Int32; }

fn main() -> Int {
    var r: Record = { a: 1 as Int32; b: 2 as Int32; };
    r.a = 2147483647 as Int32;
    if r.a == 2147483647 as Int32 && r.b == 2 as Int32 { return 0; }
    return 1;
  }
use m21_struct_mut_007.run;
fn main() -> Int { return run(); }
