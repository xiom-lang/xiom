// M35-A24: Sieve-like prime count -- count primes up to n using trial division
fn is_prime(x: Int) -> Int {
  if x <= 1 { return 0; }
  var d: Int = 2;
  while d * d <= x {
    if x % d == 0 { return 0; }
    d = d + 1;
  }
  return 1;
}
fn main() -> Int {
  var n: Int = 50;
  var count: Int = 0;
  var i: Int = 2;
  while i <= n {
    if is_prime(i) == 1 { count = count + 1; }
    i = i + 1;
  }
  if count == 15 { return 0; }
  return 1;
}
