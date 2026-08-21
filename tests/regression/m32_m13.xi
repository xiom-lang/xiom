// M32-M13: Combined pub const + pub fn -- both in one module
module lib {
  pub const FACTOR: Int = 10;
  pub fn scale(x: Int) -> Int { return x * FACTOR; }
  pub fn offset(x: Int, d: Int) -> Int { return x + d; }
}
use lib.scale;
use lib.offset;
fn main() -> Int {
  if scale(5) == 50 && offset(3, 7) == 10 { return 0; }
  return 1;
}
