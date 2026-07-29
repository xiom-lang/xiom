module m21_struct_mut_035
type A = { val: Int; }
  type B = Pair{ a: A; }
  type C = { b: B; }
  type D = { c: C; }
  type E = { d: D; }

  pub fn run() -> Int {
    var e: E = { d: { c: { b: { a: { val: 1; }; }; }; }; };
    e.d.c.b.a.val = 99;
    if e.d.c.b.a.val == 99 { return 0; }
    return 1;
  }
use m21_struct_mut_035.run;
fn main() -> Int { return run(); }
