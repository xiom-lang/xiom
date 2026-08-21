// M28: Differential test -- loop sum vs formula must produce same result
fn sum_loop(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 1;
  while i <= n {
    total = total + i;
    i = i + 1;
  }
  return total;
}
fn sum_formula(n: Int) -> Int { return n * (n + 1) / 2; }
fn main() -> Int {
  var a = sum_loop(100);
  var b = sum_formula(100);
  if a == b { return 0; }
  return 1;
}
