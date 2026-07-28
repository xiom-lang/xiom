// M35-A11: Fibonacci — DP approach using three variables (no array mutation)
fn fib_dp(n: Int) -> Int {
  if n <= 1 { return n; }
  var a: Int = 0;
  var b: Int = 1;
  var i: Int = 2;
  while i <= n {
    var c: Int = a + b;
    a = b;
    b = c;
    i = i + 1;
  }
  return b;
}
fn main() -> Int {
  if fib_dp(0) != 0 { return 1; }
  if fib_dp(1) != 1 { return 2; }
  if fib_dp(5) != 5 { return 3; }
  if fib_dp(10) != 55 { return 4; }
  if fib_dp(15) != 610 { return 5; }
  if fib_dp(20) != 6765 { return 6; }
  return 0;
}
