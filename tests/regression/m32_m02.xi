// M32-M02: pub fn visibility — access pub fn via use import
module calc {
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
use calc.mul;
fn main() -> Int {
  if mul(6, 7) == 42 { return 0; }
  return 1;
}
