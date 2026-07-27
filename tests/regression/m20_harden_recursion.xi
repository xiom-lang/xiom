fn fib(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib(n - 1) + fib(n - 2);
}
fn main() -> Int {
  if fib(10) != 55 { return 1; }
  if fib(0) != 0 { return 2; }
  if fib(1) != 1 { return 3; }
  return 0;
}