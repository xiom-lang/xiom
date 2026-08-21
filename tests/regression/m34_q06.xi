// M34-Q06: Contract runtime -- valid path succeeds, contract protects invariants
fn safe_sqrt(x: Int) -> Int
  requires: x >= 0
  ensures: result * result <= x
  ensures: (result + 1) * (result + 1) > x
{
  var i: Int = 0;
  while (i + 1) * (i + 1) <= x { i = i + 1; }
  return i;
}
fn clamped(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
  ensures: result <= 100
{
  if x > 100 { return 100; }
  return x;
}
fn add_and_clamp(a: Int, b: Int) -> Int
  requires: a >= 0
  requires: b >= 0
{
  return clamped(a + b);
}
fn main() -> Int {
  var r1 = safe_sqrt(25);
  var r2 = safe_sqrt(49);
  var r3 = add_and_clamp(30, 40);
  var r4 = clamped(200);
  if r1 == 5 && r2 == 7 && r3 == 70 && r4 == 100 { return 0; }
  return 1;
}
