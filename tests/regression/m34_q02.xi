// M34-Q02: Chained ensures — ensures propagated through call chain
fn add_one(x: Int) -> Int
  ensures: result == x + 1
{
  return x + 1;
}
fn add_two(x: Int) -> Int
  ensures: result == add_one(x) + 1
{
  return add_one(add_one(x));
}
fn add_three(x: Int) -> Int
  ensures: result == add_two(x) + 1
{
  return add_two(add_one(x));
}
fn main() -> Int {
  var r: Int = add_three(10);
  if r == 13 { return 0; }
  return 1;
}
