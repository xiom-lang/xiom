// M35-M24: e approximation: e = sum 1/k! for k=0..n
fn e_approx(terms: Int) -> Float64 {
  var sum: Float64 = 1.0;
  var fact: Float64 = 1.0;
  var i: Int = 1;
  while i <= terms {
    fact = fact * (i as Float64);
    sum = sum + 1.0 / fact;
    i = i + 1;
  }
  return sum;
}
fn main() -> Int {
  var e: Float64 = e_approx(15);
  if e > 2.718 && e < 2.719 { return 0; }
  return 1;
}
