// M36-X02: Nested modules -- modules within modules with public exports
module math {
  pub fn square(x: Int) -> Int { return x * x; }
  module advanced {
    pub fn cube(x: Int) -> Int { return square(x) * x; }
    pub fn sum_squares(a: Int, b: Int) -> Int {
      return square(a) + square(b);
    }
  }
}
use math.advanced.cube;
use math.advanced.sum_squares;
use math.square;
fn main() -> Int {
  if square(4) != 16 { return 1; }
  if cube(3) != 27 { return 2; }
  if sum_squares(3, 4) != 25 { return 3; }
  return 0;
}
