// M35-M17: harmonic number H_n = sum 1/i for i=1..n (Float64 approximation)
fn harmonic(n: Int) -> Float64 {
  if n <= 0 { return 0.0; }
  var sum: Float64 = 0.0;
  var i: Int = 1;
  while i <= n {
    sum = sum + 1.0 / (i as Float64);
    i = i + 1;
  }
  return sum;
}
fn main() -> Int {
  var h1: Float64 = harmonic(1);
  var h3: Float64 = harmonic(3);
  var h10: Float64 = harmonic(10);
  if h1 > 0.99 && h1 < 1.01 && h3 > 1.82 && h3 < 1.84 && h10 > 2.92 && h10 < 2.94 { return 0; }
  return 1;
}
