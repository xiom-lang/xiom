// M34-Q17: Contract with recursion -- recursive functions with contract chains
fn factorial(n: Int) -> Int
  requires: n >= 0
  requires: n <= 10
  ensures: result >= 1
{
  if n == 0 { return 1; }
  return n * factorial(n - 1);
}
fn fib(n: Int) -> Int
  requires: n >= 0
  requires: n <= 20
  ensures: result >= 0
{
  if n == 0 { return 0; }
  if n == 1 { return 1; }
  return fib(n - 1) + fib(n - 2);
}
fn sum_fact_fib(n: Int) -> Int
  requires: n >= 0
  requires: n <= 10
{
  var f1 = factorial(n);
  var f2 = fib(n);
  return f1 + f2;
}
fn rec_chain(x: Int) -> Int
  requires: x >= 0
  requires: x <= 5
{
  var s = sum_fact_fib(x);
  if x == 0 { return s; }
  return s + rec_chain(x - 1);
}
fn main() -> Int {
  var r1 = factorial(5);
  var r2 = fib(7);
  var r3 = rec_chain(3);
  if r1 == 120 && r2 == 13 && r3 == 14 { return 0; }
  return 1;
}
