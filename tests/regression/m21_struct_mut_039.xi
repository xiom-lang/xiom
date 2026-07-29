module m21_struct_mut_039
type Record = { a: Int; b: Int; }

  fn consume_a(r: Record) -> Int {
    return r.a;
  }

  pub fn run() -> Int {
    var r: Record = { a: 5; b: 10; };
    var x = consume_a(r);
    if x == 5 { return 0; }
    return 1;
  }
use m21_struct_mut_039.run;
fn main() -> Int { return run(); }
