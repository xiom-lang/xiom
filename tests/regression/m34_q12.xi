// M34-Q12: Contract with float domain -- requires guards for float operations
fn safe_div_float(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures: result * b == a
{
  return a / b;
}
fn safe_sqrt_newton(x: Float64, guess: Float64) -> Float64
  requires: x >= 0.0
  requires: guess > 0.0
  ensures: result > 0.0
{
  var g: Float64 = guess;
  var i: Int = 0;
  while i < 10 { g = (g + safe_div_float(x, g)) / 2.0; i = i + 1; }
  return g;
}
fn compute_hypotenuse(a: Float64, b: Float64) -> Float64
  requires: a >= 0.0
  requires: b >= 0.0
  ensures: result >= a
  ensures: result >= b
{
  return safe_sqrt_newton(a * a + b * b, a + b);
}
fn main() -> Int {
  var r1 = safe_div_float(100.0, 4.0);
  var r2 = compute_hypotenuse(3.0, 4.0);
  if r1 == 25.0 && r2 >= 4.9 && r2 <= 5.1 { return 0; }
  return 1;
}
