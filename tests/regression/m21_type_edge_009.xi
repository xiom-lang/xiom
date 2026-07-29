module m21_type_edge_009
type C = { v: Int; }
  type B = { c: C; }
  type A = { b: B; }

  fn get_nested(a: A) -> Int {
    return a.b.c.v;
  }

  pub fn run() -> Int {
    var a: A = { b: { c: { v: 99; }; }; };
    if get_nested(a) == 99 { return 0; }
    return 1;
  }
use m21_type_edge_009.run;
fn main() -> Int { return run(); }
