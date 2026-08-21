// M34-J20: Module with all features combined -- pub fn/type/enum/const/generic/contract/invariant/derive/method/impl + cross-ref
module all_in_one {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub type Coord = { x: Int; y: Int; } derive[Eq]
  pub enum Color { Red, Green, Blue(v: Int) }
  pub const DEFAULT: Int = 42;
  pub fn ident[T](x: T) -> T { return x; }
  fn helper(x: Int) -> Int { return x * 2; }
  pub fn wrap_private(x: Int) -> Int { return helper(x); }
  pub fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
  { return a / b; }
  pub type Positive = { val: Int; invariant: val > 0; }
  pub fn make_pos(v: Int) -> Positive { return Positive{ val: v; }; }
  module nested {
    pub fn deep() -> Int { return 100; }
  }
}
type Coord = all_in_one.Coord;
interface Summable { fn total(self) -> Int; }
impl Summable for Coord {
  fn total(self) -> Int { return self.x + self.y; }
}
use all_in_one.add;
use all_in_one.Coord;
use all_in_one.safe_div;
use all_in_one.wrap_private;
use all_in_one.ident;
use all_in_one.make_pos;
use all_in_one.DEFAULT;
use all_in_one.nested.deep;
fn main() -> Int {
  var s = add(3, 4);
  if s != 7 { return 1; }
  var c = Coord{ x: 1, y: 2 };
  if c.total() != 3 { return 2; }
  var d = safe_div(100, 4);
  if d != 25 { return 3; }
  var w = wrap_private(5);
  if w != 10 { return 4; }
  var idv = ident[Int](99);
  if idv != 99 { return 5; }
  var p = make_pos(7);
  if p.val != 7 { return 6; }
  if deep() != 100 { return 7; }
  if DEFAULT != 42 { return 8; }
  if c == c { } else { return 9; }
  return 0;
}
