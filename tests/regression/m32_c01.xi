// M32-C01: Simple requires — divisor must be non-zero
fn safe_div(a: Int, b: Int) -> Int
  requires: b != 0
{
  return a / b;
}
fn main() -> Int {
  var r: Int = safe_div(100, 5);
  if r == 20 { return 0; }
  return 1;
}
