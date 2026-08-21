// M35-A12: GCD -- Euclidean algorithm via while loop
fn gcd(a: Int, b: Int) -> Int {
  var x: Int = a;
  var y: Int = b;
  while y != 0 {
    var t: Int = y;
    y = x % y;
    x = t;
  }
  return x;
}
fn main() -> Int {
  if gcd(48, 18) != 6 { return 1; }
  if gcd(100, 25) != 25 { return 2; }
  if gcd(17, 13) != 1 { return 3; }
  if gcd(0, 5) != 5 { return 4; }
  if gcd(54, 24) != 6 { return 5; }
  return 0;
}
