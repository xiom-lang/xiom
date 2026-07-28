// M34-J06: use module — various import forms
module util {
  pub fn add_one(x: Int) -> Int { return x + 1; }
  pub fn sub_one(x: Int) -> Int { return x - 1; }
  pub type Pair = { a: Int; b: Int; }
  pub fn make_pair(a: Int, b: Int) -> Pair { return Pair{ a: a; b: b; }; }
}
use util.add_one;
use util.sub_one;
use util.make_pair;
fn main() -> Int {
  var x = add_one(10);
  var y = sub_one(10);
  var p = make_pair(x, y);
  if p.a == 11 && p.b == 9 { return 0; }
  return 1;
}
