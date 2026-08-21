// M36-X01: Exit code based on computation -- returns computed value not just 0
fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn main() -> Int {
  var a: Int = 5 + 3 * 2;
  if a != 11 { return 1; }
  var b: Int = (10 - 4) * (2 + 1);
  if b != 18 { return 2; }
  var f: Int = factorial(5);
  if f != 120 { return 3; }
  var r: Int = f / 10 + a;
  if r != 23 { return 4; }
  return r;
}
