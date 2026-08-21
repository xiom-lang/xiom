// M30: GCD -- while-loop vs recursive must produce identical results
fn gcd_while(a: Int, b: Int) -> Int {
  var x = a;
  var y = b;
  while y != 0 {
    var t = y;
    y = x % y;
    x = t;
  }
  return x;
}
fn gcd_rec(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return gcd_rec(b, a % b);
}
fn main() -> Int {
  if gcd_while(48, 18) == gcd_rec(48, 18) && gcd_while(100, 25) == gcd_rec(100, 25) { return 0; }
  return 1;
}
