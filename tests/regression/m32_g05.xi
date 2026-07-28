// M32-G05: Constrained generic with built-in Ord trait
fn max[T: Ord](a: T, b: T) -> T { if a > b { return a; } return b; }
fn main() -> Int {
  if max[Int](10, 20) != 20 { return 1; }
  if max[Int](-5, -10) != -5 { return 2; }
  return 0;
}
