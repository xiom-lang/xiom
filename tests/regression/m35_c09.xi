// M35-C09: while with continue — skip iterations inside loop
fn sum_odd_to(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 1;
  while i <= n {
    if i % 2 == 0 { i = i + 1; continue; }
    total = total + i;
    i = i + 1;
  }
  return total;
}
fn main() -> Int {
  if sum_odd_to(1) != 1 { return 1; }
  if sum_odd_to(5) != 9 { return 2; }
  if sum_odd_to(10) != 25 { return 3; }
  return 0;
}
