module cross_pkg.main
use math_utils;

fn main() -> Int {
  let s = math_utils.square(7);
  if s != 49 { return 1; }
  let a = math_utils.add(3, 4);
  if a != 7 { return 2; }
  return 0;
}
