// M36-C13: Every recursion depth — 1, 2, 5, 10, 20 via factorial, sum, power, fib, countdown patterns
fn depth1(n: Int) -> Int { return n; }
fn depth2(n: Int) -> Int { return depth1(n) + 1; }
fn depth5(n: Int) -> Int {
  if n <= 0 { return 0; }
  return 1 + depth5(n - 1);
}
fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn depth10_sum(n: Int) -> Int {
  if n <= 0 { return 0; }
  return n + depth10_sum(n - 1);
}
fn fib(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib(n - 1) + fib(n - 2);
}
fn depth20_power(n: Int, base: Int) -> Int {
  if n <= 0 { return 1; }
  return base * depth20_power(n - 1, base);
}
fn ack_limited(m: Int, n: Int) -> Int {
  if m == 0 { return n + 1; }
  if n == 0 { return ack_limited(m - 1, 1); }
  return ack_limited(m - 1, ack_limited(m, n - 1));
}
fn main() -> Int {
  if depth1(5) != 5 { return 1; }
  if depth2(41) != 42 { return 2; }
  if depth5(5) != 5 { return 3; }
  if depth5(1) != 1 { return 4; }
  if depth5(0) != 0 { return 5; }
  if factorial(5) != 120 { return 6; }
  if factorial(1) != 1 { return 7; }
  if factorial(0) != 1 { return 8; }
  if depth10_sum(10) != 55 { return 9; }
  if depth10_sum(5) != 15 { return 10; }
  if depth10_sum(1) != 1 { return 11; }
  if depth10_sum(0) != 0 { return 12; }
  if fib(0) != 0 { return 13; }
  if fib(1) != 1 { return 14; }
  if fib(5) != 5 { return 15; }
  if fib(10) != 55 { return 16; }
  if depth20_power(0, 2) != 1 { return 17; }
  if depth20_power(5, 2) != 32 { return 18; }
  if depth20_power(10, 2) != 1024 { return 19; }
  if ack_limited(1, 2) != 4 { return 20; }
  if ack_limited(2, 1) != 5 { return 21; }
  return 0;
}
