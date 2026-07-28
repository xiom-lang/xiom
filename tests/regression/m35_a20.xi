// M35-A20: Factorial — recursion vs iteration equivalence test
fn fact_rec(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * fact_rec(n - 1);
}
fn fact_iter(n: Int) -> Int {
  var r: Int = 1;
  var i: Int = 1;
  while i <= n {
    r = r * i;
    i = i + 1;
  }
  return r;
}
fn main() -> Int {
  var i: Int = 0;
  while i <= 10 {
    if fact_rec(i) != fact_iter(i) { return 1; }
    i = i + 1;
  }
  return 0;
}
