// M35-M12: digital root (iterative digit sum until single digit)
fn digital_root(n: Int) -> Int {
  if n < 0 { return digital_root(0 - n); }
  if n < 10 { return n; }
  var sum: Int = 0;
  var x = n;
  while x > 0 {
    sum = sum + (x % 10);
    x = x / 10;
  }
  return digital_root(sum);
}
fn main() -> Int {
  if digital_root(0) == 0 && digital_root(9) == 9 && digital_root(38) == 2 && digital_root(12345) == 6 && digital_root(999999) == 9 { return 0; }
  return 1;
}
