module m21_struct_mut_038
type Multi = Pair{ a: Int; b: Int; c: Int; }

  pub fn run() -> Int {
    var m: Multi = Pair{ a: 1; b: 2; c: 3; };
    m.a = 10;
    m.b = 20;
    m.c = 30;
    if m.a == 10 && m.b == 20 && m.c == 30 { return 0; }
    return 1;
  }
use m21_struct_mut_038.run;
fn main() -> Int { return run(); }
