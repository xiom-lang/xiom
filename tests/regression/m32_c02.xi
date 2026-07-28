// M32-C02: Simple ensures — result equals input squared
fn square(x: Int) -> Int
  ensures: result == x * x
{
  return x * x;
}
fn main() -> Int {
  var r: Int = square(7);
  if r == 49 { return 0; }
  return 1;
}
