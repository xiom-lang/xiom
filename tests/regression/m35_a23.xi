// M35-A23: Is perfect square -- check if n is a perfect square via integer sqrt
fn is_perfect_square(n: Int) -> Int {
  if n < 0 { return 0; }
  if n == 0 { return 1; }
  if n == 1 { return 1; }
  var x: Int = n;
  var y: Int = (x + 1) / 2;
  while y < x {
    x = y;
    y = (x + n / x) / 2;
  }
  if x * x == n { return 1; }
  return 0;
}
fn main() -> Int {
  if is_perfect_square(0) != 1 { return 1; }
  if is_perfect_square(1) != 1 { return 2; }
  if is_perfect_square(4) != 1 { return 3; }
  if is_perfect_square(9) != 1 { return 4; }
  if is_perfect_square(16) != 1 { return 5; }
  if is_perfect_square(25) != 1 { return 6; }
  if is_perfect_square(2) != 0 { return 7; }
  if is_perfect_square(8) != 0 { return 8; }
  if is_perfect_square(99) != 0 { return 9; }
  return 0;
}
