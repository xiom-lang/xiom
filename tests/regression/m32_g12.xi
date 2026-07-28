// M32-G12: Generic comparator - Int only
fn max[T](a: T, b: T) -> T { if a > b { return a; } return b; }
fn main() -> Int {
  if max[Int](10, 20) != 20 { return 1; }
  if max[Int](-5, 5) != 5 { return 2; }
  return 0;
}
