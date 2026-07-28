// M34-Q01: Chained requires — fn a requires calls fn b which requires fn c
fn is_valid(x: Int) -> Bool
  requires: x >= 0
{
  return x < 100;
}
fn validate(x: Int) -> Bool
  requires: is_valid(x)
{
  return x % 2 == 0;
}
fn process(x: Int) -> Int
  requires: validate(x)
{
  return x * 2;
}
fn main() -> Int {
  var r: Int = process(42);
  if r == 84 { return 0; }
  return 1;
}
