// Combinatorial stress: 3 different sum implementations must agree
fn sum_while(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 1;
  while i <= n { total = total + i; i = i + 1; }
  return total;
}
fn sum_loop_guard(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 0;
  while i < n { i = i + 1; total = total + i; }
  return total;
}
fn sum_rec(n: Int) -> Int {
  if n == 0 { return 0; }
  return n + sum_rec(n - 1);
}
fn main() -> Int {
  var a = sum_while(50);
  var b = sum_loop_guard(50);
  var c = sum_rec(50);
  if a == b && b == c && a == 1275 { return 0; }
  return 1;
}
