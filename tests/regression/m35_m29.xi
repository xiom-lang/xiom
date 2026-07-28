// M35-M29: triangle area via Heron's formula: sqrt(s*(s-a)*(s-b)*(s-c)) where s = (a+b+c)/2
fn sqrt_newton(x: Float64) -> Float64 {
  if x <= 0.0 { return 0.0; }
  var guess: Float64 = x;
  var i: Int = 0;
  while i < 20 {
    guess = (guess + x / guess) * 0.5;
    i = i + 1;
  }
  return guess;
}
fn heron_area(a: Float64, b: Float64, c: Float64) -> Float64 {
  var s = (a + b + c) * 0.5;
  return sqrt_newton(s * (s - a) * (s - b) * (s - c));
}
fn main() -> Int {
  var area: Float64 = heron_area(3.0, 4.0, 5.0);
  if area > 5.99 && area < 6.01 { return 0; }
  return 1;
}
