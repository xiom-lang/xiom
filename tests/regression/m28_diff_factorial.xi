// M28: Differential test — recursive vs iterative factorial
fn fact_rec(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * fact_rec(n - 1);
}
fn fact_iter(n: Int) -> Int {
  var result: Int = 1;
  var i: Int = 1;
  while i <= n {
    result = result * i;
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if fact_rec(5) == fact_iter(5) && fact_rec(10) == fact_iter(10) { return 0; }
  return 1;
}
