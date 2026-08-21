// M32-X09: Combinatorial + Differential -- GCD while vs recursion with struct pair+contract
type Pair = { x: Int; y: Int; }
fn gcd_while(p: Pair) -> Int
  requires: p.x >= 0
  requires: p.y >= 0
  ensures: result >= 0
{
  var a = p.x;
  var b = p.y;
  while b != 0 { var t = b; b = a % b; a = t; }
  return a;
}
fn gcd_rec(a: Int, b: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  ensures: result >= 0
{
  if b == 0 { return a; }
  return gcd_rec(b, a % b);
}
enum GcdMethod { While, Recursive }
fn gcd(m: GcdMethod, x: Int, y: Int) -> Int {
  match m {
    While => gcd_while(Pair{ x: x; y: y; }),
    Recursive => gcd_rec(x, y),
  }
}
fn main() -> Int {
  var a1 = gcd(GcdMethod.While, 48, 18);
  var b1 = gcd(GcdMethod.Recursive, 48, 18);
  var a2 = gcd(GcdMethod.While, 100, 25);
  var b2 = gcd(GcdMethod.Recursive, 100, 25);
  if a1 == b1 && a2 == b2 && a1 == 6 && a2 == 25 { return 0; }
  return 1;
}
