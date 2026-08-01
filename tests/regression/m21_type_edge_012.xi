module m21_type_edge_012
type Box[T] = { val: T; }
  type IntBox = Box[Int];
  type StrBox = Box[Str];

  pub fn run() -> Int {
    var ib: IntBox = { val: 42; };
    if ib.val == 42 { return 0; }
    return 1;
  }
use m21_type_edge_012.run;
fn main() -> Int { return run(); }
