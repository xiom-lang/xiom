module m21_struct_mut_007
type Record = Pair{ a: Int32; b: Int32; }

  pub fn run() -> Int {
    var r: Record = Pair{ a: 1i32; b: 2i32; };
    r.a = 2147483647i32;
    if r.a == 2147483647i32 && r.b == 2i32 { return 0; }
    return 1;
  }
use m21_struct_mut_007.run;
fn main() -> Int { return run(); }
