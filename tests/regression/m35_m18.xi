// M35-M18: binomial coefficient C(n,k) = n! / (k! * (n-k)!)
fn binom(n: Int, k: Int) -> Int {
  if k < 0 || k > n { return 0; }
  if k == 0 || k == n { return 1; }
  var kk = k;
  if kk > n - kk { kk = n - kk; }
  var result: Int = 1;
  var i: Int = 0;
  while i < kk {
    result = result * (n - i);
    result = result / (i + 1);
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if binom(5, 0) == 1 && binom(5, 1) == 5 && binom(5, 2) == 10 && binom(5, 5) == 1 && binom(10, 3) == 120 { return 0; }
  return 1;
}
