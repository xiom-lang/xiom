// M35-A08: Factorial — recursive computation and verify known values
fn fact(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * fact(n - 1);
}
fn main() -> Int {
  if fact(0) != 1 { return 1; }
  if fact(1) != 1 { return 2; }
  if fact(5) != 120 { return 3; }
  if fact(7) != 5040 { return 4; }
  if fact(10) != 3628800 { return 5; }
  return 0;
}
