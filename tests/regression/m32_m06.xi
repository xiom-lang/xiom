// M32-M06: Dotted path access -- dot notation for const and type
module lib {
  pub const MAGIC: Int = 42;
  pub type Flag = Bool;
  pub fn check(x: Int) -> Bool { return x == MAGIC; }
}
use lib.MAGIC;
use lib.check;
fn main() -> Int {
  if MAGIC == 42 && check(42) { return 0; }
  return 1;
}
