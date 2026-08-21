// M25: Contract runtime -- requires and ensures enforce correctness
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures: result * b == a
{
  return a / b;
}
fn main() -> Int {
  var r: Float64 = divide(10.0, 2.0);
  if r == 5.0 { return 0; }
  return 1;
}
