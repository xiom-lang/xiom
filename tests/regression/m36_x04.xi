// M36-X04: Contract chains -- requires/ensures on function call chains
fn safe_divide(a: Int, b: Int) -> Int
  requires: b != 0
  ensures: result * b == a
{
  return a / b;
}
fn positive_sub(a: Int, b: Int) -> Int
  requires: a >= b
  ensures: result >= 0
{
  return a - b;
}
fn add_and_divide(x: Int, y: Int, z: Int) -> Int
  requires: z != 0
  ensures: result > 0
{
  var sum: Int = x + y;
  return safe_divide(sum, z);
}
fn main() -> Int {
  var r1: Int = safe_divide(100, 4);
  if r1 != 25 { return 1; }
  var r2: Int = positive_sub(50, 10);
  if r2 != 40 { return 2; }
  var r3: Int = add_and_divide(10, 30, 4);
  if r3 != 10 { return 3; }
  return 0;
}
