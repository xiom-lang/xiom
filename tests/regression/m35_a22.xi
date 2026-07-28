// M35-A22: Count divisors — count all positive divisors of n
fn count_divisors(n: Int) -> Int {
  if n <= 0 { return 0; }
  var c: Int = 0;
  var i: Int = 1;
  while i * i <= n {
    if n % i == 0 {
      if i * i == n { c = c + 1; } else { c = c + 2; }
    }
    i = i + 1;
  }
  return c;
}
fn main() -> Int {
  if count_divisors(1) != 1 { return 1; }
  if count_divisors(6) != 4 { return 2; }
  if count_divisors(28) != 6 { return 3; }
  if count_divisors(12) != 6 { return 4; }
  if count_divisors(7) != 2 { return 5; }
  if count_divisors(36) != 9 { return 6; }
  return 0;
}
