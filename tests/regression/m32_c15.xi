// M32-C15: Complex contract -- multiplicative inverse with comprehensive checks
fn inverse(x: Float64) -> Float64
  requires: x != 0.0
  ensures: result * x == 1.0
{
  return 1.0 / x;
}
fn safe_op(a: Float64, b: Float64) -> Float64
  requires: a != 0.0
  requires: b != 0.0
  ensures: result != 0.0
{
  return inverse(a) * inverse(b);
}
fn main() -> Int {
  var r: Float64 = safe_op(2.0, 4.0);
  if r == 0.125 { return 0; }
  return 1;
}
