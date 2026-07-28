// M35-M09: GCD Euclidean algorithm (iterative)
fn gcd(a: Int, b: Int) -> Int {
  var x = a;
  var y = b;
  while y != 0 {
    var t = y;
    y = x % y;
    x = t;
  }
  return x;
}
fn main() -> Int {
  if gcd(48, 18) == 6 && gcd(100, 25) == 25 && gcd(7, 13) == 1 && gcd(0, 5) == 5 && gcd(60, 48) == 12 { return 0; }
  return 1;
}
