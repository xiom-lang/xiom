// M35-A09: Fibonacci — recursive implementation and verify known values
fn fib(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib(n - 1) + fib(n - 2);
}
fn main() -> Int {
  if fib(0) != 0 { return 1; }
  if fib(1) != 1 { return 2; }
  if fib(5) != 5 { return 3; }
  if fib(10) != 55 { return 4; }
  if fib(15) != 610 { return 5; }
  return 0;
}
