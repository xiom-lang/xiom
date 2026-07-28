// M32-M01: Module declaration — basic module with pub fn
module math {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
}
use math.add;
fn main() -> Int {
  if add(2, 3) == 5 { return 0; }
  return 1;
}
