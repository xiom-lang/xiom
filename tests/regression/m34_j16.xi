// M34-J16: Module with contract -- requires/ensures on module functions
module safe {
  pub fn divide(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
  { return a / b; }
  pub fn positive(x: Int) -> Int
    requires: x > 0
    ensures: result > 0
  { return x * 2; }
  pub fn in_range(v: Int, lo: Int, hi: Int) -> Int
    requires: lo <= hi
    ensures: result >= lo && result <= hi
  {
    if v < lo { return lo; }
    if v > hi { return hi; }
    return v;
  }
}
use safe.divide;
use safe.positive;
use safe.in_range;
fn main() -> Int {
  var d = divide(100, 4);
  var p = positive(5);
  var r = in_range(7, 1, 10);
  if d == 25 && p == 10 && r == 7 { return 0; }
  return 1;
}
