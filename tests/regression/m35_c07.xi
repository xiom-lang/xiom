// M35-C07: while with condition -- standard while loop with condition
fn sum_to(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 1;
  while i <= n { total = total + i; i = i + 1; }
  return total;
}
fn main() -> Int {
  if sum_to(0) != 0 { return 1; }
  if sum_to(1) != 1 { return 2; }
  if sum_to(5) != 15 { return 3; }
  if sum_to(10) != 55 { return 4; }
  return 0;
}
