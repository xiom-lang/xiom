// M34-J02: Module with pub fn — multiple public functions
module calc {
  pub fn double(x: Int) -> Int { return x * 2; }
  pub fn triple(x: Int) -> Int { return x * 3; }
  pub fn square(x: Int) -> Int { return x * x; }
  pub fn abs(x: Int) -> Int { if x < 0 { return 0 - x; } return x; }
}
use calc.double;
use calc.triple;
use calc.square;
use calc.abs;
fn main() -> Int {
  var d = double(7);
  var t = triple(7);
  var s = square(7);
  var a = abs(-42);
  if d == 14 && t == 21 && s == 49 && a == 42 { return 0; }
  return 1;
}
