// M35-A19: Integer square root -- Newton's method floor approximation
fn isqrt(n: Int) -> Int {
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  var x: Int = n;
  var y: Int = (x + 1) / 2;
  while y < x {
    x = y;
    y = (x + n / x) / 2;
  }
  return x;
}
fn main() -> Int {
  if isqrt(0) != 0 { return 1; }
  if isqrt(1) != 1 { return 2; }
  if isqrt(4) != 2 { return 3; }
  if isqrt(16) != 4 { return 4; }
  if isqrt(25) != 5 { return 5; }
  if isqrt(100) != 10 { return 6; }
  if isqrt(99) != 9 { return 7; }
  if isqrt(101) != 10 { return 8; }
  return 0;
}
